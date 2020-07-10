use crate::config;
use crate::fst::*;
use libc::{fork, signal};
use libc::{SIGHUP, SIG_IGN};
use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use std::process::{exit, Command, Stdio};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Run {
  /// Configuration path (YAML)
  #[structopt()]
  config: PathBuf,
  /// Run the task in background
  #[structopt(long = "background", short = "b")]
  background: bool,
}

impl Run {
  pub fn exec(&self) {
    if !self.config.exists() {
      eprintln!("The config file {} does not exists", self.config.display());
      return;
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
    let yaml = fs::read_to_string(self.config.as_path())
      .map_err(|msg| format!("Can't read the config file: {}", msg))?;

    let mut tasks = config::read_tasks(yaml.as_str())
      .map_err(|msg| format!("Can't process the config file: {}", msg))?;

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
          let err_msg = format!("{} depends on {} but does not exists", task.id, prev);
          let prev_state = tasks.get(prev).ok_or(err_msg)?.state;
          graph.add_arc(prev_state, task.state);
        }
      }
    }

    if graph.is_cyclic() {
      let err_msg = "Can't execute your configuration. There is a deadlock in your tasks !";
      return Err(err_msg.to_string());
    }

    let processes: &mut Vec<Option<std::process::Child>> = &mut vec![];
    for _ in 0..graph.states.len() {
      processes.push(None);
    }

    let graph_iter = &mut graph.iter();
    loop {
      if let Some(task) = graph_iter.next() {
        let label = task.label.to_string();
        let cmd_line = tasks.get(&label).unwrap().commands.join(" && ");
        let child = Command::new("sh")
          .arg("-c")
          .arg(cmd_line.to_string())
          .stdin(self.stdin())
          .stdout(self.stdout()?)
          .stderr(self.stderr()?)
          .spawn()
          .map_err(|msg| format!("Can't run command `{}`: {}", cmd_line, msg))?;
        processes[task.id] = Some(child);
      } else if graph_iter.is_done() {
        return Ok(());
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

  fn stdout(&self) -> Result<Stdio, String> {
    if self.background {
      Ok(Stdio::from(
        OpenOptions::new()
          .create(true)
          .append(true)
          .open("task-scheduler.out")
          .map_err(|msg| format!("Can't open output file: {}", msg))?,
      ))
    } else {
      Ok(Stdio::piped())
    }
  }

  fn stdin(&self) -> Stdio {
    if self.background {
      Stdio::null()
    } else {
      Stdio::piped()
    }
  }

  fn stderr(&self) -> Result<Stdio, String> {
    if self.background {
      Ok(Stdio::from(
        OpenOptions::new()
          .create(true)
          .append(true)
          .open("task-scheduler.err")
          .map_err(|msg| format!("Can't open error file: {}", msg))?,
      ))
    } else {
      Ok(Stdio::piped())
    }
  }
}
