use super::OnFailure;

#[derive(Debug, PartialEq, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Task {
  #[serde(skip)]
  pub(super) id: String,
  #[serde(default)]
  pub commands: Vec<String>,
  #[serde(default)]
  pub depends_on: Vec<String>,
  #[serde(default)]
  pub on_failure: Option<OnFailure>,
  #[serde(default)]
  pub working_dir: Option<String>,
  #[serde(skip)]
  state: usize,
}

impl Task {
  pub fn new<S: Into<String>>(
    id: S,
    commands: Vec<String>,
    depends_on: Vec<String>,
    on_failure: Option<OnFailure>,
  ) -> Task {
    Task {
      id: id.into(),
      commands,
      depends_on,
      on_failure,
      working_dir: None,
      state: 0,
    }
  }

  pub fn id(&self) -> &String {
    &self.id
  }
  pub fn full_command(&self) -> String {
    self.commands.join(" && ")
  }
  pub fn short_command(&self) -> String {
    if self.commands.len() > 0 {
      let cmd = self.commands.first().unwrap();
      cmd.splitn(2, " ").next().unwrap().to_string()
    } else {
      format!("<unknonw short command>")
    }
  }
  pub fn label(&self) -> String {
    if self.id.len() > 0 {
      self.id().clone()
    } else {
      self.short_command()
    }
  }
  pub fn state(&self) -> usize {
    self.state
  }
  pub fn set_state(&mut self, state: usize) {
    self.state = state
  }
}
