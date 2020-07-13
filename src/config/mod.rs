use crate::config::task::Task;
use crate::config::yaml_trait::YamlTasksScheduler;
use std::collections::HashMap;
use yaml_rust::YamlLoader;

mod reader;
mod task;
mod yaml_trait;

pub struct Config {
  tasks: HashMap<String, Task>,
  concurrency: i64,
}

impl Config {
  pub fn from_str(s: &str) -> Result<Config, String> {
    let yaml = YamlLoader::load_from_str(s).map_err(|msg| format!("Wrong Yaml format: {}", msg))?;
    let yaml = &yaml
      .first()
      .ok_or(String::from("This config yaml is empty."))?;

    Ok(Config {
      tasks: reader::read_tasks(yaml)?,
      concurrency: yaml.get_concurrency()?,
    })
  }

  pub fn tasks(&self) -> &HashMap<String, Task> {
    &self.tasks
  }

  pub fn tasks_values_mut(&mut self) -> std::collections::hash_map::ValuesMut<String, Task> {
    self.tasks.values_mut()
  }

  pub fn concurrency(&self) -> i64 {
    self.concurrency
  }
}
