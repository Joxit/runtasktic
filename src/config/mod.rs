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
  messages: Messages,
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

#[derive(Debug, PartialEq, Clone)]
pub struct Messages {
  task_end: String,
  all_tasks_end: String,
  task_failed: String,
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
  pub fn new(slack: Option<Slack>, when: WhenNotify, messages: Messages) -> Notification {
    Notification {
      slack,
      when,
      messages,
    }
  }

  pub fn slack(&self) -> &Option<Slack> {
    &self.slack
  }

  pub fn when(&self) -> &WhenNotify {
    &self.when
  }

  pub fn messages(&self) -> &Messages {
    &self.messages
  }

  pub fn notify_task_end(&self, task: &Task, status_code: std::process::ExitStatus) {
    if !self.when.should_notify(&WhenNotify::TaskEnd) {
      return;
    }
    if let Some(slack) = self.slack() {
      if let Some(when) = slack.when() {
        if !when.should_notify(&WhenNotify::TaskEnd) {
          return;
        }
      }
      let msg = crate::notification::replace_templates(self.messages().task_end());
      let msg = msg.replace("{task.id}", task.id());
      let msg = msg.replace("{task.status_code}", &format!("{}", status_code));
      if let Err(e) = crate::notification::post_slack(&slack, msg.as_str()) {
        eprintln!("Can't use slack notification: {}", e);
      }
    }
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

impl WhenNotify {
  pub fn should_notify(&self, when: &WhenNotify) -> bool {
    self != &WhenNotify::Never && (self == &WhenNotify::Always || self == when)
  }
}

impl Messages {
  fn new(task_end: String, all_tasks_end: String, task_failed: String) -> Self {
    Self {
      task_end,
      all_tasks_end,
      task_failed,
    }
  }

  pub fn task_end(&self) -> &String {
    &self.task_end
  }

  pub fn all_tasks_end(&self) -> &String {
    &self.all_tasks_end
  }

  pub fn task_failed(&self) -> &String {
    &self.task_failed
  }
}

impl Default for Messages {
  fn default() -> Self {
    Self {
      task_end: String::from("Task {task.id} ended with status code {task.status_code}"),
      all_tasks_end: String::from("All tasks ended. Got {resume.success} success and {resume.failures} failure."),
      task_failed: String::from("Tasks ended prematurely. Got {resume.success} success and {resume.failures} failure. Contains one critical failure."),
    }
  }
}
