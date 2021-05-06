use crate::config::OnFailure;

#[derive(Debug, PartialEq, Clone)]
pub struct Task {
  id: String,
  commands: Vec<String>,
  depends_on: Vec<String>,
  on_failure: Option<OnFailure>,
  state: usize,
}

impl Task {
  pub fn new(
    id: String,
    commands: Vec<String>,
    depends_on: Vec<String>,
    on_failure: Option<OnFailure>,
  ) -> Task {
    Task {
      id: id,
      commands: commands,
      depends_on: depends_on,
      on_failure: on_failure,
      state: 0,
    }
  }

  pub fn id(&self) -> &String {
    &self.id
  }
  pub fn commands(&self) -> &Vec<String> {
    &self.commands
  }
  pub fn full_command(&self) -> String {
    self.commands().join(" && ")
  }
  pub fn short_command(&self) -> String {
    let cmd = self.commands().first().unwrap_or(&format!(""));
    cmd.splitn(2, " ").next().unwrap().to_string()
  }
  pub fn depends_on(&self) -> &Vec<String> {
    &self.depends_on
  }
  pub fn on_failure(&self) -> &Option<OnFailure> {
    &self.on_failure
  }
  pub fn state(&self) -> usize {
    self.state
  }
  pub fn set_state(&mut self, state: usize) {
    self.state = state
  }
}
