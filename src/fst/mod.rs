pub mod dot;
mod iter;

use crate::fst::iter::*;

#[derive(Debug, PartialEq, Clone)]
pub struct TaskFst {
  states: Vec<TaskFstState>,
  start_states: Vec<usize>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TaskFstState {
  label: String,
  id: usize,
  next: Vec<usize>,
  prev: Vec<usize>,
}

impl TaskFst {
  pub fn new() -> TaskFst {
    TaskFst {
      states: vec![],
      start_states: vec![],
    }
  }

  pub fn len(&self) -> usize {
    self.states.len()
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

  pub fn get_state_from_id(&self, id: usize) -> &TaskFstState {
    &self.states[id]
  }

  pub fn is_cyclic(&self) -> bool {
    let visited: &mut Vec<usize> = &mut Vec::with_capacity(self.states.len());
    for s in &self.start_states {
      visited.push(*s);
      if self.is_cyclic_dfs(visited) {
        return true;
      }
      visited.pop();
    }
    false
  }

  fn is_cyclic_dfs(&self, visited: &mut Vec<usize>) -> bool {
    let cur = visited[visited.len() - 1];
    for s in &self.states[cur].next {
      if visited.contains(s) {
        return true;
      }
      visited.push(*s);
      if self.is_cyclic_dfs(visited) {
        return true;
      }
      visited.pop();
    }
    false
  }

  pub fn iter(&self) -> TaskIter {
    TaskIter::new(&self)
  }
}

impl TaskFstState {
  pub fn label(&self) -> &String {
    &self.label
  }

  pub fn id(&self) -> usize {
    self.id
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
  pub fn is_cyclic() {
    let mut fst = TaskFst::new();
    fst.add_state("a".to_string());
    fst.add_state("b".to_string());
    fst.add_arc(0, 1);
    fst.add_start_state(0);

    assert!(!fst.is_cyclic());

    fst.add_state("c".to_string());
    fst.add_arc(0, 2);
    assert!(!fst.is_cyclic());
    fst.add_arc(1, 2);
    assert!(!fst.is_cyclic());

    fst.add_state("d".to_string());
    fst.add_arc(2, 3);
    assert!(!fst.is_cyclic());

    fst.add_state("e".to_string());
    fst.add_start_state(4);
    assert!(!fst.is_cyclic());

    fst.add_arc(4, 3);
    assert!(!fst.is_cyclic());

    fst.add_arc(3, 1);
    assert!(fst.is_cyclic());
  }
}
