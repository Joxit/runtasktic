use crate::config::Config;
use crate::fst::dot::*;
use crate::fst::*;
use anyhow::{anyhow, ensure, Context, Result};
use clap::Parser;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct Dot {
  /// Path of the configuration file to visualize.
  #[structopt()]
  config: PathBuf,
  /// Path for the image. `dot` command is required.
  #[structopt()]
  image: PathBuf,
}

impl Dot {
  pub fn exec(&self) -> Result<()> {
    ensure!(
      self.config.exists(),
      "The config file {} does not exists",
      self.config.display()
    );
    self.run()
  }

  fn run(&self) -> Result<()> {
    let yaml = fs::read_to_string(self.config.as_path())
      .with_context(|| anyhow!("Can't read the config file: {}", self.config.display()))?;

    let mut config = Config::from_str(yaml.as_str())
      .with_context(|| anyhow!("Can't read the config file {}", self.config.display()))?;

    let mut graph = TaskFst::new();
    for task in config.tasks_values_mut() {
      task.set_state(graph.add_state(task.id()));
    }

    for task in config.tasks().values() {
      if task.depends_on().len() == 0 {
        graph.add_start_state(task.state());
      } else {
        for prev in task.depends_on().iter() {
          let err_msg = format!("{} depends on {} but does not exists", task.id(), prev);
          let prev_state = config.tasks().get(prev).ok_or(anyhow!(err_msg))?.state();
          graph.add_arc(prev_state, task.state());
        }
      }
    }

    ensure!(
      !graph.is_cyclic(),
      "Can't execute your configuration. There is a deadlock in your tasks !"
    );

    let mut buf: Vec<u8> = vec![];
    dot_write_file(&graph, &mut buf).with_context(|| "Can't create dot file")?;

    dot_write_png(&mut Cursor::new(buf), &self.image)
      .with_context(|| "Can't save graph as png file")?;

    Ok(())
  }
}
