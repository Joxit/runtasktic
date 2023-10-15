use anyhow::{ensure, Context, Result};
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::process::{Command, Stdio};

pub trait CommandConfig {
  fn working_dir(&mut self, dir: &Option<String>) -> Result<&mut Self>;
  fn stdout_opt(&mut self, stdout: &Option<String>, inherit: bool) -> Result<&mut Self>;
  fn stderr_opt(&mut self, stdout: &Option<String>, inherit: bool) -> Result<&mut Self>;
}

impl CommandConfig for Command {
  fn working_dir(&mut self, dir: &Option<String>) -> Result<&mut Self> {
    if let Some(d) = dir {
      ensure!(
        Path::new(d).is_dir(),
        "Invalid working directory: `{}` is not a directory",
        d
      );
      Ok(self.current_dir(d))
    } else {
      Ok(self)
    }
  }

  fn stdout_opt(&mut self, stdout: &Option<String>, inherit: bool) -> Result<&mut Self> {
    let stdio = process_stdio(stdout, inherit, "runtasktic.out")?;

    Ok(self.stdout(stdio))
  }

  fn stderr_opt(&mut self, stderr: &Option<String>, inherit: bool) -> Result<&mut Self> {
    let stdio = process_stdio(stderr, inherit, "runtasktic.err")?;

    Ok(self.stderr(stdio))
  }
}

fn process_stdio(
  file: &Option<String>,
  inherit: bool,
  default_file: &'static str,
) -> Result<Stdio> {
  let res = if let Some(stdio) = file {
    match stdio.as_str() {
      "none" | "/dev/null" => Stdio::null(),
      _ => Stdio::from(open_options(stdio)?),
    }
  } else if !inherit {
    Stdio::from(open_options(default_file)?)
  } else {
    Stdio::inherit()
  };

  Ok(res)
}

fn open_options<P: AsRef<Path>>(file: P) -> Result<File> {
  OpenOptions::new()
    .create(true)
    .append(true)
    .open(file.as_ref())
    .with_context(|| format!("Can't open output file {}", file.as_ref().display()))
}
