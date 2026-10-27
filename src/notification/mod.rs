pub use self::discord::*;
pub use self::mail::*;
pub use self::print::*;
pub use self::slack::*;
use chrono::{DateTime, TimeZone};
use hostname::get as get_hostname;
use regex::Regex;
use std::env::var as get_env;

mod discord;
mod mail;
mod print;
mod slack;

const HOSTNAME_TEMPLATE: &str = "{hostname}";
const ENVIRONMENT_TEMPLATE: &str = "\\{env.(?P<key>[a-zA-Z0-9_]+)\\}";

pub fn replace_templates<S: AsRef<str>>(message: S) -> String {
  let local = chrono::Local::now();
  let utc = local.to_utc();
  let msg = replace_hostname(message);
  let msg = replace_environments(msg);
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

  #[test]
  fn replace_templates() -> anyhow::Result<()> {
    let utc_now = super::replace_templates("{date.utc.now}");
    assert!(
      Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$")?.is_match(&utc_now),
      "Fail to parse `date.utc.now` and got: {utc_now}"
    );

    let utc_today = super::replace_templates("{date.utc.today}");
    assert!(
      Regex::new(r"^\d{4}-\d{2}-\d{2}$")?.is_match(&utc_today),
      "Fail to parse `date.utc.today` and got: {utc_today}",
    );

    let utc_time = super::replace_templates("{date.utc.time}");
    assert!(
      Regex::new(r"^\d{2}:\d{2}:\d{2}\.\d{3}Z$")?.is_match(&utc_time),
      "Fail to parse `date.utc.time` and got: {utc_time}",
    );

    let local_now = super::replace_templates("{date.local.now}");
    assert!(
      Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}$")?.is_match(&local_now),
      "Fail to parse `date.local.now` and got: {local_now}",
    );

    let local_today = super::replace_templates("{date.local.today}");
    assert!(
      Regex::new(r"^\d{4}-\d{2}-\d{2}$")?.is_match(&local_today),
      "Fail to parse `date.local.today` and got: {local_today}",
    );

    let local_time = super::replace_templates("{date.local.time}");
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
}
