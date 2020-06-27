use crate::config::Task;
use std::collections::HashMap;
use yaml_rust::{Yaml, YamlLoader};

pub fn read_tasks(s: &str) -> HashMap<String, Task> {
  let yaml = YamlLoader::load_from_str(s).unwrap();
  let mut result = HashMap::new();

  if yaml.len() == 0 {
    return result;
  }

  println!("{:?}", yaml);
  let tasks = yaml[0]
    .as_hash()
    .unwrap()
    .get(&Yaml::String("tasks".to_string()))
    .unwrap()
    .as_hash()
    .unwrap();

  for (id, task) in tasks.iter() {
    let id = id.as_str().unwrap();
    let default = &linked_hash_map::LinkedHashMap::new();
    let task = task.as_hash().unwrap_or(default);
    let commands = task
      .get(&Yaml::String("commands".to_string()))
      .unwrap_or(&Yaml::Array(vec![]))
      .as_vec()
      .unwrap()
      .iter()
      .map(|c| c.as_str().unwrap().to_string())
      .collect();
    let depends_on = task
      .get(&Yaml::String("depends_on".to_string()))
      .unwrap_or(&Yaml::Array(vec![]))
      .as_vec()
      .unwrap()
      .iter()
      .map(|c| c.as_str().unwrap().to_string())
      .collect();

    result.insert(
      id.to_string(),
      Task::new(id.to_string(), commands, depends_on),
    );
  }

  result
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  pub fn read_tasks_empty_yaml() {
    assert_eq!(read_tasks(""), HashMap::new());
  }

  #[test]
  pub fn read_tasks_single_empty_task() {
    let yaml = "
    tasks:
      a:
    ";
    let mut expected: HashMap<String, Task> = HashMap::new();
    expected.insert(
      String::from("a"),
      Task::new(String::from("a"), vec![], vec![]),
    );
    assert_eq!(read_tasks(yaml), expected);
  }

  #[test]
  pub fn read_tasks_single_command_task() {
    let yaml = "
    tasks:
      a:
        commands:
        - echo OK
    ";
    let mut expected: HashMap<String, Task> = HashMap::new();
    expected.insert(
      String::from("a"),
      Task::new(String::from("a"), vec![String::from("echo OK")], vec![]),
    );
    assert_eq!(read_tasks(yaml), expected);
  }

  #[test]
  pub fn read_tasks_single_command_depends_on_task() {
    let yaml = "
    tasks:
      a:
        commands:
        - echo OK
        depends_on:
        - b
    ";
    let mut expected: HashMap<String, Task> = HashMap::new();
    expected.insert(
      String::from("a"),
      Task::new(String::from("a"), vec![String::from("echo OK")], vec![String::from("b")]),
    );
    assert_eq!(read_tasks(yaml), expected);
  }
}
