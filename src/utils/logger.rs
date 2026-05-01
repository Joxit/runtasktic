use chrono::Local;
use log::{Level, Log, Metadata, Record, SetLoggerError};

struct Logger {
  level: Level,
  target: &'static str,
}

impl Log for Logger {
  fn enabled(&self, metadata: &Metadata) -> bool {
    metadata.level() <= self.level
  }

  fn log(&self, record: &Record) {
    if self.enabled(record.metadata()) {
      let target = if let Some(module) = record.module_path()
        && let Some(last) = module.split("::").last()
        && !self.target.ends_with(&format!("::{last}"))
      {
        format!("{}::{}", self.target, last)
      } else {
        self.target.to_string()
      };
      eprintln!(
        "{} {:<5} [{}] {}",
        Local::now().format("%Y-%m-%dT%H:%M:%S.%3f"),
        record.level().to_string(),
        target,
        record.args()
      );
    }
  }

  fn flush(&self) {}
}

pub fn init_with_level(level: Level, target: &'static str) -> Result<(), SetLoggerError> {
  let logger = Logger { level, target };
  log::set_boxed_logger(Box::new(logger))?;
  log::set_max_level(level.to_level_filter());
  Ok(())
}
