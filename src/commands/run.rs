use crate::config::{Config, OnFailure};
use crate::fst::*;
use crate::utils::traits::CommandConfig;
use libc::{fork, signal};
use libc::{SIGHUP, SIG_IGN};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{exit, Command, Stdio};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Run {
  /// Configurations path (YAML)
  #[structopt()]
  config: Vec<PathBuf>,
  /// Override the starting task if the job had already been started before.
  /// When using many configuration files, start states must be in the first configuration file.
  /// Can be many task ids with comma separated values.
  #[structopt(long = "start", short = "s", number_of_values = 1)]
  starts: Vec<String>,
  /// Run the task in background
  #[structopt(long = "background", short = "b")]
  background: bool,
}

impl Run {
  pub fn exec(&self) {
    for config in &self.config {
      if !config.exists() {
        eprintln!("The config file {} does not exists", config.display());
        exit(1);
      }
    }

    if self.config.is_empty() {
      let clap = crate::Runtasktic::clap();
      let args = format!("{} run --help", clap.get_name());
      clap.get_matches_from(args.split(" "));
    }

    if self.background && unsafe { fork() } != 0 {
      // The main process should return
      return;
    } else if self.background {
      // Ignoring SIGHUP in background mode
      unsafe { signal(SIGHUP, SIG_IGN) };
    }

    for (i, config) in self.config.iter().enumerate() {
      let starts = if i == 0 { self.starts.clone() } else { vec![] };
      if let Err(e) = self.run(&config.as_path(), &starts) {
        eprintln!("{}", e);
        exit(1);
      }
    }
  }

  fn run(&self, config_path: &Path, starts: &Vec<String>) -> Result<(), String> {
    let yaml = fs::read_to_string(config_path)
      .map_err(|msg| format!("Can't read the config file: {}", msg))?;

    let mut config = Config::from_str(yaml.as_str())
      .map_err(|msg| format!("Can't process the config file: {}", msg))?;

    if config.tasks().is_empty() {
      return Err(format!(
        "Need at least one task in the config file to run: `{}`",
        config_path.display()
      ));
    }

    let mut graph = TaskFst::new();
    for task in config.tasks_values_mut() {
      task.set_state(graph.add_state(task.id().to_owned()));
    }

    for task in config.tasks().values() {
      if (task.depends_on().len() == 0 && starts.len() == 0)
        || (starts.len() > 0 && starts.contains(task.id()))
      {
        graph.add_start_state(task.state());
      } else {
        for prev in task.depends_on().iter() {
          let err_msg = format!("{} depends on {} but does not exists", task.id(), prev);
          let prev_state = config.tasks().get(prev).ok_or(err_msg)?.state();
          graph.add_arc(prev_state, task.state());
        }
      }
    }

    if graph.is_cyclic() {
      let err_msg = "Can't execute your configuration. There is a deadlock in your tasks !";
      return Err(err_msg.to_string());
    }

    let processes: &mut Vec<Option<std::process::Child>> = &mut vec![];
    for _ in 0..graph.len() {
      processes.push(None);
    }

    let mut exit_success = 0;
    let mut exit_failure = 0;
    let mut ask_for_exit = false;
    let graph_iter = &mut graph.iter();

    if starts.len() != 0 {
      graph
        .reachable_states()
        .iter()
        .enumerate()
        .filter(|(_, reachable)| !*reachable)
        .for_each(|(state, _)| graph_iter.set_done(state));
    }

    loop {
      if graph_iter.has_next()
        && (graph_iter.n_in_progress() < config.concurrency() || config.concurrency() < 0)
        && !ask_for_exit
      {
        let task = graph_iter.next().unwrap();
        let label = task.label().to_string();
        let cmd_line = config.tasks().get(&label).unwrap().full_command();
        let child = Command::new("sh")
          .arg("-c")
          .arg(cmd_line.to_string())
          .stdin(self.stdin())
          .stdout_opt(config.stdout(), !self.background)?
          .stderr_opt(config.stderr(), !self.background)?
          .working_dir(config.working_dir())?
          .spawn()
          .map_err(|msg| format!("Can't run command `{}`: {}", cmd_line, msg))?;
        processes[task.id()] = Some(child);
      } else if graph_iter.is_done() {
        break;
      } else {
        let mut done = 0;
        for id in 0..processes.len() {
          if let Some(child) = processes[id].as_mut() {
            if let Ok(Some(exit)) = child.try_wait() {
              let label = graph.get_state_from_id(id).label().to_string();
              let is_failure = if exit.success() {
                exit_success = exit_success + 1;
                false
              } else {
                exit_failure = exit_failure + 1;
                true
              };

              done = done + 1;
              graph_iter.mark_done(id);
              processes[id] = None;

              if let Some(notification) = config.notification() {
                notification.notify_task_end(config.tasks().get(&label).unwrap(), exit);
              }
              let on_failure = config.tasks().get(&label).unwrap().on_failure().as_ref();

              if is_failure && on_failure.unwrap_or(config.on_failure()) == &OnFailure::Exit {
                ask_for_exit = true;
              }
            }
          }
        }

        if graph_iter.n_in_progress() == 0 && ask_for_exit {
          break;
        } else if done == 0 {
          std::thread::sleep(std::time::Duration::from_millis(100));
        }
      }
    }

    if let Some(notification) = config.notification() {
      notification.notify_all_tasks_end(exit_success, exit_failure, ask_for_exit);
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
}
