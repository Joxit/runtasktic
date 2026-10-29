use self::de::*;
pub use self::task::Task;
use crate::process::ExitStatus;
use anyhow::Result;
use std::collections::HashMap;

mod de;
mod task;

#[derive(Debug, PartialEq, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Config {
  #[serde(default, deserialize_with = "deserialize_task")]
  pub tasks: HashMap<String, Task>,
  #[serde(default = "default_concurrency")]
  pub concurrency: i64,
  pub notification: Option<Notification>,
  pub working_dir: Option<String>,
  pub stdout: Option<String>,
  pub stderr: Option<String>,
  #[serde(default)]
  pub on_failure: OnFailure,
}

#[derive(Debug, PartialEq, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Notification {
  pub slack: Option<Slack>,
  pub discord: Option<Discord>,
  pub print: Option<Print>,
  #[serde(rename = "email")]
  pub mail: Option<Mail>,
  #[serde(default)]
  pub when: WhenNotify,
  #[serde(default)]
  pub messages: Messages,
}

#[derive(Debug, PartialEq, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Slack {
  #[serde(
    default = "default_notification_slack_url",
    deserialize_with = "notification_slack_url"
  )]
  pub url: String,
  #[serde(
    default = "default_notification_slack_channel",
    deserialize_with = "notification_slack_channel"
  )]
  pub channel: String,
  pub emoji: Option<String>,
  #[serde(
    default = "default_notification_slack_username",
    deserialize_with = "notification_slack_username"
  )]
  pub username: Option<String>,
  pub when: Option<WhenNotify>,
}

#[derive(Debug, PartialEq, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Discord {
  #[serde(
    default = "default_notification_discord_url",
    deserialize_with = "notification_discord_url"
  )]
  pub url: String,
  #[serde(
    default = "default_notification_discord_username",
    deserialize_with = "notification_discord_username"
  )]
  pub username: Option<String>,
  pub when: Option<WhenNotify>,
}

#[derive(Debug, PartialEq, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Print {
  pub output: String,
  pub when: Option<WhenNotify>,
}

#[derive(Debug, PartialEq, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Mail {
  pub from: MailAddress,
  pub to: MailAddress,
  #[serde(default = "default_email_subject")]
  pub subject: String,
  pub smtp: MailSMTP,
  pub when: Option<WhenNotify>,
}

#[derive(Debug, PartialEq, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case", untagged)]
pub enum MailAddress {
  A(String),
  NA {
    name: Option<String>,
    address: String,
  },
  V(Vec<MailAddress>),
}

#[derive(Debug, PartialEq, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MailSMTP {
  #[serde(
    default = "default_notification_email_smtp_hostname",
    deserialize_with = "notification_email_smtp_hostname"
  )]
  pub hostname: String,
  #[serde(default = "default_email_port")]
  pub port: u16,
  #[serde(
    default = "default_notification_email_smtp_username",
    deserialize_with = "notification_email_smtp_username"
  )]
  pub username: Option<String>,
  #[serde(
    default = "default_notification_email_smtp_secret",
    deserialize_with = "notification_email_smtp_secret"
  )]
  pub secret: String,
  #[serde(default = "default_true")]
  pub tls: bool,
}

#[derive(Debug, PartialEq, Clone, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WhenNotify {
  Always,
  TaskEnd,
  End,
  Never,
  Fail,
}

#[derive(Debug, PartialEq, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OnFailure {
  Continue,
  Exit,
}

#[derive(Debug, PartialEq, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Messages {
  pub task_end: String,
  pub all_tasks_end: String,
  pub task_failed: String,
}

impl Config {
  pub fn from_str(s: &str) -> Result<Config> {
    Ok(serde_yaml::from_str(s)?)
  }

  pub fn tasks(&self) -> &HashMap<String, Task> {
    &self.tasks
  }

  pub fn tasks_values_mut(&'_ mut self) -> std::collections::hash_map::ValuesMut<'_, String, Task> {
    self.tasks.values_mut()
  }

  pub fn working_dir(&self, task: &Task) -> Option<String> {
    if let Some(working_dir) = &task.working_dir {
      Some(working_dir.clone())
    } else {
      self.working_dir.clone()
    }
  }
}

impl Notification {
  pub async fn notify_task_end(&self, task: &Task, exit: ExitStatus) {
    let success = Some(exit.success);
    if !self.when.should_notify(&WhenNotify::TaskEnd, success) {
      return;
    }

    let short_cmd = task.short_command();
    let id = task.label();
    let msg = crate::notification::replace_templates(&self.messages.task_end);
    let msg = msg.replace("{task.id}", &id);
    let msg = msg.replace("{task.full_cmd}", &task.full_command());
    let msg = msg.replace("{task.short_cmd}", &short_cmd);
    let msg = msg.replace("{task.status_code}", &format!("{}", exit.status));
    let msg = msg.replace(
      "{task.elapsed.seconds}",
      &format!("{}", exit.elapsed_seconds()),
    );
    let msg = msg.replace("{task.elapsed.short}", &format!("{}", exit.elapsed_short()));
    let msg = msg.replace("{task.elapsed.long}", &format!("{}", exit.elapsed_long()));
    let msg = msg.replace(
      "{task.elapsed.ms}",
      &format!("{}", exit.elapsed_milliseconds()),
    );

    if let Some(print) = self.print.notify(&WhenNotify::TaskEnd, success) {
      if let Err(e) = crate::notification::notification_print(&print, msg.as_str()) {
        eprintln!("Can't use print notification: {}", e);
      }
    }

    if let Some(slack) = self.slack.notify(&WhenNotify::TaskEnd, success) {
      if let Err(e) = crate::notification::post_slack(&slack, msg.as_str()) {
        eprintln!("Can't use slack notification: {}", e);
      }
    }

    if let Some(discord) = self.discord.notify(&WhenNotify::TaskEnd, success) {
      if let Err(e) = crate::notification::post_discord(&discord, msg.as_str()) {
        eprintln!("Can't use discord notification: {}", e);
      }
    }

    if let Some(email) = self.mail.notify(&WhenNotify::TaskEnd, success) {
      if let Err(e) = crate::notification::notification_email(&email, msg.as_str()).await {
        eprintln!("Can't use email notification: {}", e);
      }
    };
  }

  pub async fn notify_all_tasks_end(&self, success: i32, failures: i32, failed: bool) {
    let is_success = Some(failures == 0);
    if !self.when.should_notify(&WhenNotify::End, is_success) {
      return;
    }
    let msg = if !failed {
      &self.messages.all_tasks_end
    } else {
      &self.messages.task_failed
    };
    let msg = crate::notification::replace_templates(msg);
    let msg = msg.replace("{resume.success}", &format!("{}", success));
    let msg = msg.replace("{resume.failures}", &format!("{}", failures));

    if let Some(print) = self.print.notify(&WhenNotify::End, is_success) {
      if let Err(e) = crate::notification::notification_print(&print, msg.as_str()) {
        eprintln!("Can't use print notification: {}", e);
      }
    }

    if let Some(slack) = self.slack.notify(&WhenNotify::End, is_success) {
      if let Err(e) = crate::notification::post_slack(&slack, msg.as_str()) {
        eprintln!("Can't use slack notification: {}", e);
      }
    }

    if let Some(discord) = self.discord.notify(&WhenNotify::End, is_success) {
      if let Err(e) = crate::notification::post_discord(&discord, msg.as_str()) {
        eprintln!("Can't use discord notification: {}", e);
      }
    }

    if let Some(email) = self.mail.notify(&WhenNotify::End, is_success) {
      if let Err(e) = crate::notification::notification_email(&email, msg.as_str()).await {
        eprintln!("Can't use email notification: {}", e);
      }
    };
  }
}

impl Mail {
  pub fn smtp_hostname(&self) -> &String {
    &self.smtp.hostname
  }

  pub fn smtp_port(&self) -> u16 {
    self.smtp.port
  }

  pub fn smtp_username(&self) -> &String {
    match &self.smtp.username {
      Some(username) => username,
      None => self.from.email(),
    }
  }

  pub fn smtp_secret(&self) -> &String {
    &self.smtp.secret
  }

  pub fn smtp_tls(&self) -> bool {
    self.smtp.tls
  }
}

impl Default for OnFailure {
  fn default() -> OnFailure {
    OnFailure::Continue
  }
}

impl WhenNotify {
  pub fn should_notify(&self, when: &WhenNotify, success: Option<bool>) -> bool {
    self != &WhenNotify::Never
      && (self == &WhenNotify::Always
        || self == when
        || (Some(false) == success && self == &WhenNotify::Fail))
  }
}

trait When {
  fn when(&self) -> &Option<WhenNotify>;
}

impl When for Print {
  fn when(&self) -> &Option<WhenNotify> {
    &self.when
  }
}

impl When for Slack {
  fn when(&self) -> &Option<WhenNotify> {
    &self.when
  }
}
impl When for Discord {
  fn when(&self) -> &Option<WhenNotify> {
    &self.when
  }
}

impl When for Mail {
  fn when(&self) -> &Option<WhenNotify> {
    &self.when
  }
}

trait Notify {
  fn notify(&self, when: &WhenNotify, success: Option<bool>) -> Self;
}

impl<T: Clone + When> Notify for Option<T> {
  fn notify(&self, state: &WhenNotify, success: Option<bool>) -> Self {
    if let Some(it) = self {
      if let Some(when) = it.when() {
        if !when.should_notify(state, success) {
          return None;
        }
      }
      return Some(it.clone());
    }
    None
  }
}

impl Default for Messages {
  fn default() -> Self {
    Self {
      task_end: String::from(
        "`{hostname}`: Task `{task.id}` ended with status code {task.status_code}",
      ),
      all_tasks_end: String::from(
        "`{hostname}`: All tasks ended. Got {resume.success} success and {resume.failures} failure.",
      ),
      task_failed: String::from(
        "`{hostname}`: Tasks ended prematurely. Got {resume.success} success and {resume.failures} failure. Contains one critical failure.",
      ),
    }
  }
}

impl Default for WhenNotify {
  fn default() -> Self {
    Self::End
  }
}

impl Into<(String, String)> for MailAddress {
  fn into(self) -> (String, String) {
    match self {
      MailAddress::A(a) => ("".into(), a.into()),
      MailAddress::NA { name, address } => (name.unwrap_or("".into()), address.into()),
      MailAddress::V(v) => v.first().map(|e| e.clone().into()).unwrap(),
    }
  }
}

impl Into<Vec<(String, String)>> for MailAddress {
  fn into(self) -> Vec<(String, String)> {
    match &self {
      MailAddress::A(a) => vec![("".into(), a.into())],
      MailAddress::NA { name, address } => {
        vec![(name.clone().unwrap_or("".into()), address.into())]
      }
      MailAddress::V(v) => v.into_iter().map(|e| e.clone().into()).collect(),
    }
  }
}

impl MailAddress {
  fn email(&self) -> &String {
    match self {
      Self::A(email) => email,
      Self::NA { name: _, address } => address,
      Self::V(addresses) => addresses.first().unwrap().email(),
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use std::fs;

  const SAMPLE_YAML: &str = "tests/resources/sample.yml";
  const NOTIFICATION_YAML: &str = "tests/resources/notification.yml";
  const CONCURRENCY_YAML: &str = "tests/resources/concurrency.yml";
  const ON_FAILURE_YAML: &str = "tests/resources/on_failure.yml";

  #[test]
  pub fn get_no_tasks() -> anyhow::Result<()> {
    let config: Config = serde_yaml::from_str(
      "
    notifications:
      slack: none
    ",
    )?;

    assert_eq!(config.notification, None);
    assert_eq!(config.tasks, HashMap::new());
    Ok(())
  }

  #[test]
  pub fn get_tasks_single_empty_task() -> anyhow::Result<()> {
    let config: Config = serde_yaml::from_str(
      "
    tasks:
      a:
    ",
    )?;
    let mut expected: HashMap<String, Task> = HashMap::new();
    expected.insert(
      String::from("a"),
      Task::new(String::from("a"), vec![], vec![], None),
    );
    assert_eq!(config.tasks, expected);
    Ok(())
  }

  #[test]
  pub fn get_tasks_single_command_task() -> anyhow::Result<()> {
    let config: Config = serde_yaml::from_str(
      "
    tasks:
      a:
        commands:
        - echo OK
    ",
    )?;
    let mut expected: HashMap<String, Task> = HashMap::new();
    expected.insert(
      String::from("a"),
      Task::new(
        String::from("a"),
        vec![String::from("echo OK")],
        vec![],
        None,
      ),
    );

    assert_eq!(config.tasks, expected);
    Ok(())
  }

  #[test]
  pub fn get_tasks_single_command_depends_on_task() -> anyhow::Result<()> {
    let config: Config = serde_yaml::from_str(
      "
    tasks:
      a:
        commands:
        - echo OK
        depends_on:
        - b
    ",
    )?;

    let mut expected: HashMap<String, Task> = HashMap::new();
    expected.insert(
      String::from("a"),
      Task::new(
        String::from("a"),
        vec![String::from("echo OK")],
        vec![String::from("b")],
        None,
      ),
    );

    assert_eq!(config.tasks, expected);
    Ok(())
  }

  #[test]
  pub fn get_email_notifiaction_default_values() -> anyhow::Result<()> {
    let notification: Notification = serde_yaml::from_str(
      "
    email:
      from: sender@example.com
      to: receiver@example.com
      smtp:
        hostname: smtp.example.com
        secret: secret-password
    ",
    )?;
    let expected_smtp = MailSMTP {
      hostname: "smtp.example.com".to_string(),
      port: 587,
      username: None,
      secret: "secret-password".to_string(),
      tls: true,
    };
    let expected_mail = Mail {
      from: MailAddress::A("sender@example.com".to_string()),
      to: MailAddress::A("receiver@example.com".to_string()),
      subject: super::de::default_email_subject(),
      smtp: expected_smtp,
      when: None,
    };
    let email = notification.mail;

    assert!(email.is_some());
    assert_eq!(email.clone(), Some(expected_mail));
    assert_eq!(
      email.clone().unwrap().smtp_username(),
      &"sender@example.com".to_string()
    );
    Ok(())
  }

  #[test]
  fn sample_yaml() {
    let yaml = fs::read_to_string(SAMPLE_YAML).unwrap();
    let config = Config::from_str(yaml.as_str()).unwrap();

    let a = Task::new("a".to_string(), vec![echo("a"), sleep("0.5")], vec![], None);
    let b = Task::new("b", vec![echo("b"), sleep("0.5")], vs(&["a"]), None);
    let c = Task::new("c", vec![echo("c")], vs(&["a"]), None);
    let d = Task::new("d", vec![echo("d")], vs(&["b", "c"]), None);

    assert_eq!(config.notification, None);
    assert_eq!(config.concurrency, -1);
    assert_eq!(config.working_dir, Some(String::from("/custom/directory")));
    assert_eq!(config.stdout, Some(String::from("none")));
    assert_eq!(config.stderr, Some(String::from("none")));
    assert_eq!(config.tasks.len(), 4);
    assert_eq!(config.tasks.get(&"a".to_string()), Some(&a));
    assert_eq!(config.tasks.get(&"b".to_string()), Some(&b));
    assert_eq!(config.tasks.get(&"c".to_string()), Some(&c));
    assert_eq!(config.tasks.get(&"d".to_string()), Some(&d));
  }

  #[test]
  fn concurrency_yaml() {
    let yaml = fs::read_to_string(CONCURRENCY_YAML).unwrap();
    let config = Config::from_str(yaml.as_str()).unwrap();

    let a1 = Task::new(
      "a1",
      vec![echo("Begin {task.id} at {date.utc.now}"), sleep("0.5"), echo("End a1")],
      vec![],
      None,
    );
    let a2 = Task::new(
      "a2",
      vec![echo("Begin {task.id} at {date.utc.today}"), sleep("1"), echo("End a2")],
      vec![],
      None,
    );
    let b = Task::new(
      "b",
      vec![echo("Begin {task.id} at {date.utc.time}"), sleep("0.5"), echo("End b")],
      vs(&["a1", "a2"]),
      None,
    );
    let c = Task::new(
      "c",
      vec![echo("Begin {task.id} at {date.local.now}"), sleep("1"), echo("End c")],
      vs(&["a1"]),
      None,
    );
    let d = Task::new(
      "d",
      vec![echo("Begin {task.id} at {date.local.today}"), sleep("0.5"), echo("End d")],
      vs(&["a1"]),
      None,
    );
    let e = Task::new(
      "e",
      vec![echo("Begin {task.id} at {date.local.time}"), sleep("0.5"), echo("End e")],
      vs(&["b", "c", "d"]),
      None,
    );
    let f = Task::new(
      "f",
      vec![echo("Begin {task.id} on {hostname}"), sleep("1"), echo("End f")],
      vs(&["c"]),
      None,
    );

    assert_eq!(config.notification, None);
    assert_eq!(config.concurrency, 2);
    assert_eq!(config.tasks.len(), 7);
    assert_eq!(config.tasks.get(&"a1".to_string()), Some(&a1));
    assert_eq!(config.tasks.get(&"a2".to_string()), Some(&a2));
    assert_eq!(config.tasks.get(&"b".to_string()), Some(&b));
    assert_eq!(config.tasks.get(&"c".to_string()), Some(&c));
    assert_eq!(config.tasks.get(&"d".to_string()), Some(&d));
    assert_eq!(config.tasks.get(&"e".to_string()), Some(&e));
    assert_eq!(config.tasks.get(&"f".to_string()), Some(&f));
  }

  #[test]
  fn notification_yaml() {
    let yaml = fs::read_to_string(NOTIFICATION_YAML).unwrap();
    let config = Config::from_str(yaml.as_str()).unwrap();

    let a = Task::new(
      "a",
      vec![echo("Begin a"), sleep("0.5"), echo("End a")],
      vec![],
      None,
    );
    let b = Task::new(
      "b",
      vec![echo("Begin b"), sleep("0.5"), echo("End b")],
      vs(&["a"]),
      None,
    );
    let c = Task::new(
      "c",
      vec![echo("Begin c"), sleep("1"), echo("End c")],
      vs(&["a"]),
      None,
    );
    let d = Task::new(
      "d",
      vec![echo("Begin d"), sleep("0.5"), echo("End d")],
      vs(&["a"]),
      None,
    );
    let e = Task::new(
      "e",
      vec![echo("Begin e"), sleep("0.5"), echo("End e")],
      vs(&["b", "c", "d"]),
      None,
    );

    let slack = Slack {
      url: "https://hooks.slack.com/services/XXXXX/XXXXX/XXXXX".to_string(),
      channel: "#channel".to_string(),
      username: Some("runtasktic".to_string()),
      emoji: Some(":rocket:".to_string()),
      when: None,
    };
    let discord = Discord {
      url: "https://discord.com/api/webhooks/XXXXX/XXXXX".to_string(),
      username: Some("runtasktic".to_string()),
      when: None,
    };
    let print = Print {
      output: "stderr".to_string(),
      when: None,
    };
    let email = Mail {
      from: MailAddress::NA {
        name: Some("Sender Name".to_string()),
        address: "sender@example.com".to_string(),
      },
      to: MailAddress::V(vec![MailAddress::NA {
        name: Some("Receiver Name".to_string()),
        address: "receiver@example.com".to_string(),
      }]),
      subject: "Subject".to_string(),
      smtp: MailSMTP {
        hostname: "smtp.example.com".to_string(),
        port: 1587,
        username: Some("sender@example.com".to_string()),
        secret: "secret-password".to_string(),
        tls: false,
      },
      when: None,
    };
    let notification = Notification {
      slack: Some(slack),
      discord: Some(discord),
      print: Some(print),
      mail: Some(email),
      when: WhenNotify::Always,
      messages: Messages::default(),
    };

    assert_eq!(config.notification, Some(notification));
    assert_eq!(config.concurrency, -1);
    assert_eq!(config.tasks.len(), 5);
    assert_eq!(config.tasks.get(&"a".to_string()), Some(&a));
    assert_eq!(config.tasks.get(&"b".to_string()), Some(&b));
    assert_eq!(config.tasks.get(&"c".to_string()), Some(&c));
    assert_eq!(config.tasks.get(&"d".to_string()), Some(&d));
    assert_eq!(config.tasks.get(&"e".to_string()), Some(&e));
  }

  #[test]
  fn on_failure_yaml() {
    let yaml = fs::read_to_string(ON_FAILURE_YAML).unwrap();
    let config = Config::from_str(yaml.as_str()).unwrap();

    let a = Task::new(
      "a",
      vec![echo("Begin a"), format!("unknown-cmd"), echo("End a")],
      vec![],
      Some(OnFailure::Continue),
    );
    let b = Task::new(
      "b",
      vec![echo("Begin b"), format!("unknown-cmd"), echo("End b")],
      vs(&["a"]),
      Some(OnFailure::Exit),
    );
    let c = Task::new(
      "c",
      vec![echo("Begin c"), sleep("1"), echo("End c")],
      vs(&["a"]),
      None,
    );
    let d = Task::new(
      "d",
      vec![echo("Begin d"), sleep("0.5"), echo("End d")],
      vs(&["a"]),
      None,
    );
    let e = Task::new(
      "e",
      vec![echo("Begin e"), sleep("0.5"), echo("End e")],
      vs(&["b", "c", "d"]),
      None,
    );

    assert_eq!(config.on_failure, OnFailure::Continue);
    assert_eq!(config.notification, None);
    assert_eq!(config.concurrency, 2);
    assert_eq!(config.tasks.len(), 5);
    assert_eq!(config.tasks.get(&"a".to_string()), Some(&a));
    assert_eq!(config.tasks.get(&"b".to_string()), Some(&b));
    assert_eq!(config.tasks.get(&"c".to_string()), Some(&c));
    assert_eq!(config.tasks.get(&"d".to_string()), Some(&d));
    assert_eq!(config.tasks.get(&"e".to_string()), Some(&e));
  }

  fn echo(msg: &str) -> String {
    format!("echo {}", msg)
  }

  fn sleep(time: &str) -> String {
    format!("sleep {}", time)
  }

  fn vs(vec: &[&str]) -> Vec<String> {
    vec.iter().map(|s| s.to_string()).collect()
  }
}
