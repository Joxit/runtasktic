#[derive(Debug, PartialEq, Clone)]
pub struct TaskFst {
  pub states: Vec<TaskFstState>,
  pub start_states: Vec<usize>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TaskFstState {
  pub label: String,
  pub id: usize,
  pub next: Vec<usize>,
  pub prev: Vec<usize>,
}

impl TaskFst {
  pub fn new() -> TaskFst {
    TaskFst {
      states: vec![],
      start_states: vec![],
    }
  }

  pub fn add_state(&mut self, label: String) -> usize {
    self.states.push(TaskFstState {
      label: label,
      id: self.states.len(),
      next: vec![],
      prev: vec![],
    });
    self.states.len() - 1
  }

  pub fn add_start_state(&mut self, id: usize) {
    self.start_states.push(id)
  }

  pub fn add_arc(&mut self, from: usize, to: usize) {
    self.states[from].next.push(to);
    self.states[to].prev.push(from);
  }

  pub fn get_state_from_label(&self, label: String) -> &TaskFstState {
    self.states.iter().find(|s| s.label == label).unwrap()
  }

  pub fn get_state_id_from_label(&self, label: String) -> usize {
    self.states.iter().position(|s| s.label == label).unwrap()
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  pub fn create_task_fst() {
    assert_eq!(
      TaskFst::new(),
      TaskFst {
        states: vec![],
        start_states: vec![]
      }
    );
  }

  #[test]
  pub fn add_arc() {
    let mut fst = TaskFst::new();
    fst.add_state("a".to_string());
    fst.add_state("b".to_string());
    fst.add_arc(0, 1);

    assert_eq!(
      fst,
      TaskFst {
        states: vec![
          TaskFstState {
            label: "a".to_string(),
            id: 0,
            next: vec![1],
            prev: vec![]
          },
          TaskFstState {
            label: "b".to_string(),
            id: 1,
            next: vec![],
            prev: vec![0]
          }
        ],
        start_states: vec![]
      }
    )
  }

  #[test]
  pub fn get_state_from_label() {
    let mut fst = TaskFst::new();
    fst.add_state("a".to_string());
    fst.add_state("b".to_string());
    fst.add_arc(0, 1);

    assert_eq!(
      fst.get_state_from_label("b".to_string()),
      &TaskFstState {
        label: "b".to_string(),
        id: 1,
        next: vec![],
        prev: vec![0]
      }
    )
  }

  #[test]
  pub fn get_state_id_from_label() {
    let mut fst = TaskFst::new();
    fst.add_state("a".to_string());
    fst.add_state("b".to_string());
    fst.add_arc(0, 1);

    assert_eq!(fst.get_state_id_from_label("b".to_string()), 1)
  }
}
