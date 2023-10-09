use chrono::Local;
use cron::Schedule;

pub trait WaitSchedule<T> {
  fn wait(&self, timezone: T);
}

impl WaitSchedule<Local> for Option<Schedule> {
  fn wait(&self, timezone: Local) {
    if let Some(cron) = self {
      cron.wait(timezone);
    }
  }
}

impl WaitSchedule<Local> for Schedule {
  fn wait(&self, timezone: Local) {
    let date = self
      .upcoming(timezone)
      .next()
      .expect("Cannot get upcoming cron date");

    std::thread::sleep(
      (date - Local::now())
        .to_std()
        .expect("Cannot transform Duration"),
    );
  }
}
