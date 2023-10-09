use crate::config::Config;
use crate::utils::traits::{CommandConfig, WaitSchedule};
use chrono::Local;
use cron::Schedule;
use libc::{fork, signal};
use libc::{SIGHUP, SIG_IGN};
use std::fs;
use std::path::PathBuf;
use std::process::{exit, Command, Stdio};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Exec {
  /// Configuration path (YAML).
  /// Will use config file located `~/.runtasktic.yml` or `~/.runtasktic.yaml` by default.
  /// If you want no config file execusion, use `--config -`
  #[structopt(long = "config", short = "c")]
  config: Option<PathBuf>,
  /// Run a single task from the configuration file.
  #[structopt(long = "task", short = "t", requires = "config")]
  task: Option<String>,
  /// Exec the command in background
  #[structopt(long = "background", short = "b")]
  background: bool,
  /// Schedule your tasks using cron expression.
  #[structopt(long = "cron")]
  cron: Option<Schedule>,
  /// Command to execute
  #[structopt()]
  command: Vec<String>,
}

impl Exec {
  pub fn exec(&self) {
    if let Some(config) = &self.config {
      if config != &PathBuf::from("-") && !config.exists() {
        eprintln!("The config file {} does not exists", config.display());
        return;
      }
    }
    let timezone = Local::now().timezone();

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

    loop {
      self.cron.wait(timezone);

      if let Err(e) = self.run() {
        eprintln!("{}", e);
        if self.cron.is_none() {
          exit(1);
        }
      }

      if self.cron.is_none() {
        break;
      }
    }
  }

  fn run(&self) -> Result<(), String> {
    let (config, path) = if Some(PathBuf::from("-")) == self.config {
      (Config::default(), format!("-"))
    } else if let Some(path) = &self.config {
      self.config_path(path)?
    } else if let Some(path) = self.default_config_path() {
      self.config_path(&path)?
    } else {
      (Config::default(), format!("<No Config File Path>"))
    };

    let task = if let Some(task) = &self.task {
      config
        .tasks()
        .get(task)
        .ok_or(format!(
          "The task `{}` does not exist in your config file `{}`",
          task, path
        ))?
        .clone()
    } else {
      crate::config::Task::new(format!(""), vec![self.command.join(" ")], vec![], None)
    };
    let cmd_line = task.full_command();

    let mut child = Command::new("sh")
      .arg("-c")
      .arg(&cmd_line)
      .stdin(self.stdin())
      .stdout_opt(config.stdout(), !self.background)?
      .stderr_opt(config.stderr(), !self.background)?
      .working_dir(config.working_dir())?
      .spawn()
      .map_err(|msg| format!("Can't run command `{}`: {}", cmd_line, msg))?;

    let exit = child.wait().unwrap();
    if let Some(notification) = config.notification() {
      notification.notify_task_end(&task, exit);
    }

    Ok(())
  }

  fn stdin(&self) -> Stdio {
    if self.background {
      Stdio::null()
    } else {
      Stdio::inherit()
    }
  }

  fn default_config_path(&self) -> Option<PathBuf> {
    if let Ok(home) = std::env::var("HOME") {
      vec![".runtasktic.yml", ".runtasktic.yaml"]
        .iter()
        .map(|path| PathBuf::from(&home).join(path))
        .find(|path| path.as_path().exists())
    } else {
      None
    }
  }

  fn config_path(&self, path: &PathBuf) -> Result<(Config, String), String> {
    let yaml = fs::read_to_string(path.as_path())
      .map_err(|msg| format!("Can't read the config file: {}", msg))?;

    let config = Config::from_str(yaml.as_str())
      .map_err(|msg| format!("Can't process the config file: {}", msg))?;

    Ok((config, format!("{}", path.display())))
  }
}
