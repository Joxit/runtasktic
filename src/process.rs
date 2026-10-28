use anyhow::Result;
use chrono::{DateTime, Local};
use millisecond::{Millisecond, MillisecondFormatter, pretty::*};
use std::process::Child;

pub struct Process {
  child: Child,
  start: DateTime<Local>,
}

pub struct ExitStatus {
  pub elapsed: u64,
  pub status: i32,
  pub success: bool,
}

impl Process {
  pub fn new(child: Child) -> Self {
    Process {
      child,
      start: Local::now(),
    }
  }

  fn exit_status(&self, exit: &std::process::ExitStatus) -> ExitStatus {
    ExitStatus {
      elapsed: (Local::now() - self.start).num_milliseconds() as u64,
      status: exit.code().unwrap_or(-255),
      success: exit.success(),
    }
  }

  pub fn try_wait(&mut self) -> Result<Option<ExitStatus>> {
    match self.child.try_wait() {
      Ok(Some(exit)) => Ok(Some(self.exit_status(&exit))),
      Ok(None) => Ok(None),
      Err(err) => Err(err.into()),
    }
  }

  pub fn wait(&mut self) -> Result<ExitStatus> {
    match self.child.wait() {
      Ok(exit) => Ok(self.exit_status(&exit)),
      Err(err) => Err(err.into()),
    }
  }
}
