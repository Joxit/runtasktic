use crate::config::yaml_trait::YamlTasksScheduler;
use crate::config::Task;
use std::collections::HashMap;
use yaml_rust::Yaml;

pub fn read_tasks(yaml: &Yaml) -> Result<HashMap<String, Task>, String> {
  let mut result = HashMap::new();
  let tasks = yaml.get_tasks()?;

  for (id, task) in tasks.iter() {
    let id = id.as_str().ok_or("Task ids must be strings".to_string())?;
    let commands = task.get_commands();
    let depends_on = task.get_depends_on();

    result.insert(
      id.to_string(),
      Task::new(id.to_string(), commands, depends_on),
    );
  }

  Ok(result)
}

#[cfg(test)]
mod test {
  use super::*;
  use yaml_rust::YamlLoader;

  #[test]
  pub fn read_tasks_single_empty_task() {
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
    assert_eq!(read_tasks(yaml), Ok(expected));
  }

  #[test]
  pub fn read_tasks_single_command_task() {
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
    assert_eq!(read_tasks(yaml), Ok(expected));
  }

  #[test]
  pub fn read_tasks_single_command_depends_on_task() {
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
    assert_eq!(read_tasks(yaml), Ok(expected));
  }
}
