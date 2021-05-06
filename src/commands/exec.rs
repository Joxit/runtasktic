use crate::config::{Config, WhenNotify};
use crate::utils::traits::CommandConfig;
use libc::{fork, signal};
use libc::{SIGHUP, SIG_IGN};
use std::fs;
use std::path::PathBuf;
use std::process::{exit, Command, Stdio};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Exec {
  /// Configuration path (YAML)
  #[structopt(long = "config", short = "c")]
  config: Option<PathBuf>,
  /// Run a single task from the configuration file.
  #[structopt(long = "task", short = "t", requires = "config")]
  task: Option<String>,
  /// Exec the command in background
  #[structopt(long = "background", short = "b")]
  background: bool,
  /// Command to execute
  #[structopt()]
  command: Vec<String>,
}

impl Exec {
  pub fn exec(&self) {
    if let Some(config) = &self.config {
      if !config.exists() {
        eprintln!("The config file {} does not exists", config.display());
        return;
      }
    }

    if self.command.is_empty() && self.task.is_none() {
      let clap = crate::Runtasktic::clap();
      let args = format!("{} exec --help", clap.get_name());
      clap.get_matches_from(args.split(" "));
    }

    if self.background && unsafe { fork() } != 0 {
      // The main process should return
      return;
    } else if self.background {
      // Ignoring SIGHUP in background mode
      unsafe { signal(SIGHUP, SIG_IGN) };
    }

    if let Err(e) = self.run() {
      eprintln!("{}", e);
      exit(1);
    }
  }

  fn run(&self) -> Result<(), String> {
    let (config, path) = if let Some(path) = &self.config {
      let yaml = fs::read_to_string(path.as_path())
        .map_err(|msg| format!("Can't read the config file: {}", msg))?;

      let config = Config::from_str(yaml.as_str())
        .map_err(|msg| format!("Can't process the config file: {}", msg))?;

      (config, format!("{}", path.display()))
    } else {
      (Config::default(), format!("<No Config File Path>"))
    };

    let cmd_line = if let Some(task) = &self.task {
      config
        .tasks()
        .get(task)
        .ok_or(format!(
          "The task `{}` does not exist in your config file `{}`",
          task, path
        ))?
        .full_command()
    } else {
      self.command.join(" ")
    };

    let first_cmd = cmd_line.splitn(2, " ").next().unwrap();

    let mut child = Command::new("sh")
      .arg("-c")
      .arg(cmd_line.clone())
      .stdin(self.stdin())
      .stdout_opt(config.stdout(), !self.background)?
      .stderr_opt(config.stderr(), !self.background)?
      .working_dir(config.working_dir())?
      .spawn()
      .map_err(|msg| format!("Can't run command `{}`: {}", cmd_line, msg))?;

    if let Ok(exit) = child.wait() {
      let msg = format!("Command `{}` ended with status code {}", &first_cmd, exit);
      self.notify(&config, msg, WhenNotify::End);
    } else {
      let msg = format!("Command `{}` ended with failure", &first_cmd);
      self.notify(&config, msg, WhenNotify::End);
    }

    Ok(())
  }

  fn notify(&self, config: &Config, msg: String, when: WhenNotify) {
    if let Some(notification) = config.notification() {
      if *notification.when() == WhenNotify::Never
        || (*notification.when() != WhenNotify::Always && *notification.when() != when)
      {
        return;
      }
      if let Some(slack) = notification.slack() {
        if let Some(when_slack) = slack.when() {
          if *when_slack == WhenNotify::Never
            || (*when_slack != WhenNotify::Always && *when_slack != when)
          {
            return;
          }
        }
        if let Err(e) = crate::notification::post_slack(&slack, msg.as_str()) {
          eprintln!("Can't use slac notification: {}", e);
        }
      }
    }
  }

  fn stdin(&self) -> Stdio {
    if self.background {
      Stdio::null()
    } else {
      Stdio::inherit()
    }
  }
}
