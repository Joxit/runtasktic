use regex::{Regex, RegexBuilder};

#[derive(Debug)]
pub struct Substitute {
  regex: Regex,
  substitution: String,
  global: bool,
}

impl Substitute {
  pub fn new<S: AsRef<str>>(input: S) -> anyhow::Result<Self> {
    let input = input.as_ref();
    if let Some(s) = input.get(0..1)
      && s == "s"
      && let Some(sep) = input.get(1..2)
      && let split = input.split(sep).collect::<Vec<&str>>()
      && split.len() == 4
    {
      let mut case_insensitive = false;
      let mut global = false;

      for char in split[3].chars() {
        if char == 'i' {
          case_insensitive = true;
        } else if char == 'g' {
          global = true;
        } else {
          bail!("{input} is not a valid regex: option {char} not unknown");
        }
      }
      let regex = RegexBuilder::new(split[1])
        .case_insensitive(case_insensitive)
        .build()?;

      Ok(Self {
        regex,
        substitution: split[2].to_string(),
        global,
      })

      // "s/regex/substitution/options"
    } else {
      bail!("{input} is not a regex")
    }
  }

  pub fn replace<S: AsRef<str>>(&self, input: S) -> String {
    if self.global {
      self
        .regex
        .replace_all(input.as_ref(), &self.substitution)
        .to_string()
    } else {
      self
        .regex
        .replace(input.as_ref(), &self.substitution)
        .to_string()
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn replace() -> anyhow::Result<()> {
    assert_eq!(
      Substitute::new(r"s/^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+/[REDACTED_IP]/")?.replace("127.0.0.0"),
      "[REDACTED_IP]"
    );
    assert_eq!(
      Substitute::new(r"s,[0-9]+,x,g")?.replace("127.0.0.0"),
      "x.x.x.x"
    );
    assert_eq!(Substitute::new(r"s/o/O/i")?.replace("ooOOoo"), "OoOOoo");
    assert_eq!(Substitute::new(r"s/o/O/ig")?.replace("ooOOoo"), "OOOOOO");
    assert_eq!(
      Substitute::new(r"s,https?://.*/(.*),$1,ig")?.replace("https://joxit.dev/random/path"),
      "path"
    );
    assert_eq!(
      Substitute::new(r"s/(a)(a)/$1\1/ig")?.replace("aaaa"),
      r"a\1a\1"
    );

    Ok(())
  }

  #[test]
  fn new() {
    assert!(Substitute::new("").is_err());
    assert!(Substitute::new("s/ok/").is_err());
    assert!(Substitute::new("s/ok/OK").is_err());
    assert!(Substitute::new("d/ok/").is_err());
  }
}
