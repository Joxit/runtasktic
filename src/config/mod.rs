pub use crate::config::task::Task;
use crate::config::yaml_trait::YamlTasksScheduler;
use std::collections::HashMap;
use yaml_rust::YamlLoader;

mod task;
mod yaml_trait;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Config {
  tasks: HashMap<String, Task>,
  concurrency: i64,
  notification: Option<Notification>,
  working_dir: Option<String>,
  stdout: Option<String>,
  stderr: Option<String>,
  on_failure: OnFailure,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Notification {
  slack: Option<Slack>,
  when: WhenNotify,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Slack {
  url: String,
  channel: String,
  emoji: Option<String>,
  username: Option<String>,
  when: Option<WhenNotify>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum WhenNotify {
  Always,
  TaskEnd,
  End,
  Never,
}

#[derive(Debug, PartialEq, Clone)]
pub enum OnFailure {
  Continue,
  Exit,
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
      working_dir: yaml.get_working_dir()?,
      stdout: yaml.get_stdout()?,
      stderr: yaml.get_stderr()?,
      on_failure: yaml.get_on_failure()?.unwrap_or(OnFailure::Continue),
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

  pub fn notification(&self) -> &Option<Notification> {
    &self.notification
  }

  pub fn working_dir(&self) -> &Option<String> {
    &self.working_dir
  }

  pub fn stdout(&self) -> &Option<String> {
    &self.stdout
  }

  pub fn stderr(&self) -> &Option<String> {
    &self.stderr
  }

  pub fn on_failure(&self) -> &OnFailure {
    &self.on_failure
  }
}

impl Notification {
  pub fn new(slack: Option<Slack>, when: WhenNotify) -> Notification {
    Notification { slack, when }
  }

  pub fn slack(&self) -> &Option<Slack> {
    &self.slack
  }

  pub fn when(&self) -> &WhenNotify {
    &self.when
  }
}

impl Slack {
  pub fn new(
    url: String,
    channel: String,
    username: Option<String>,
    emoji: Option<String>,
    when: Option<WhenNotify>,
  ) -> Slack {
    Slack {
      url,
      channel,
      username,
      emoji,
      when,
    }
  }

  pub fn url(&self) -> &String {
    &self.url
  }

  pub fn channel(&self) -> &String {
    &self.channel
  }

  pub fn emoji(&self) -> &Option<String> {
    &self.emoji
  }

  pub fn username(&self) -> &Option<String> {
    &self.username
  }

  pub fn when(&self) -> &Option<WhenNotify> {
    &self.when
  }
}

impl Default for OnFailure {
  fn default() -> OnFailure {
    OnFailure::Continue
  }
}
