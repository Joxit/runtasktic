use chrono::{DateTime, Duration, Local, Utc};
use millisecond::{Millisecond, prelude::*, pretty::OutputFormat};
use std::fs::File;
use std::io::{Stderr, Write};

enum Output {
  File(File),
  Stderr(Stderr),
}

pub struct Eta {
  enabled: bool,
  output: Output,
  tasks: usize,
  state: usize,
  last_tick: DateTime<Utc>,
  start: DateTime<Utc>,
}

impl Eta {
  pub fn new(
    enabled: bool,
    background: bool,
    stderr: Option<String>,
    tasks: usize,
  ) -> anyhow::Result<Self> {
    let output = if background && enabled {
      Output::File(super::open_options(
        stderr.unwrap_or("runtasktic.err".into()),
      )?)
    } else {
      Output::Stderr(std::io::stderr())
    };
    Ok(Self {
      enabled,
      tasks,
      last_tick: Utc::now(),
      start: Utc::now(),
      output: output,
      state: 0,
    })
  }

  pub fn tick(&mut self) {
    if !self.enabled || self.last_tick + Duration::seconds(10) > Utc::now() {
      return;
    }
    let elapsed = (Utc::now() - self.start).num_milliseconds() as u64;
    let pretty_elapsed = Eta::pretty_millis(elapsed);
    let pretty_eta =
      Eta::pretty_millis((elapsed / self.state as u64 * self.tasks as u64).max(elapsed) - elapsed);
    let pretty_avg = Eta::pretty_millis(elapsed / self.state as u64);
    let msg = format!(
      "{} ETA: {pretty_eta} / Elapsed: {pretty_elapsed} / AVG: {pretty_avg} / Done: {}/{}",
      Local::now().format("%Y-%m-%dT%H:%M:%S.%3f"),
      self.state,
      self.tasks
    );

    let res = match &mut self.output {
      Output::File(f) => writeln!(f, "{msg}"),
      Output::Stderr(stderr) => writeln!(stderr, "{msg}"),
    };
    self.last_tick = Utc::now();
    if let Err(err) = res {
      error!("Could not write to output {err}");
    }
  }

  fn pretty_millis(millis: u64) -> String {
    Millisecond::from_millis(millis).pretty_with(MillisecondOption {
      format: OutputFormat::Short,
      seconds: SecondsOptions::Combine,
      ..Default::default()
    })
  }

  pub fn finished_task(&mut self) {
    self.state = self.state + 1;
  }

  pub fn end(&mut self) {
    let elapsed = (Utc::now() - self.start).num_milliseconds() as u64;
    let pretty_elapsed = Eta::pretty_millis(elapsed);
    let pretty_avg = Eta::pretty_millis(elapsed / self.tasks as u64);
    let msg = format!(
      "{0} ETA: 0s / Elapsed: {pretty_elapsed} / AVG: {pretty_avg} / Done: {1}/{1}",
      Local::now().format("%Y-%m-%dT%H:%M:%S.%3f"),
      self.tasks
    );
    let res = match &mut self.output {
      Output::File(f) => writeln!(f, "{msg}"),
      Output::Stderr(stderr) => writeln!(stderr, "{msg}"),
    };
    if let Err(err) = res {
      error!("Could not write to output {err}");
    }
  }
}
