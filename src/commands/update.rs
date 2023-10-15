use anyhow::{anyhow, bail, ensure, Context, Result};
use clap::Parser;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::prelude::OpenOptionsExt;
use std::{env, fs};

#[derive(Parser, Debug)]
pub struct Update {}

impl Update {
  pub fn exec(&self) -> Result<()> {
    self.update()
  }

  fn update(&self) -> Result<()> {
    let path = env::current_exe().with_context(|| "Cannot find the executable")?;
    let metadata = fs::metadata(&path)
      .with_context(|| anyhow!("Cannot find metadata of the executable {}", path.display()))?;
    if metadata.is_dir()
      || metadata.is_symlink()
      || !metadata.is_file()
      || metadata.permissions().readonly()
    {
      bail!("The executable cannot be replaced.");
    }
    print!(
      "The original executable has been located {}",
      path.display()
    );
    let new_path = path.with_extension("tmp");
    let mode = metadata.permissions().mode();
    let latest_version = Self::get_latest_version()?;
    let binary = Self::get_binary(&latest_version)?;
    let digest = Self::get_sha256(&latest_version)?;
    let binary_digest = sha256::digest(&binary);

    ensure!(
      binary_digest == digest,
      "Binary corrupted the downloaded sha256 does not match trusted: {} downloaded: {}",
      digest,
      binary_digest
    );

    let mut file = fs::OpenOptions::new()
      .create(true)
      .write(true)
      .mode(mode)
      .open(&new_path)
      .with_context(|| anyhow!("Cannot create binary {} on disk", new_path.display()))?;

    file
      .write_all(&binary)
      .with_context(|| anyhow!("Cannot write binary {} on disk", new_path.display()))?;

    fs::rename(&new_path, &path)
      .with_context(|| anyhow!("Cannot rename {} to {}", new_path.display(), path.display()))?;

    Ok(())
  }

  fn get_latest_version() -> Result<String> {
    let response = attohttpc::get("https://api.github.com/repos/Joxit/runtasktic/releases/latest")
      .send()
      .with_context(|| "Cannot get the latest version of the project")?;

    ensure!(
      response.is_success(),
      "Cannot get the latest version of the project from GitHub API: {}",
      response.status()
    );

    let response_json = json::parse(
      &response
        .text()
        .with_context(|| "Cannot get the GitHub API release")?,
    )
    .with_context(|| "Cannot parse GitHub API release")?;

    let obj = match &response_json {
      json::JsonValue::Object(obj) => obj,
      _ => {
        bail!(
          "Cannot get the latest version of the project: {}",
          response_json
        )
      }
    };

    let tag_name = obj.get("tag_name");

    if let Some(body) = obj.get("body") {
      println!("{}", body);
    }

    let err = anyhow!("The tag cannot be parsed");
    if let Some(test) = tag_name {
      test.as_str().map(|version| version.to_string()).ok_or(err)
    } else {
      Err(err)
    }
  }

  fn get_binary(version: &String) -> Result<Vec<u8>> {
    let url = format!(
      "https://github.com/Joxit/runtasktic/releases/download/{}/runtasktic-linux-x86_64",
      version
    );
    let response = attohttpc::get(url)
      .send()
      .with_context(|| format!("Cannot get the binary version {} of the project", version))?;

    ensure!(
      response.is_success(),
      "Cannot get the binary version {} of the project: {}",
      version,
      response.status()
    );

    let bytes = response.bytes().with_context(|| {
      anyhow!(
        "Cannot collect all the bytes of the binary version {}",
        version
      )
    })?;

    Ok(bytes)
  }

  fn get_sha256(version: &String) -> Result<String> {
    let url = format!(
      "https://github.com/Joxit/runtasktic/releases/download/{}/runtasktic-linux-x86_64.sha256",
      version
    );
    let response = attohttpc::get(url).send().with_context(|| {
      anyhow!(
        "Cannot get the binary's sha256 of the project version {}",
        version
      )
    })?;

    ensure!(
      response.is_success(),
      "Cannot get the binary's sha256 of the project version {}: {}",
      version,
      response.status()
    );

    let sha256 = response
      .text()
      .with_context(|| {
        anyhow!(
          "Cannot collect the binary's sha256 of the project version {}",
          version
        )
      })?
      .trim()
      .split_once(" ")
      .unwrap_or_default()
      .0
      .to_string();

    Ok(sha256)
  }
}
