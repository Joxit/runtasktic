use crate::config::{Config, WhenNotify};
use anyhow::{Context, Result};
use clap::Parser;
use std::{path::PathBuf, str::FromStr};

mod completion;
mod config;
mod dot;
mod exec;
mod run;
mod update;

#[derive(Parser, Debug)]
pub enum Command {
  /// Run all tasks from your configuration in background or foreground.
  ///
  /// Set the notification, messages, output files, concurency, working directory and many more options in your configuration.
  #[command(name = "run")]
  Run(run::Run),
  /// Export the configuration to a graph (needs graphviz/dot).
  #[command(name = "dot")]
  Dot(dot::Dot),
  /// Execute a single command with notification in background or foreground.
  ///
  /// Inherit the notification from a configuration file and set your default one in your home: `~/.runtasktic.yml` or `~/.runtasktic.yaml`.
  #[command(name = "exec")]
  Exec(exec::Exec),
  /// Generate completion script for your shell.
  #[command(name = "completion", subcommand)]
  Completion(completion::Completion),
  /// Self update of the binary.
  #[command(name = "update")]
  Update(update::Update),
  /// Parse, resolve and render configuration file.
  #[command(name = "config")]
  Config(config::Config),
}

impl Command {
  pub fn exec(&self) -> Result<()> {
    match self {
      Command::Run(executable) => executable.exec(),
      Command::Exec(executable) => executable.exec(),
      Command::Dot(executable) => executable.exec(),
      Command::Completion(executable) => executable.exec(),
      Command::Update(executable) => executable.exec(),
      Command::Config(executable) => executable.exec(),
    }
  }
}

#[derive(Debug, PartialEq, Clone, serde::Deserialize)]
#[serde(rename_all = "kebab-case", untagged)]
pub enum NotificationArg {
  All,
  None,
  Service(NotificationType),
  When(WhenNotify),
  ServiceWhen(NotificationType, WhenNotify),
}

#[derive(Debug, PartialEq, Clone, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NotificationType {
  Slack,
  Discord,
  Print,
  Email,
}

impl FromStr for NotificationArg {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s == "all" {
      Ok(NotificationArg::All)
    } else if s == "none" {
      Ok(NotificationArg::None)
    } else if let Some((stype, swhen)) = s.split_once(":") {
      let stype =
        serde_json::from_str::<NotificationType>(&format!(r#""{stype}""#)).map_err(|e| {
          format!("Incorrect notification argument `{s}`, first argument is notification type: {e}")
        })?;
      let swhen = serde_json::from_str::<WhenNotify>(&format!(r#""{swhen}""#)).map_err(|e| {
        format!("Incorrect notification argument `{s}`, second argument is when to notify: {e}")
      })?;

      Ok(Self::ServiceWhen(stype, swhen))
    } else if let Ok(notif) = serde_json::from_str(&format!(r#""{s}""#)) {
      Ok(notif)
    } else {
      Err(format!("Incorrect notification argument `{s}` not found"))
    }
  }
}

impl NotificationArg {
  fn config_mut(&self, config: &mut Config) {
    if let Some(notif) = config.notification.as_mut() {
      match self {
        Self::None => notif.when = WhenNotify::Never,
        Self::All => notif.when = WhenNotify::Always,
        Self::Service(service) => {
          Self::ServiceWhen(service.clone(), WhenNotify::Always).config_mut(config)
        }
        Self::When(when) => {
          notif.when = when.clone();
          vec![
            NotificationType::Discord,
            NotificationType::Email,
            NotificationType::Slack,
            NotificationType::Print,
          ]
          .into_iter()
          .for_each(|service| Self::ServiceWhen(service, when.clone()).config_mut(config));
        }
        Self::ServiceWhen(service, when) => match service {
          NotificationType::Discord => {
            if let Some(discord) = notif.discord.as_mut() {
              discord.when = Some(when.clone());
            }
          }
          NotificationType::Email => {
            if let Some(mail) = notif.mail.as_mut() {
              mail.when = Some(when.clone());
            }
          }
          NotificationType::Slack => {
            if let Some(slack) = notif.slack.as_mut() {
              slack.when = Some(when.clone());
            }
          }
          NotificationType::Print => {
            if let Some(print) = notif.print.as_mut() {
              print.when = Some(when.clone());
            }
          }
        },
      }
    }
  }
}

pub fn default_config_path() -> Option<PathBuf> {
  if let Ok(home) = std::env::var("HOME") {
    vec![".runtasktic", ".runtasktic.yml", ".runtasktic.yaml"]
      .iter()
      .map(|path| PathBuf::from(&home).join(path))
      .find(|path| path.as_path().exists())
  } else {
    None
  }
}

pub fn config_path(path: &PathBuf) -> Result<(Config, String)> {
  let yaml = std::fs::read_to_string(&path)
    .with_context(|| anyhow!("Can't read the config file {}", &path.display()))?;

  let config = Config::from_str(yaml.as_str())
    .with_context(|| anyhow!("Can't process the config file {}", &path.display()))?;

  Ok((config, format!("{}", path.display())))
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn notification_arg() {
    let asserts = vec![
      ("all", NotificationArg::All),
      ("none", NotificationArg::None),
      ("slack", NotificationArg::Service(NotificationType::Slack)),
      (
        "discord",
        NotificationArg::Service(NotificationType::Discord),
      ),
      ("print", NotificationArg::Service(NotificationType::Print)),
      ("email", NotificationArg::Service(NotificationType::Email)),
      ("always", NotificationArg::When(WhenNotify::Always)),
      ("task-end", NotificationArg::When(WhenNotify::TaskEnd)),
      ("end", NotificationArg::When(WhenNotify::End)),
      ("never", NotificationArg::When(WhenNotify::Never)),
      ("fail", NotificationArg::When(WhenNotify::Fail)),
      (
        "slack:fail",
        NotificationArg::ServiceWhen(NotificationType::Slack, WhenNotify::Fail),
      ),
      (
        "print:always",
        NotificationArg::ServiceWhen(NotificationType::Print, WhenNotify::Always),
      ),
      (
        "discord:task-end",
        NotificationArg::ServiceWhen(NotificationType::Discord, WhenNotify::TaskEnd),
      ),
    ];
    for (str, expected) in asserts {
      let value = NotificationArg::from_str(str);
      assert_eq!(
        value,
        Ok(expected.clone()),
        "`{value:?} == {expected:?}` failed (`{str}`)"
      );
    }
  }

  #[test]
  fn notification_arg_fail() {
    assert!(NotificationArg::from_str("failing:arg").is_err());
    assert!(NotificationArg::from_str("failing:always").is_err());
    assert!(NotificationArg::from_str("always:failing").is_err());
    assert!(NotificationArg::from_str("slack:failing").is_err());
  }
}
