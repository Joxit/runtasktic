use crate::config::task::Task;
use crate::config::yaml_trait::YamlTasksScheduler;
use std::collections::HashMap;
use yaml_rust::YamlLoader;

mod task;
mod yaml_trait;

#[derive(Debug, PartialEq, Clone)]
pub struct Config {
  tasks: HashMap<String, Task>,
  concurrency: i64,
  notification: Option<Notification>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Notification {
  slack: Option<Slack>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Slack {
  url: String,
  channel: String,
  emoji: Option<String>,
}

impl Config {
  pub fn from_str(s: &str) -> Result<Config, String> {
    let yaml = YamlLoader::load_from_str(s).map_err(|msg| format!("Wrong Yaml format: {}", msg))?;
    let yaml = &yaml
      .first()
      .ok_or(String::from("This config yaml is empty."))?;

    Ok(Config {
      tasks: yaml.get_tasks()?,
      concurrency: yaml.get_concurrency()?,
      notification: yaml.get_notification()?,
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

impl Notification {
  pub fn slack(&self) -> &Slack {
    &self.slack()
  }
}

impl Slack {
  pub fn url(&self) -> &String {
    &self.url
  }

  pub fn channel(&self) -> &String {
    &self.channel
  }

  pub fn emoji(&self) -> &Option<String> {
    &self.emoji
  }
}
