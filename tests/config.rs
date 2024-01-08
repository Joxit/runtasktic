use runtasktic::config::*;
use std::fs;

const SAMPLE_YAML: &str = "tests/resources/sample.yml";
const NOTIFICATION_YAML: &str = "tests/resources/notification.yml";
const CONCURRENCY_YAML: &str = "tests/resources/concurrency.yml";
const ON_FAILURE_YAML: &str = "tests/resources/on_failure.yml";

#[test]
fn sample_yaml() {
  let yaml = fs::read_to_string(SAMPLE_YAML).unwrap();
  let config = Config::from_str(yaml.as_str()).unwrap();

  let a = Task::new("a".to_string(), vec![echo("a"), sleep("0.5")], vec![], None);
  let b = Task::new("b", vec![echo("b"), sleep("0.5")], vs(&["a"]), None);
  let c = Task::new("c", vec![echo("c")], vs(&["a"]), None);
  let d = Task::new("d", vec![echo("d")], vs(&["b", "c"]), None);

  assert_eq!(*config.notification(), None);
  assert_eq!(config.concurrency(), -1);
  assert_eq!(
    *config.working_dir(),
    Some(String::from("/custom/directory"))
  );
  assert_eq!(*config.stdout(), Some(String::from("none")));
  assert_eq!(*config.stderr(), Some(String::from("none")));
  assert_eq!(config.tasks().len(), 4);
  assert_eq!(config.tasks().get(&"a".to_string()), Some(&a));
  assert_eq!(config.tasks().get(&"b".to_string()), Some(&b));
  assert_eq!(config.tasks().get(&"c".to_string()), Some(&c));
  assert_eq!(config.tasks().get(&"d".to_string()), Some(&d));
}

#[test]
fn concurrency_yaml() {
  let yaml = fs::read_to_string(CONCURRENCY_YAML).unwrap();
  let config = Config::from_str(yaml.as_str()).unwrap();

  let a1 = Task::new(
    "a1",
    vec![echo("Begin a1"), sleep("0.5"), echo("End a1")],
    vec![],
    None,
  );
  let a2 = Task::new(
    "a2",
    vec![echo("Begin a2"), sleep("1"), echo("End a2")],
    vec![],
    None,
  );
  let b = Task::new(
    "b",
    vec![echo("Begin b"), sleep("0.5"), echo("End b")],
    vs(&["a1", "a2"]),
    None,
  );
  let c = Task::new(
    "c",
    vec![echo("Begin c"), sleep("1"), echo("End c")],
    vs(&["a1"]),
    None,
  );
  let d = Task::new(
    "d",
    vec![echo("Begin d"), sleep("0.5"), echo("End d")],
    vs(&["a1"]),
    None,
  );
  let e = Task::new(
    "e",
    vec![echo("Begin e"), sleep("0.5"), echo("End e")],
    vs(&["b", "c", "d"]),
    None,
  );
  let f = Task::new(
    "f",
    vec![echo("Begin f"), sleep("1"), echo("End f")],
    vs(&["c"]),
    None,
  );

  assert_eq!(*config.notification(), None);
  assert_eq!(config.concurrency(), 2);
  assert_eq!(config.tasks().len(), 7);
  assert_eq!(config.tasks().get(&"a1".to_string()), Some(&a1));
  assert_eq!(config.tasks().get(&"a2".to_string()), Some(&a2));
  assert_eq!(config.tasks().get(&"b".to_string()), Some(&b));
  assert_eq!(config.tasks().get(&"c".to_string()), Some(&c));
  assert_eq!(config.tasks().get(&"d".to_string()), Some(&d));
  assert_eq!(config.tasks().get(&"e".to_string()), Some(&e));
  assert_eq!(config.tasks().get(&"f".to_string()), Some(&f));
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

  let slack = Slack::new(
    "https://hooks.slack.com/services/XXXXX/XXXXX/XXXXX".to_string(),
    "#channel".to_string(),
    Some("runtasktic".to_string()),
    Some(":rocket:".to_string()),
    None,
  );
  let print = Print::new("stderr".to_string(), None);
  let email = Mail::new(
    ("Sender Name".to_string(), "sender@example.com".to_string()),
    vec![(
      ("Receiver Name".to_string()),
      "receiver@example.com".to_string(),
    )],
    "Subject".to_string(),
    MailSMTP::new(
      "smtp.example.com".to_string(),
      1587,
      "sender@example.com".to_string(),
      "secret-password".to_string(),
      false,
    ),
    None,
  );
  let notification = Notification::new(
    Some(slack),
    Some(print),
    Some(email),
    WhenNotify::Always,
    Messages::default(),
  );

  assert_eq!(*config.notification(), Some(notification));
  assert_eq!(config.concurrency(), -1);
  assert_eq!(config.tasks().len(), 5);
  assert_eq!(config.tasks().get(&"a".to_string()), Some(&a));
  assert_eq!(config.tasks().get(&"b".to_string()), Some(&b));
  assert_eq!(config.tasks().get(&"c".to_string()), Some(&c));
  assert_eq!(config.tasks().get(&"d".to_string()), Some(&d));
  assert_eq!(config.tasks().get(&"e".to_string()), Some(&e));
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

  assert_eq!(config.on_failure(), &OnFailure::Continue);
  assert_eq!(config.notification(), &None);
  assert_eq!(config.concurrency(), 2);
  assert_eq!(config.tasks().len(), 5);
  assert_eq!(config.tasks().get(&"a".to_string()), Some(&a));
  assert_eq!(config.tasks().get(&"b".to_string()), Some(&b));
  assert_eq!(config.tasks().get(&"c".to_string()), Some(&c));
  assert_eq!(config.tasks().get(&"d".to_string()), Some(&d));
  assert_eq!(config.tasks().get(&"e".to_string()), Some(&e));
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
