pub use crate::notification::mail::*;
pub use crate::notification::print::*;
pub use crate::notification::slack::*;
use hostname::get as get_hostname;
use regex::Regex;
use std::env::var as get_env;

mod mail;
mod print;
mod slack;

const HOSTNAME_TEMPLATE: &str = "{hostname}";
const ENVIRONMENT_TEMPLATE: &str = "\\{env.(?P<key>[a-zA-Z0-9_]+)\\}";

pub fn replace_templates<S: AsRef<str>>(message: S) -> String {
  let msg = replace_hostname(message);
  let msg = replace_environments(msg);
  msg
}

fn replace_hostname<S: AsRef<str>>(message: S) -> String {
  let hostname = if let Ok(hostname) = get_hostname() {
    hostname.into_string().unwrap()
  } else {
    format!("<Hostname Not Found>")
  };
  message.as_ref().replace(HOSTNAME_TEMPLATE, &hostname)
}

fn replace_environments<S: AsRef<str>>(message: S) -> String {
  let regex = Regex::new(ENVIRONMENT_TEMPLATE).unwrap();
  regex
    .captures_iter(message.as_ref())
    .map(|caps| caps["key"].to_string())
    .filter(|key| key.len() > 0)
    .fold(message.as_ref().to_string(), |m, key| {
      let value = if let Ok(value) = get_env(&key) {
        value
      } else {
        format!("")
      };

      m.replace(&format!("{{env.{}}}", key), &value)
    })
}

#[cfg(test)]
mod test {
  use super::*;
  use std::env::{remove_var as remove_env, set_var as set_env};

  #[test]
  fn replace_hostname() {
    assert_eq!(
      super::replace_hostname(HOSTNAME_TEMPLATE).find(HOSTNAME_TEMPLATE),
      None
    );

    assert_eq!(
      super::replace_hostname(HOSTNAME_TEMPLATE.to_string()).find(HOSTNAME_TEMPLATE),
      None
    );

    assert_eq!(
      super::replace_hostname(format!(
        "{}: Check my hostname {0} in a long message",
        HOSTNAME_TEMPLATE
      ))
      .find(HOSTNAME_TEMPLATE),
      None
    );
    assert_eq!(super::replace_hostname("hostname"), "hostname".to_string());
  }

  #[test]
  fn replace_environments() {
    set_env("RUNTASKTIK", "value for RUNTASKTIK environment");
    set_env("RUNTASKTIK_test", "RUNTASKTIK_test value");
    set_env("RUNTASKTIK_empty", "");
    set_env("RUNTASKTIK_with_number_0_1_2", "0 1 2 3 4");
    remove_env("RUNTASKTIK_undefined");

    assert_eq!(
      super::replace_environments("Test for {env.RUNTASKTIK}"),
      "Test for value for RUNTASKTIK environment".to_string()
    );
    assert_eq!(
      super::replace_environments("{env.RUNTASKTIK_test}"),
      "RUNTASKTIK_test value".to_string()
    );
    assert_eq!(
      super::replace_environments("Test for {env.RUNTASKTIK_empty}"),
      "Test for ".to_string()
    );
    assert_eq!(
      super::replace_environments("{env.RUNTASKTIK_with_number_0_1_2}"),
      "0 1 2 3 4".to_string()
    );
    assert_eq!(
      super::replace_environments("{env.RUNTASKTIK_undefined}"),
      "".to_string()
    );
  }
}
