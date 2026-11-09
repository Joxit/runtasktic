use crate::utils::logger;
use anyhow::Result;
use clap::Parser;
use std::collections::HashMap;
use std::{io::Write, path::PathBuf};

#[derive(Parser, Debug)]
pub struct Config {
  /// Print a list of tasks from configuration files.
  #[arg(long = "task", short = 't')]
  task: Vec<String>,
  /// Print all tasks from configuration files.
  #[arg(long = "tasks", conflicts_with = "task")]
  tasks: bool,
  /// Print also environments from configuration files.
  #[arg(long = "environments")]
  environments: bool,
  /// Print all tasks from configuration files.
  #[arg(long = "format", short = 'f', default_value = "yaml")]
  format: Format,
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
      config.tasks.iter_mut().for_each(|(_, task)| {
        task.commands = vec![task.full_command()];
        if !self.environments {
          task.env_file = None;
          task.environment = HashMap::new();
        }
      });
      if !self.environments {
        config.env_file = None;
        config.environment = HashMap::new();
      }
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
    let mut writer = std::io::stdout();

    match self.format {
      Format::Yaml => serde_yaml::to_writer(&writer, &se)?,
      Format::Json => {
        serde_json::to_writer(&writer, &se)?;
        writer.write(b"\n")?;
      }
      Format::JsonPretty => {
        serde_json::to_writer_pretty(&writer, &se)?;
        writer.write(b"\n")?;
      }
    }

    Ok(())
  }
}

#[derive(Debug, PartialEq, Clone, serde::Deserialize, serde::Serialize, clap::ValueEnum)]
enum Format {
  Yaml,
  Json,
  JsonPretty,
}
