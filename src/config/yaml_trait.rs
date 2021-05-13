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
const USERNAME_KEY: &str = "username";
const PRINT_KEY: &str = "print";
const OUTPUT_KEY: &str = "output";
const WHEN_KEY: &str = "when";
const WORKING_DIR_KEY: &str = "working_dir";
const STDOUT_KEY: &str = "stdout";
const STDERR_KEY: &str = "stderr";
const ON_FAILURE_KEY: &str = "on_failure";
const MESSAGES_KEY: &str = "messages";
const MESSAGES_TASK_END: &str = "task_end";
const MESSAGES_ALL_TASKS_END: &str = "all_tasks_end";
const MESSAGES_TASK_FAILED: &str = "task_failed";

pub trait YamlTasksScheduler {
  fn get_tasks(&self) -> Result<HashMap<String, Task>, String>;
  fn get_commands(&self) -> Vec<String>;
  fn get_depends_on(&self) -> Vec<String>;
  fn get_concurrency(&self) -> Result<i64, String>;
  fn get_notification(&self) -> Result<Option<Notification>, String>;
  fn get_slack(&self) -> Result<Option<Slack>, String>;
  fn get_print(&self) -> Result<Option<Print>, String>;
  fn get_string(&self, key: &str) -> Result<Option<String>, String>;
  fn get_when_notify(&self) -> Result<Option<WhenNotify>, String>;
  fn get_on_failure(&self) -> Result<Option<OnFailure>, String>;
  fn get_messages(&self) -> Result<Messages, String>;
  fn get_working_dir(&self) -> Result<Option<String>, String> {
    self.get_string(WORKING_DIR_KEY)
  }
  fn get_stdout(&self) -> Result<Option<String>, String> {
    self.get_string(STDOUT_KEY)
  }
  fn get_stderr(&self) -> Result<Option<String>, String> {
    self.get_string(STDERR_KEY)
  }
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
          let on_failure = task.get_on_failure()?;
          result.insert(
            id.to_string(),
            Task::new(id.to_string(), commands, depends_on, on_failure),
          );
        }
        return Ok(result);
      }
    }
    Ok(HashMap::new())
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
      return Ok(Some(Notification::new(
        notification.get_slack()?,
        notification.get_print()?,
        notification.get_when_notify()?.unwrap_or(WhenNotify::End),
        notification.get_messages()?,
      )));
    }
    Ok(None)
  }

  fn get_slack(&self) -> Result<Option<Slack>, String> {
    if let Some(slack) = self.get(&Yaml::String(String::from(SLACK_KEY))) {
      return Ok(Some(Slack::new(
        slack
          .get_string(URL_KEY)?
          .ok_or(String::from("Slack url is required!"))?,
        slack
          .get_string(CHANNEL_KEY)?
          .ok_or(String::from("Slack channel is required!"))?,
        slack.get_string(USERNAME_KEY)?,
        slack.get_string(EMOJI_KEY)?,
        slack.get_when_notify()?,
      )));
    }
    Ok(None)
  }

  fn get_print(&self) -> Result<Option<Print>, String> {
    if let Some(print) = self.get(&Yaml::String(String::from(PRINT_KEY))) {
      return Ok(Some(Print::new(
        print
          .get_string(OUTPUT_KEY)?
          .ok_or(String::from("print output is required!"))?,
        print.get_when_notify()?,
      )));
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

  fn get_when_notify(&self) -> Result<Option<WhenNotify>, String> {
    if let Some(when) = self.get_string(WHEN_KEY)? {
      match when.as_str() {
        "always" => Ok(Some(WhenNotify::Always)),
        "task-end" => Ok(Some(WhenNotify::TaskEnd)),
        "end" => Ok(Some(WhenNotify::End)),
        "never" => Ok(Some(WhenNotify::Never)),
        "" => Ok(None),
        _ => Err(format!("{} is an incorrect value for when", when)),
      }
    } else {
      Ok(None)
    }
  }

  fn get_on_failure(&self) -> Result<Option<OnFailure>, String> {
    if let Some(on_failure) = self.get_string(ON_FAILURE_KEY)? {
      match on_failure.as_str() {
        "continue" => Ok(Some(OnFailure::Continue)),
        "exit" => Ok(Some(OnFailure::Exit)),
        "" => Ok(None),
        _ => Err(format!(
          "{} is an incorrect value for on_failure",
          on_failure
        )),
      }
    } else {
      Ok(None)
    }
  }

  fn get_messages(&self) -> Result<Messages, String> {
    let default = Messages::default();
    if let Some(messages) = self.get(&Yaml::String(String::from(MESSAGES_KEY))) {
      Ok(Messages::new(
        messages
          .get_string(MESSAGES_TASK_END)?
          .ok_or(default.task_end())?,
        messages
          .get_string(MESSAGES_ALL_TASKS_END)?
          .ok_or(default.all_tasks_end())?,
        messages
          .get_string(MESSAGES_TASK_FAILED)?
          .ok_or(default.task_failed())?,
      ))
    } else {
      Ok(default)
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

  fn get_print(&self) -> Result<Option<Print>, String> {
    if let Some(print) = self.as_hash() {
      print.get_print()
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

  fn get_when_notify(&self) -> Result<Option<WhenNotify>, String> {
    if let Some(when_notify) = self.as_hash() {
      when_notify.get_when_notify()
    } else {
      Ok(None)
    }
  }

  fn get_on_failure(&self) -> Result<Option<OnFailure>, String> {
    if let Some(on_failure) = self.as_hash() {
      on_failure.get_on_failure()
    } else {
      Ok(None)
    }
  }

  fn get_messages(&self) -> Result<Messages, String> {
    if let Some(messages) = self.as_hash() {
      messages.get_messages()
    } else {
      Ok(Messages::default())
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use yaml_rust::YamlLoader;

  #[test]
  pub fn get_no_tasks() {
    let yaml = YamlLoader::load_from_str(
      "
    notifications:
      slack: none
    ",
    )
    .unwrap();
    let yaml = yaml.first().unwrap();
    let expected: HashMap<String, Task> = HashMap::new();

    assert_eq!(yaml.get_tasks(), Ok(expected));
  }

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
      Task::new(String::from("a"), vec![], vec![], None),
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
      Task::new(
        String::from("a"),
        vec![String::from("echo OK")],
        vec![],
        None,
      ),
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
        None,
      ),
    );
    assert_eq!(yaml.get_tasks(), Ok(expected));
  }
}
