use std::process::Command;

pub trait CommandConfig {
  fn working_dir(&mut self, dir: &Option<String>) -> &mut Self;
}

impl CommandConfig for Command {
  fn working_dir(&mut self, dir: &Option<String>) -> &mut Self {
    if let Some(d) = dir {
      self.current_dir(d)
    } else {
      self
    }
  }
}
