use crate::config::task::Task;
use std::collections::HashMap;

mod reader;
mod task;

pub struct Config {
  tasks: HashMap<String, Task>,
}

impl Config {
  pub fn new() -> Config {
    Config {
      tasks: HashMap::new(),
    }
  }
}
