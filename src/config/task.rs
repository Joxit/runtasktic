#[derive(Debug, PartialEq)]
pub struct Task {
  id: String,
  commands: Vec<String>,
  depends_on: Vec<String>,
}

impl Task {
  pub fn new(id: String, commands: Vec<String>, depends_on: Vec<String>) -> Task {
    Task {
      id: id,
      commands: commands,
      depends_on: depends_on,
    }
  }
}
