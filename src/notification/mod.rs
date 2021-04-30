pub use crate::notification::slack::*;
use hostname::get as get_hostname;
mod slack;

const HOSTNAME_TEMPLATE: &str = "{{HOSTNAME}}";

pub fn replace_templates<S: AsRef<str>>(message: S) -> String {
  replace_hostname(message)
}

fn replace_hostname<S: AsRef<str>>(message: S) -> String {
  let hostname = if let Ok(hostname) = get_hostname() {
    hostname.into_string().unwrap()
  } else {
    format!("<Hostname Not Found>")
  };
  message.as_ref().replace(HOSTNAME_TEMPLATE, &hostname)
}

#[cfg(test)]
mod test {
  use super::*;

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
    assert_eq!(
      super::replace_hostname("{HOSTNAME}"),
      "{HOSTNAME}".to_string()
    );
  }
}
