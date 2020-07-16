use crate::fst::*;

#[derive(Debug, PartialEq, Clone)]
pub struct TaskIter {
  fst: TaskFst,
  states: Vec<TaskStatus>,
  next: Vec<usize>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TaskStatus {
  Todo,
  InProgress,
  Done,
}

impl TaskIter {
  pub fn new(fst: &TaskFst) -> TaskIter {
    let copy = fst.clone();
    TaskIter {
      fst: copy,
      states: fst.states.iter().map(|_| TaskStatus::Todo).collect(),
      next: fst.start_states.clone(),
    }
  }

  pub fn is_done(&self) -> bool {
    self.states.iter().all(|state| *state == TaskStatus::Done)
  }

  pub fn mark_done(&mut self, state: usize) {
    self.states[state] = TaskStatus::Done;
    for s in &self.fst.states[state].next {
      if self.states[*s] == TaskStatus::Todo // Not done or in progress
        && !self.next.contains(s) // not in the list
        && self.fst.states[*s] // all previous tasks done
          .prev
          .iter()
          .all(|p| self.states[*p] == TaskStatus::Done)
      {
        self.next.push(*s);
      }
    }
  }

  pub fn n_in_progress(&self) -> i64 {
    self
      .states
      .iter()
      .map(|s| if *s == TaskStatus::InProgress { 1 } else { 0 })
      .sum()
  }

  pub fn has_next(&self) -> bool {
    self.next.len() > 0
  }

  pub fn next(&mut self) -> Option<TaskFstState> {
    if self.next.len() > 0 {
      let pos = self.next.remove(0);
      self.states[pos] = TaskStatus::InProgress;
      Some(self.fst.states[pos].clone())
    } else {
      None
    }
  }
}
