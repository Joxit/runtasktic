use anyhow::Result;
use chrono::{DateTime, Local};
use millisecond::{Millisecond, MillisecondFormatter, pretty::*};
use std::process::Child;

pub struct Process {
  child: Child,
  start: DateTime<Local>,
}

#[derive(Default)]
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

impl ExitStatus {
  pub fn elapsed_short(&self) -> String {
    Millisecond::from_millis(self.elapsed).pretty_with(MillisecondOption {
      format: OutputFormat::Short,
      seconds: SecondsOptions::Combine,
      ..Default::default()
    })
  }

  pub fn elapsed_long(&self) -> String {
    Millisecond::from_millis(self.elapsed).pretty_with(MillisecondOption {
      format: OutputFormat::Long,
      seconds: SecondsOptions::Combine,
      ..Default::default()
    })
  }

  pub fn elapsed_seconds(&self) -> String {
    format!("{}s", self.elapsed as f64 / 1000f64)
  }

  pub fn elapsed_milliseconds(&self) -> String {
    format!("{}ms", self.elapsed)
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn format_duration() {
    let exit_status = ExitStatus {
      elapsed: 70_200,
      ..Default::default()
    };
    assert_eq!(
      exit_status.elapsed_long(),
      "1 minute 10.2 seconds".to_string()
    );
    assert_eq!(exit_status.elapsed_short(), "1m 10.2s".to_string());
    assert_eq!(exit_status.elapsed_seconds(), "70.2s".to_string());
    assert_eq!(exit_status.elapsed_milliseconds(), "70200ms".to_string());
  }
}
