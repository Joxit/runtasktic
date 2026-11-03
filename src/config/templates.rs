use chrono::{DateTime, TimeZone};
use hostname::get as get_hostname;
use regex::Regex;
use std::collections::HashMap;
use std::env::var as get_env;

const HOSTNAME_TEMPLATE: &str = "{hostname}";
const ENVIRONMENT_TEMPLATE: &str = "\\{env.(?P<key>[a-zA-Z0-9_]+)\\}";

pub fn replace<S: AsRef<str>>(message: S, env: &HashMap<String, String>) -> String {
  let local = chrono::Local::now();
  let utc = local.to_utc();
  let msg = replace_hostname(message);
  let msg = replace_environments(msg, &env);
  msg
    .replace("{date.utc.now}", &now(&utc, true))
    .replace("{date.utc.today}", &today(&utc))
    .replace("{date.utc.time}", &time(&utc, true))
    .replace("{date.local.now}", &now(&local, false))
    .replace("{date.local.today}", &today(&local))
    .replace("{date.local.time}", &time(&local, false))
}

fn now<Tz: TimeZone>(d: &DateTime<Tz>, is_utc: bool) -> String
where
  Tz::Offset: std::fmt::Display,
{
  if is_utc {
    d.format("%FT%T%.3fZ").to_string()
  } else {
    d.format("%FT%T%.3f").to_string()
  }
}

fn today<Tz: TimeZone>(d: &DateTime<Tz>) -> String
where
  Tz::Offset: std::fmt::Display,
{
  d.format("%F").to_string()
}

fn time<Tz: TimeZone>(d: &DateTime<Tz>, is_utc: bool) -> String
where
  Tz::Offset: std::fmt::Display,
{
  if is_utc {
    d.format("%T%.3fZ").to_string()
  } else {
    d.format("%T%.3f").to_string()
  }
}

fn replace_hostname<S: AsRef<str>>(message: S) -> String {
  let hostname = if let Ok(hostname) = get_hostname() {
    hostname.into_string().unwrap()
  } else {
    format!("<Hostname Not Found>")
  };
  message.as_ref().replace(HOSTNAME_TEMPLATE, &hostname)
}

fn replace_environments<S: AsRef<str>>(message: S, env: &HashMap<String, String>) -> String {
  let regex = Regex::new(ENVIRONMENT_TEMPLATE).unwrap();
  regex
    .captures_iter(message.as_ref())
    .map(|caps| caps["key"].to_string())
    .filter(|key| key.len() > 0)
    .fold(message.as_ref().to_string(), |m, key| {
      let value = if let Ok(value) = get_env(&key) {
        value
      } else if let Some(value) = env.get(&key) {
        value.to_string()
      } else {
        format!("")
      };

      m.replace(&format!("{{env.{}}}", key), &value)
    })
}

pub fn parse_env_file<S: AsRef<str>>(content: S) -> anyhow::Result<HashMap<String, String>> {
  let mut env = HashMap::default();
  for mut entry in content.as_ref().split("\n").map(|line| line.splitn(2, "=")) {
    if let Some(raw_key) = entry.next()
      && let Some(raw_value) = entry.next()
      && !raw_key.contains("#")
      && let yaml = format!("{raw_key}: {raw_value}")
      && let Some((key, value)) = serde_yaml::from_str::<HashMap<String, String>>(&yaml)?
        .iter()
        .next()
    {
      env.insert(key.to_string(), value.to_string());
    }
  }

  Ok(env)
}

#[cfg(test)]
mod test {
  use super::*;
  use chrono::FixedOffset;
  use regex::Regex;
  use std::env::{remove_var as remove_env, set_var as set_env};
  use std::str::FromStr;

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
    unsafe {
      set_env("RUNTASKTIK", "value for RUNTASKTIK environment");
      set_env("RUNTASKTIK_test", "RUNTASKTIK_test value");
      set_env("RUNTASKTIK_empty", "");
      set_env("RUNTASKTIK_with_number_0_1_2", "0 1 2 3 4");
      remove_env("RUNTASKTIK_undefined");
    }

    assert_eq!(
      super::replace_environments("Test for {env.RUNTASKTIK}", &Default::default()),
      "Test for value for RUNTASKTIK environment".to_string()
    );
    assert_eq!(
      super::replace_environments("{env.RUNTASKTIK_test}", &Default::default()),
      "RUNTASKTIK_test value".to_string()
    );
    assert_eq!(
      super::replace_environments("Test for {env.RUNTASKTIK_empty}", &Default::default()),
      "Test for ".to_string()
    );
    assert_eq!(
      super::replace_environments("{env.RUNTASKTIK_with_number_0_1_2}", &Default::default()),
      "0 1 2 3 4".to_string()
    );
    assert_eq!(
      super::replace_environments("{env.RUNTASKTIK_undefined}", &Default::default()),
      "".to_string()
    );
  }

  #[test]
  fn replace_templates() -> anyhow::Result<()> {
    let utc_now = super::replace("{date.utc.now}", &Default::default());
    assert!(
      Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$")?.is_match(&utc_now),
      "Fail to parse `date.utc.now` and got: {utc_now}"
    );

    let utc_today = super::replace("{date.utc.today}", &Default::default());
    assert!(
      Regex::new(r"^\d{4}-\d{2}-\d{2}$")?.is_match(&utc_today),
      "Fail to parse `date.utc.today` and got: {utc_today}",
    );

    let utc_time = super::replace("{date.utc.time}", &Default::default());
    assert!(
      Regex::new(r"^\d{2}:\d{2}:\d{2}\.\d{3}Z$")?.is_match(&utc_time),
      "Fail to parse `date.utc.time` and got: {utc_time}",
    );

    let local_now = super::replace("{date.local.now}", &Default::default());
    assert!(
      Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}$")?.is_match(&local_now),
      "Fail to parse `date.local.now` and got: {local_now}",
    );

    let local_today = super::replace("{date.local.today}", &Default::default());
    assert!(
      Regex::new(r"^\d{4}-\d{2}-\d{2}$")?.is_match(&local_today),
      "Fail to parse `date.local.today` and got: {local_today}",
    );

    let local_time = super::replace("{date.local.time}", &Default::default());
    assert!(
      Regex::new(r"^\d{2}:\d{2}:\d{2}\.\d{3}$")?.is_match(&local_time),
      "Fail to parse `date.local.time` and got: {local_time}",
    );

    Ok(())
  }

  #[test]
  fn date_time_format() {
    let date: DateTime<FixedOffset> = DateTime::from_str("2017-08-30 13:37:00+02:00").unwrap();
    let utc = date.to_utc();

    assert_eq!(super::now(&date, false).as_str(), "2017-08-30T13:37:00.000");
    assert_eq!(super::today(&date).as_str(), "2017-08-30");
    assert_eq!(super::time(&date, false).as_str(), "13:37:00.000");

    assert_eq!(super::now(&utc, true).as_str(), "2017-08-30T11:37:00.000Z");
    assert_eq!(super::today(&utc).as_str(), "2017-08-30");
    assert_eq!(super::time(&utc, true).as_str(), "11:37:00.000Z");
  }

  #[test]
  fn parse_env_file() -> anyhow::Result<()> {
    let content = r#"
ENV_1=value
ENV_2 = value with removed dangling spaces 
ENV_3 ='value with single quote' 
ENV_4 = "value with double quote"
ENV_5 = ' value with single quote and dangling spaces '
override=first value
override=second value
empty_1=
empty_2=''
empty_3=""
# comment=should not be set
 # comment2=should not be set
comment3=comment3 content # with comment
comment4#=test
ignore
    "#;
    let map = super::parse_env_file(content)?;
    assert_eq!(map.get("ENV_1"), Some(&"value".into()));
    assert_eq!(
      map.get("ENV_2"),
      Some(&"value with removed dangling spaces".into())
    );
    assert_eq!(map.get("ENV_3"), Some(&"value with single quote".into()));
    assert_eq!(map.get("ENV_4"), Some(&"value with double quote".into()));
    assert_eq!(
      map.get("ENV_5"),
      Some(&" value with single quote and dangling spaces ".into())
    );
    assert_eq!(map.get("override"), Some(&"second value".into()));
    assert_eq!(map.get("empty_1"), Some(&"".into()));
    assert_eq!(map.get("empty_2"), Some(&"".into()));
    assert_eq!(map.get("empty_3"), Some(&"".into()));
    assert_eq!(map.get("ignore"), None);
    assert_eq!(map.get("# comment"), None);
    assert_eq!(map.get("comment"), None);
    assert_eq!(map.get(" comment"), None);
    assert_eq!(map.get(" # comment2"), None);
    assert_eq!(map.get("# comment2"), None);
    assert_eq!(map.get(" comment2"), None);
    assert_eq!(map.get("comment2"), None);
    assert_eq!(map.get("comment3"), Some(&"comment3 content".into()));
    assert_eq!(map.get("comment4"), None);
    assert_eq!(map.get("comment4#"), None);
    assert_eq!(map.get(""), None);
    Ok(())
  }

  #[test]
  fn parse_env_file_err() -> anyhow::Result<()> {
    assert!(super::parse_env_file("single_quote='").is_err());
    assert!(super::parse_env_file("double_quote=\"").is_err());
    assert!(super::parse_env_file("=empty key").is_err());
    Ok(())
  }
}
