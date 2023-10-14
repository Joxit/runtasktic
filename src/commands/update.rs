use clap::Parser;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::prelude::OpenOptionsExt;
use std::{env, fs};

#[derive(Parser, Debug)]
pub struct Update {}

impl Update {
  pub fn exec(&self) {
    if let Err(err) = self.update() {
      eprintln!("{}", err);
    }
  }

  fn update(&self) -> Result<(), String> {
    let path = env::current_exe().map_err(|msg| format!("Cannot find the executable: {}", msg))?;
    let metadata = fs::metadata(&path)
      .map_err(|msg| format!("Cannot find metadata of the executable: {}", msg))?;
    if metadata.is_dir()
      || metadata.is_symlink()
      || !metadata.is_file()
      || metadata.permissions().readonly()
    {
      return Err(format!("The executable cannot be replaced."));
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

    if binary_digest != digest {
      return Err(format!(
        "Binary corrupted the downloaded sha256 does not match trusted: {} downloaded: {}",
        digest, binary_digest
      ));
    }

    let mut file = fs::OpenOptions::new()
      .create(true)
      .write(true)
      .mode(mode)
      .open(&new_path)
      .map_err(|msg| format!("Cannot create binary on disk: {}", msg))?;

    file
      .write_all(&binary)
      .map_err(|msg| format!("Cannot write binary on disk: {}", msg))?;

    fs::rename(&new_path, &path).map_err(|msg| {
      format!(
        "Cannot rename {} to {}: {}",
        new_path.display(),
        path.display(),
        msg
      )
    })?;

    Ok(())
  }

  fn get_latest_version() -> Result<String, String> {
    let response = attohttpc::get("https://api.github.com/repos/Joxit/runtasktic/releases/latest")
      .send()
      .map_err(|msg| format!("Cannot get the latest version of the project: {}", msg))?;

    if !response.is_success() {
      return Err(format!(
        "Cannot get the latest version of the project: {}",
        response.status()
      ));
    }

    let response_json = json::parse(
      &response
        .text()
        .map_err(|msg| format!("Cannot get the GitHub API release: {}", msg))?,
    )
    .map_err(|msg| format!("Cannot parse GitHub API release: {}", msg))?;

    let obj = match &response_json {
      json::JsonValue::Object(obj) => obj,
      _ => {
        return Err(format!(
          "Cannot get the latest version of the project: {}",
          response_json
        ))
      }
    };

    let tag_name = obj.get("tag_name");

    if let Some(body) = obj.get("body") {
      println!("{}", body);
    }

    let err = format!("The tag cannot be parsed");
    if let Some(test) = tag_name {
      test.as_str().map(|version| version.to_string()).ok_or(err)
    } else {
      Err(err)
    }
  }

  fn get_binary(version: &String) -> Result<Vec<u8>, String> {
    let url = format!(
      "https://github.com/Joxit/runtasktic/releases/download/{}/runtasktic-linux-x86_64",
      version
    );
    let response = attohttpc::get(url)
      .send()
      .map_err(|msg| format!("Cannot get the binary of the project: {}", msg))?;

    if !response.is_success() {
      return Err(format!(
        "Cannot get the binary of the project: {}",
        response.status()
      ));
    }

    let bytes = response
      .bytes()
      .map_err(|msg| format!("Cannot collect all the bytes of the binary: {}", msg))?;

    Ok(bytes)
  }

  fn get_sha256(version: &String) -> Result<String, String> {
    let url = format!(
      "https://github.com/Joxit/runtasktic/releases/download/{}/runtasktic-linux-x86_64.sha256",
      version
    );
    let response = attohttpc::get(url)
      .send()
      .map_err(|msg| format!("Cannot get the binary's sha256 of the project: {}", msg))?;

    if !response.is_success() {
      return Err(format!(
        "Cannot get the binary's sha256 of the project: {}",
        response.status()
      ));
    }

    let sha256 = response
      .text()
      .map_err(|msg| format!("Cannot collect the binary's sha256 of the project: {}", msg))?
      .trim()
      .split_once(" ")
      .unwrap_or_default()
      .0
      .to_string();

    Ok(sha256)
  }
}
