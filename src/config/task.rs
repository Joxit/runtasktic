#[derive(Debug, PartialEq)]
pub struct Task {
  pub id: String,
  pub commands: Vec<String>,
  pub depends_on: Vec<String>,
  pub state: usize,
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
}
