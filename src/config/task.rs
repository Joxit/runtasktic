#[derive(Debug, PartialEq, Clone)]
pub struct Task {
  id: String,
  commands: Vec<String>,
  depends_on: Vec<String>,
  state: usize,
}

impl Task {
  pub fn new(id: String, commands: Vec<String>, depends_on: Vec<String>) -> Task {
    Task {
      id: id,
      commands: commands,
      depends_on: depends_on,
      state: 0,
    }
  }

  pub fn id(&self) -> &String {
    &self.id
  }
  pub fn commands(&self) -> &Vec<String> {
    &self.commands
  }
  pub fn depends_on(&self) -> &Vec<String> {
    &self.depends_on
  }
  pub fn state(&self) -> usize {
    self.state
  }
  pub fn set_state(&mut self, state: usize) {
    self.state = state
  }
}
