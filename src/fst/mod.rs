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
  fn new() -> TaskFst {
    TaskFst {
      states: vec![],
      start_states: vec![],
    }
  }

  fn add_state(&mut self, label: String) {
    self.states.push(TaskFstState {
      label: label,
      id: self.states.len(),
      next: vec![],
      prev: vec![],
    })
  }

  fn add_edge(&mut self, from: usize, to: usize) {
    self.states[from].next.push(to);
    self.states[to].prev.push(from);
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
  pub fn read_tasks_empty_yaml() {
    let mut fst = TaskFst::new();
    fst.add_state("a".to_string());
    fst.add_state("b".to_string());
    fst.add_edge(0, 1);

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
}
