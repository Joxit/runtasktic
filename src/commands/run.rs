use crate::config;
use crate::fst::*;
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

    let mut graph = TaskFst::new();
    for task in tasks.values_mut() {
      task.state = graph.add_state(task.id.to_string());
    }

    for task in tasks.values() {
      if task.depends_on.len() == 0 {
        graph.add_start_state(task.state);
      } else {
        for prev in task.depends_on.iter() {
          graph.add_arc(tasks.get(prev).unwrap().state, task.state);
        }
      }
    }

    let processes: &mut Vec<Option<std::process::Child>> = &mut vec![];
    for _ in 0..graph.states.len() {
      processes.push(None);
    }

    let graph_iter = &mut graph.iter();
    loop {
      if let Some(task) = graph_iter.next() {
        let label = task.label.to_string();
        let child = std::process::Command::new("sh")
          .arg("-c")
          .arg(tasks.get(&label).unwrap().commands.join(" && "))
          .spawn()
          .unwrap();
        processes[task.id] = Some(child);
      } else if graph_iter.is_done() {
        break;
      } else {
        let mut done = 0;
        for id in 0..processes.len() {
          if let Some(child) = processes[id].as_mut() {
            if child.try_wait().unwrap().is_some() {
              done = done + 1;
              graph_iter.mark_done(id);
              processes[id] = None;
            }
          }
        }
        if done == 0 {
          std::thread::sleep(std::time::Duration::from_millis(100));
        }
      }
    }
  }
}
