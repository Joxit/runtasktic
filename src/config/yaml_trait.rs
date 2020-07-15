use crate::config::*;
use linked_hash_map::LinkedHashMap;
use std::collections::HashMap;
use yaml_rust::Yaml;

const TASKS_KEY: &str = "tasks";
const COMMAND_KEY: &str = "commands";
const DEPENDS_ON_KEY: &str = "depends_on";
const CONCURRENCY_KEY: &str = "concurrency";
const NOTIFICATION_KEY: &str = "notification";
const SLACK_KEY: &str = "slack";
const URL_KEY: &str = "url";
const CHANNEL_KEY: &str = "channel";
const EMOJI_KEY: &str = "emoji";

pub trait YamlTasksScheduler {
  fn get_tasks(&self) -> Result<HashMap<String, Task>, String>;
  fn get_commands(&self) -> Vec<String>;
  fn get_depends_on(&self) -> Vec<String>;
  fn get_concurrency(&self) -> Result<i64, String>;
  fn get_notification(&self) -> Result<Option<Notification>, String>;
  fn get_slack(&self) -> Result<Option<Slack>, String>;
  fn get_string(&self, key: &str) -> Result<Option<String>, String>;
}

impl YamlTasksScheduler for LinkedHashMap<Yaml, Yaml> {
  fn get_tasks(&self) -> Result<HashMap<String, Task>, String> {
    if let Some(tasks) = self.get(&Yaml::String(String::from(TASKS_KEY))) {
      if let Some(tasks) = tasks.as_hash() {
        let mut result = HashMap::new();
        for (id, task) in tasks.iter() {
          let id = id.as_str().ok_or("Task ids must be strings".to_string())?;
          let commands = task.get_commands();
          let depends_on = task.get_depends_on();
          result.insert(
            id.to_string(),
            Task::new(id.to_string(), commands, depends_on),
          );
        }
        return Ok(result);
      }
    }
    Err(String::from("Tasks not found"))
  }

  fn get_commands(&self) -> Vec<String> {
    if let Some(commands) = self.get(&Yaml::String(String::from(COMMAND_KEY))) {
      if let Some(commands) = commands.as_vec() {
        return commands
          .iter()
          .map(|c| c.as_str())
          .filter(|c| c.is_some())
          .map(|c| c.unwrap().to_string())
          .collect();
      }
    }
    vec![]
  }

  fn get_depends_on(&self) -> Vec<String> {
    if let Some(commands) = self.get(&Yaml::String(String::from(DEPENDS_ON_KEY))) {
      if let Some(commands) = commands.as_vec() {
        return commands
          .iter()
          .map(|c| c.as_str())
          .filter(|c| c.is_some())
          .map(|c| c.unwrap().to_string())
          .collect();
      }
    }
    vec![]
  }

  fn get_concurrency(&self) -> Result<i64, String> {
    if let Some(concurrency) = self.get(&Yaml::String(String::from(CONCURRENCY_KEY))) {
      if let Some(concurrency) = concurrency.as_i64() {
        if concurrency < 1 {
          Err(String::from("Concurrency must be greater than 0 !"))
        } else {
          Ok(concurrency)
        }
      } else {
        Err(String::from("Incorrect value, should be an integer"))
      }
    } else {
      Ok(-1)
    }
  }

  fn get_notification(&self) -> Result<Option<Notification>, String> {
    if let Some(notification) = self.get(&Yaml::String(String::from(NOTIFICATION_KEY))) {
      return Ok(Some(Notification {
        slack: notification.get_slack()?,
      }));
    }
    Ok(None)
  }

  fn get_slack(&self) -> Result<Option<Slack>, String> {
    if let Some(slack) = self.get(&Yaml::String(String::from(SLACK_KEY))) {
      return Ok(Some(Slack {
        url: slack
          .get_string(URL_KEY)?
          .ok_or(String::from("Slack url is required!"))?,
        channel: slack
          .get_string(CHANNEL_KEY)?
          .ok_or(String::from("Slack channel is required!"))?,
        emoji: slack.get_string(EMOJI_KEY)?,
      }));
    }
    Ok(None)
  }

  fn get_string(&self, key: &str) -> Result<Option<String>, String> {
    if let Some(value) = self.get(&Yaml::String(String::from(key))) {
      let value = value
        .as_str()
        .ok_or(format!("{} is not a string type", key))?
        .to_string();
      Ok(Some(value))
    } else {
      Ok(None)
    }
  }
}

impl YamlTasksScheduler for Yaml {
  fn get_tasks(&self) -> Result<HashMap<String, Task>, String> {
    self
      .as_hash()
      .ok_or(String::from("task key must be present !"))?
      .get_tasks()
  }

  fn get_commands(&self) -> Vec<String> {
    if let Some(commands) = self.as_hash() {
      commands.get_commands()
    } else {
      vec![]
    }
  }

  fn get_depends_on(&self) -> Vec<String> {
    if let Some(depends_on) = self.as_hash() {
      depends_on.get_depends_on()
    } else {
      vec![]
    }
  }

  fn get_concurrency(&self) -> Result<i64, String> {
    if let Some(concurrency) = self.as_hash() {
      concurrency.get_concurrency()
    } else {
      Ok(-1)
    }
  }

  fn get_notification(&self) -> Result<Option<Notification>, String> {
    if let Some(notification) = self.as_hash() {
      notification.get_notification()
    } else {
      Ok(None)
    }
  }

  fn get_slack(&self) -> Result<Option<Slack>, String> {
    if let Some(slack) = self.as_hash() {
      slack.get_slack()
    } else {
      Ok(None)
    }
  }

  fn get_string(&self, key: &str) -> Result<Option<String>, String> {
    if let Some(string) = self.as_hash() {
      string.get_string(key)
    } else {
      Ok(None)
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use yaml_rust::YamlLoader;

  #[test]
  pub fn get_tasks_single_empty_task() {
    let yaml = YamlLoader::load_from_str(
      "
    tasks:
      a:
    ",
    )
    .unwrap();
    let yaml = yaml.first().unwrap();
    let mut expected: HashMap<String, Task> = HashMap::new();
    expected.insert(
      String::from("a"),
      Task::new(String::from("a"), vec![], vec![]),
    );
    assert_eq!(yaml.get_tasks(), Ok(expected));
  }

  #[test]
  pub fn get_tasks_single_command_task() {
    let yaml = YamlLoader::load_from_str(
      "
    tasks:
      a:
        commands:
        - echo OK
    ",
    )
    .unwrap();
    let yaml = yaml.first().unwrap();
    let mut expected: HashMap<String, Task> = HashMap::new();
    expected.insert(
      String::from("a"),
      Task::new(String::from("a"), vec![String::from("echo OK")], vec![]),
    );
    assert_eq!(yaml.get_tasks(), Ok(expected));
  }

  #[test]
  pub fn get_tasks_single_command_depends_on_task() {
    let yaml = YamlLoader::load_from_str(
      "
    tasks:
      a:
        commands:
        - echo OK
        depends_on:
        - b
    ",
    )
    .unwrap();
    let yaml = yaml.first().unwrap();

    let mut expected: HashMap<String, Task> = HashMap::new();
    expected.insert(
      String::from("a"),
      Task::new(
        String::from("a"),
        vec![String::from("echo OK")],
        vec![String::from("b")],
      ),
    );
    assert_eq!(yaml.get_tasks(), Ok(expected));
  }
}
