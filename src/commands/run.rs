use crate::config;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Run {
  #[structopt()]
  config: PathBuf,
}

impl Run {
  pub fn exec(&self) {
    if !self.config.exists() {
      eprintln!("The config file {} does not exists", self.config.display());
      return;
    }
    let yaml = fs::read_to_string(self.config.as_path()).unwrap();
    let tasks = config::read_tasks(yaml.as_str()).unwrap();
    let initial_states: Vec<String> = tasks
      .values()
      .filter(|task| task.depends_on.len() == 0)
      .map(|task| task.id.to_string())
      .collect();
      println!("Will start with {:?} as initial states", initial_states);
  }
}
