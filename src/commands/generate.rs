use crate::config::{Config, OnFailure, Task};
use crate::utils::substitute::Substitute;
use anyhow::Result;
use clap::Parser;
use std::collections::HashMap;
use std::{fs::File, io, io::IsTerminal, path::PathBuf};

lazy_static! {
  static ref REPLACE_TOKEN_REGEX: regex::Regex = regex::Regex::new("\\{([0-9]+)\\}").unwrap();
}

#[derive(Parser, Debug)]
pub struct Generate {
  /// Base for the configuration, defaulted to global configuration
  #[arg(long = "base")]
  base: Option<PathBuf>,
  /// Concurrency to set in configuration file
  #[arg(long = "concurrency", short = 'c', default_value = "1")]
  concurrency: i64,
  /// On failure to set in configuration file
  #[arg(long = "on-failure", default_value = "continue")]
  on_failure: OnFailure,
  /// How to create task ids, defaulted to task index
  /// You can use sed-like substitution to generate the task id, captured group will use `$` instead of `\`, e.g. `s/(.*)/$1/`.
  /// You can choose your own separator, for example use | instead of / when you want to math URLs, e.g. `s|http://.*/(.*)|$1|`
  /// Sed options `i` and `g` are also available
  /// You can use simple substitution using {} to use the full input line or {1}, {2} to match parts (separated by space)
  #[arg(long = "id")]
  id: Option<String>,
  /// Where to store the new configuration, default on stdout
  #[arg(long = "output", short = 'o')]
  output: Option<PathBuf>,
  /// Command to add in your configuration file.
  /// You can use simple substitution using {} to use the full input line or {1}, {2} to match parts (separated by space)
  #[arg(trailing_var_arg = true)]
  command: Vec<String>,
}

impl Generate {
  pub fn exec(&self) -> Result<()> {
    let mut config = if let Some(path) = &self.base {
      Config::from_path(&path)?
    } else if let Some(path) = super::default_config_path() {
      Config::from_path(&path)?
    } else {
      Config::default()
    };
    config.tasks = HashMap::default();
    config.concurrency = self.concurrency;
    config.on_failure = self.on_failure;
    let substitute = self
      .id
      .clone()
      .map(|id| Substitute::new(&id).ok())
      .flatten();
    let mut idx = 0;
    loop {
      if std::io::stdin().is_terminal() {
        return Ok(());
      }
      let mut buffer = String::new();
      match std::io::stdin().read_line(&mut buffer) {
        Ok(0) => break,
        Ok(_) => {
          let id = self.id(&buffer, &substitute, idx);
          let cmd: Vec<String> = self
            .command
            .iter()
            .map(|s| Self::replace_token(s, &buffer))
            .collect();
          config
            .tasks
            .insert(id.clone(), Task::new(id, vec![cmd.join(" ")], vec![], None));
          idx = idx + 1;
        }
        Err(_) => break,
      }
    }
    if let Some(output) = &self.output {
      let mut file = File::create(&output)?;
      serde_yaml::to_writer(&mut file, &config)?;
    } else {
      serde_yaml::to_writer(io::stdout(), &config)?;
    }
    Ok(())
  }

  fn replace_token(token: &String, input: &String) -> String {
    if *token == "{}".to_string() {
      input.trim()
    } else if let Some(num) = REPLACE_TOKEN_REGEX.find(input) {
      let nth = input
        .split(" ")
        .nth(num.as_str().parse::<usize>().unwrap().max(1) - 1);
      nth.unwrap_or_default()
    } else {
      token
    }
    .to_string()
  }

  fn id(&self, input: &String, substitute: &Option<Substitute>, idx: i32) -> String {
    if let Some(substitute) = substitute
      && let token = substitute.replace(&input)
      && !token.is_empty()
    {
      token
    } else if let Some(id) = &self.id
      && let token = Self::replace_token(&id, input)
      && !token.is_empty()
    {
      token
    } else {
      idx.to_string()
    }
    .trim()
    .to_string()
  }
}
