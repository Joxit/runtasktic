use crate::utils::logger;
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct Config {
  /// Print a list of tasks from configuration files.
  #[arg(long = "task", short = 't')]
  task: Vec<String>,
  /// Print all tasks from configuration files.
  #[arg(long = "tasks", conflicts_with = "task")]
  tasks: bool,
  /// Configuration path (YAML).
  /// Will use config file located `~/.runtasktic.yml` or `~/.runtasktic.yaml` by default.
  #[arg()]
  configs: Vec<PathBuf>,
  /// Display debug messages on STDERR
  #[arg(long = "verbose", short = 'v')]
  verbose: bool,
}

impl Config {
  pub fn exec(&self) -> Result<()> {
    if self.verbose {
      logger::init_with_level(log::Level::Debug, "runtasktik::exec")?;
    }
    let configs = if self.configs.len() > 0 {
      &self.configs
    } else if let Some(config) = super::default_config_path() {
      &vec![config]
    } else {
      crate::Runtasktic::display_help("exec");
      bail!("No configuration found, even on default locations");
    };

    for path in configs {
      info!("Reading configuration {}", path.display());
      let (mut config, _) = super::config_path(path)?;
      config
        .tasks
        .iter_mut()
        .for_each(|(_, task)| task.commands = vec![task.full_command()]);
      if !self.task.is_empty() {
        for task in self
          .task
          .iter()
          .map(|task| config.tasks.get(task))
          .filter(|task| task.is_some())
          .map(|task| task.unwrap())
        {
          self.to_stdout(&task)?;
        }
      } else if self.tasks {
        for (_, task) in config.tasks.iter() {
          self.to_stdout(&task)?;
        }
      } else {
        self.to_stdout(&config)?;
      }
    }

    Ok(())
  }

  fn to_stdout<S: serde::Serialize>(&self, se: S) -> anyhow::Result<()> {
    serde_yaml::to_writer(&std::io::stdout(), &se)?;
    Ok(())
  }
}
