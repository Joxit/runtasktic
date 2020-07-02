use crate::config;
use rustfst::prelude::*;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Run {
  /// Configuration path (YAML)
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
    let mut tasks = config::read_tasks(yaml.as_str()).unwrap();
    let initial_states: Vec<String> = tasks
      .values()
      .filter(|task| task.depends_on.len() == 0)
      .map(|task| task.id.to_string())
      .collect();

    println!("Will start with {:?} as initial states", initial_states);

    let mut graph = VectorFst::<BooleanWeight>::new();
    let s0 = graph.add_state(); // meta initial state
    for task in tasks.values_mut() {
      let state = graph.add_state();
      task.state = state;
    }

    for task in tasks.values() {
      let arc = Arc::new(0, 0, false, task.state);
      if task.depends_on.len() == 0 {
        graph.add_arc(s0, arc).unwrap();
      } else {
        for prev in task.depends_on.iter() {
          graph
            .add_arc(tasks.get(prev).unwrap().state, arc.clone())
            .unwrap();
        }
      }
    }
  }
}
