use crate::config::Config;
use crate::fst::dot::*;
use crate::fst::*;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::process::exit;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Dot {
  /// Path of the configuration file to visualize.
  #[structopt()]
  config: PathBuf,
  /// Path for the image. `dot` command is required.
  #[structopt()]
  image: PathBuf,
}

impl Dot {
  pub fn exec(&self) {
    if !self.config.exists() {
      eprintln!("The config file {} does not exists", self.config.display());
      return;
    }
    if let Err(e) = self.run() {
      eprintln!("{}", e);
      exit(1);
    }
  }

  fn run(&self) -> Result<(), String> {
    let yaml = fs::read_to_string(self.config.as_path())
      .map_err(|msg| format!("Can't read the config file: {}", msg))?;

    let mut config = Config::from_str(yaml.as_str())
      .map_err(|msg| format!("Can't process the config file: {}", msg))?;

    let mut graph = TaskFst::new();
    for task in config.tasks_values_mut() {
      task.set_state(graph.add_state(task.id().to_owned()));
    }

    for task in config.tasks().values() {
      if task.depends_on().len() == 0 {
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

    let mut buf: Vec<u8> = vec![];
    dot_write_file(&graph, &mut buf).map_err(|e| format!("Can't create dot file: {}", e))?;

    dot_write_png(&mut Cursor::new(buf), &self.image)
      .map_err(|e| format!("Can't save graph as png file: {}", e))?;

    Ok(())
  }
}
