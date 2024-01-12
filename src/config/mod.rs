pub use crate::config::task::Task;
use crate::config::yaml_trait::YamlTasksScheduler;
use anyhow::{anyhow, Result};
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
  print: Option<Print>,
  mail: Option<Mail>,
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
pub struct Print {
  output: String,
  when: Option<WhenNotify>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Mail {
  from: (String, String),
  to: Vec<(String, String)>,
  subject: String,
  smtp: MailSMTP,
  when: Option<WhenNotify>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MailSMTP {
  hostname: String,
  port: u16,
  username: String,
  secret: String,
  tls: bool,
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
  pub fn from_str(s: &str) -> Result<Config> {
    let yaml = YamlLoader::load_from_str(s)?;
    let yaml = &yaml.first().ok_or(anyhow!("This config yaml is empty."))?;

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
  pub fn new(
    slack: Option<Slack>,
    print: Option<Print>,
    mail: Option<Mail>,
    when: WhenNotify,
    messages: Messages,
  ) -> Notification {
    Notification {
      slack,
      print,
      mail,
      when,
      messages,
    }
  }

  pub fn slack(&self) -> &Option<Slack> {
    &self.slack
  }

  pub fn print(&self) -> &Option<Print> {
    &self.print
  }

  pub fn email(&self) -> &Option<Mail> {
    &self.mail
  }

  pub fn when(&self) -> &WhenNotify {
    &self.when
  }

  pub fn messages(&self) -> &Messages {
    &self.messages
  }

  pub fn notify_task_end(&self, task: &Task, status_code: std::process::ExitStatus) {
    if !self.when().should_notify(&WhenNotify::TaskEnd) {
      return;
    }

    let short_cmd = task.short_command();
    let id = if task.id().len() > 0 {
      task.id()
    } else {
      &short_cmd
    };
    let msg = crate::notification::replace_templates(self.messages().task_end());
    let msg = msg.replace("{task.id}", id);
    let msg = msg.replace("{task.full_cmd}", &task.full_command());
    let msg = msg.replace("{task.short_cmd}", &short_cmd);
    let msg = msg.replace("{task.status_code}", &format!("{}", status_code));

    if let Some(print) = self.print() {
      if let Some(when) = print.when() {
        if !when.should_notify(&WhenNotify::TaskEnd) {
          return;
        }
      }
      if let Err(e) = crate::notification::notification_print(&print, msg.as_str()) {
        eprintln!("Can't use print notification: {}", e);
      }
    }

    if let Some(slack) = self.slack() {
      if let Some(when) = slack.when() {
        if !when.should_notify(&WhenNotify::TaskEnd) {
          return;
        }
      }
      if let Err(e) = crate::notification::post_slack(&slack, msg.as_str()) {
        eprintln!("Can't use slack notification: {}", e);
      }
    }

    if let Some(email) = self.email() {
      if let Some(when) = email.when() {
        if !when.should_notify(&WhenNotify::TaskEnd) {
          return;
        }
      }
      if let Err(e) = crate::notification::notification_email(&email, msg.as_str()) {
        eprintln!("Can't use email notification: {}", e);
      }
    }
  }

  pub fn notify_all_tasks_end(&self, success: i32, failures: i32, failed: bool) {
    if !self.when().should_notify(&WhenNotify::End) {
      return;
    }
    let msg = if !failed {
      self.messages().all_tasks_end()
    } else {
      self.messages().task_failed()
    };
    let msg = crate::notification::replace_templates(msg);
    let msg = msg.replace("{resume.success}", &format!("{}", success));
    let msg = msg.replace("{resume.failures}", &format!("{}", failures));

    if let Some(print) = self.print() {
      if let Some(when) = print.when() {
        if !when.should_notify(&WhenNotify::End) {
          return;
        }
      }
      if let Err(e) = crate::notification::notification_print(&print, msg.as_str()) {
        eprintln!("Can't use print notification: {}", e);
      }
    }

    if let Some(slack) = self.slack() {
      if let Some(when) = slack.when() {
        if !when.should_notify(&WhenNotify::End) {
          return;
        }
      }
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

impl Print {
  pub fn new(output: String, when: Option<WhenNotify>) -> Self {
    Self { output, when }
  }

  pub fn output(&self) -> &String {
    &self.output
  }

  pub fn when(&self) -> &Option<WhenNotify> {
    &self.when
  }
}

impl Mail {
  pub fn new(
    from: (String, String),
    to: Vec<(String, String)>,
    subject: String,
    smtp: MailSMTP,
    when: Option<WhenNotify>,
  ) -> Self {
    Self {
      from,
      to,
      subject,
      smtp,
      when,
    }
  }

  pub fn from(&self) -> &(String, String) {
    &self.from
  }

  pub fn to(&self) -> &Vec<(String, String)> {
    &self.to
  }

  pub fn subject(&self) -> &String {
    &self.subject
  }

  pub fn smtp_hostname(&self) -> &String {
    &self.smtp.hostname
  }

  pub fn smtp_port(&self) -> u16 {
    self.smtp.port
  }

  pub fn smtp_username(&self) -> &String {
    &self.smtp.username
  }

  pub fn smtp_secret(&self) -> &String {
    &self.smtp.secret
  }

  pub fn smtp_tls(&self) -> bool {
    self.smtp.tls
  }

  pub fn when(&self) -> &Option<WhenNotify> {
    &self.when
  }
}

impl MailSMTP {
  pub fn new(hostname: String, port: u16, username: String, secret: String, tls: bool) -> Self {
    Self {
      hostname,
      port,
      username,
      secret,
      tls,
    }
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
