use crate::config::Config;
use crate::utils::logger;
use crate::utils::traits::{Executable, WaitSchedule};
use anyhow::{Context, Result};
use chrono::Local;
use clap::Parser;
use cron::Schedule;
use libc::{SIG_IGN, SIGHUP, fork, signal};
use std::fs;
use std::path::PathBuf;
use std::process::exit;
use tokio::runtime::Runtime;

#[derive(Parser, Debug)]
pub struct Exec {
  /// Configuration path (YAML).
  /// Will use config file located `~/.runtasktic.yml` or `~/.runtasktic.yaml` by default.
  /// If you want no config file execusion, use `--config -`
  #[arg(long = "config", short = 'c')]
  config: Option<PathBuf>,
  /// Run a single task from the configuration file.
  #[arg(long = "task", short = 't', requires = "config")]
  task: Option<String>,
  /// Exec the command in background
  #[arg(long = "background", short = 'b')]
  background: bool,
  /// Schedule your tasks using cron expression.
  #[arg(long = "cron")]
  cron: Option<Schedule>,
  /// Set a custom name for your task, default value will be the executable name.
  #[arg(long = "name", conflicts_with = "task")]
  name: Option<String>,
  /// Notification rule to apply `all`, `none`, `<notification provider>`, `<when>` or `<notification provider>:<when>`. Can be repeated or join with comma `,`
  #[arg(long = "notification", value_delimiter = ',')]
  notification: Vec<super::NotificationArg>,
  /// Command to execute
  #[arg()]
  command: Vec<String>,
  /// Display debug messages on STDERR
  #[arg(long = "verbose", short = 'v')]
  verbose: bool,
}

impl Exec {
  pub fn exec(&self) -> Result<()> {
    if self.verbose {
      logger::init_with_level(log::Level::Debug, "runtasktik::exec")?;
    }
    if let Some(config) = &self.config {
      ensure!(
        config == &PathBuf::from("-") || config.exists(),
        "The config file {} does not exists",
        config.display()
      );
    }
    let timezone = Local::now().timezone();

    if self.command.is_empty() && self.task.is_none() {
      crate::Runtasktic::display_help("exec");
    }

    if self.background && unsafe { fork() } != 0 {
      // The main process should return
      return Ok(());
    } else if self.background {
      // Ignoring SIGHUP in background mode
      unsafe { signal(SIGHUP, SIG_IGN) };
    }

    loop {
      self.cron.wait(timezone);

      if let Err(e) = self.run() {
        eprintln!("{:?}", e);
        if self.cron.is_none() {
          exit(1);
        }
      }

      if self.cron.is_none() {
        return Ok(());
      }
    }
  }

  fn run(&self) -> Result<()> {
    let rt = Runtime::new()?;
    let (mut config, path) = if Some(PathBuf::from("-")) == self.config {
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
        .ok_or(anyhow!(
          "The task `{task}` does not exist in your config file `{path}`"
        ))?
        .clone()
    } else {
      let id = self.name.clone().unwrap_or(format!(""));
      crate::config::Task::new(id, vec![self.command.join(" ")], vec![], None)
    };
    let label = task.label();
    debug!("Executing task {label}");

    for notif_arg in &self.notification {
      notif_arg.config_mut(&mut config);
    }

    let mut process = task.exec(&config, self.background)?;

    let exit = process.wait().unwrap();
    debug!(
      "Task {label} finished with status code {} in {}s",
      exit.status, exit.elapsed,
    );

    if let Some(notification) = config.notification {
      rt.block_on(notification.notify_task_end(&task, exit));
    }

    Ok(())
  }

  fn default_config_path(&self) -> Option<PathBuf> {
    if let Ok(home) = std::env::var("HOME") {
      vec![".runtasktic", ".runtasktic.yml", ".runtasktic.yaml"]
        .iter()
        .map(|path| PathBuf::from(&home).join(path))
        .find(|path| path.as_path().exists())
    } else {
      None
    }
  }

  fn config_path(&self, path: &PathBuf) -> Result<(Config, String)> {
    let yaml = fs::read_to_string(&path)
      .with_context(|| anyhow!("Can't read the config file {}", &path.display()))?;

    let config = Config::from_str(yaml.as_str())
      .with_context(|| anyhow!("Can't process the config file {}", &path.display()))?;

    Ok((config, format!("{}", path.display())))
  }
}
