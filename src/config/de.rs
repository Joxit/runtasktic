use serde::de::Deserialize;
use std::collections::HashMap;
use std::env::var as get_env;

const ENV_NOTIFICATION_SLACK_URL: &str = "RUNTASKTIK_NOTIFICATION_SLACK_URL";
const ENV_NOTIFICATION_SLACK_CHANNEL: &str = "RUNTASKTIK_NOTIFICATION_SLACK_CHANNEL";
const ENV_NOTIFICATION_SLACK_USERNAME: &str = "RUNTASKTIK_NOTIFICATION_SLACK_USERNAME";

const ENV_NOTIFICATION_DISCORD_URL: &str = "RUNTASKTIK_NOTIFICATION_DISCORD_URL";
const ENV_NOTIFICATION_DISCORD_USERNAME: &str = "RUNTASKTIK_NOTIFICATION_DISCORD_USERNAME";

const ENV_NOTIFICATION_EMAIL_SMTP_HOSTNAME: &str = "RUNTASKTIK_NOTIFICATION_EMAIL_SMTP_HOSTNAME";
const ENV_NOTIFICATION_EMAIL_SMTP_USERNAME: &str = "RUNTASKTIK_NOTIFICATION_EMAIL_SMTP_USERNAME";
const ENV_NOTIFICATION_EMAIL_SMTP_SECRET: &str = "RUNTASKTIK_NOTIFICATION_EMAIL_SMTP_SECRET";

pub fn default_concurrency() -> i64 {
  -1
}

pub fn default_email_port() -> u16 {
  587
}

pub fn default_true() -> bool {
  true
}

pub fn default_email_subject() -> String {
  format!("Runtasktik: task ended")
}

pub fn deserialize_task<'de, D>(deserializer: D) -> Result<HashMap<String, super::Task>, D::Error>
where
  D: serde::Deserializer<'de>,
{
  if let Ok(mut tasks) = HashMap::deserialize(deserializer) {
    for e in tasks.iter_mut() {
      let (id, task): (&String, &mut super::Task) = e;
      task.id = id.clone();
    }
    Ok(tasks)
  } else {
    Ok(HashMap::new())
  }
}

pub fn notification_slack_url<'de, D>(deserializer: D) -> Result<String, D::Error>
where
  D: serde::Deserializer<'de>,
{
  if let Ok(env) = get_env(ENV_NOTIFICATION_SLACK_URL) {
    Ok(env)
  } else {
    String::deserialize(deserializer)
  }
}

pub fn notification_slack_channel<'de, D>(deserializer: D) -> Result<String, D::Error>
where
  D: serde::Deserializer<'de>,
{
  if let Ok(env) = get_env(ENV_NOTIFICATION_SLACK_CHANNEL) {
    Ok(env)
  } else {
    String::deserialize(deserializer)
  }
}

pub fn notification_slack_username<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
  D: serde::Deserializer<'de>,
{
  if let Ok(env) = get_env(ENV_NOTIFICATION_SLACK_USERNAME) {
    Ok(Some(env))
  } else {
    Option::deserialize(deserializer)
  }
}

pub fn notification_discord_url<'de, D>(deserializer: D) -> Result<String, D::Error>
where
  D: serde::Deserializer<'de>,
{
  if let Ok(env) = get_env(ENV_NOTIFICATION_DISCORD_URL) {
    Ok(env)
  } else {
    String::deserialize(deserializer)
  }
}

pub fn notification_discord_username<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
  D: serde::Deserializer<'de>,
{
  if let Ok(env) = get_env(ENV_NOTIFICATION_DISCORD_USERNAME) {
    Ok(Some(env))
  } else {
    Option::deserialize(deserializer)
  }
}

pub fn notification_email_smtp_hostname<'de, D>(deserializer: D) -> Result<String, D::Error>
where
  D: serde::Deserializer<'de>,
{
  if let Ok(env) = get_env(ENV_NOTIFICATION_EMAIL_SMTP_HOSTNAME) {
    Ok(env)
  } else {
    String::deserialize(deserializer)
  }
}

pub fn notification_email_smtp_username<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
  D: serde::Deserializer<'de>,
{
  if let Ok(env) = get_env(ENV_NOTIFICATION_EMAIL_SMTP_USERNAME) {
    Ok(Some(env))
  } else {
    Option::deserialize(deserializer)
  }
}

pub fn notification_email_smtp_secret<'de, D>(deserializer: D) -> Result<String, D::Error>
where
  D: serde::Deserializer<'de>,
{
  if let Ok(env) = get_env(ENV_NOTIFICATION_EMAIL_SMTP_SECRET) {
    Ok(env)
  } else {
    String::deserialize(deserializer)
  }
}
