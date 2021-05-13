use crate::config::Print;
use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;

pub fn notification_print(print: &Print, message: &str) -> Result<(), String> {
  writeln(
    print.output(),
    format!(
      "{} INFO [notification::print] {}",
      Local::now().format("%Y-%m-%d %H:%M:%S,%3f"),
      message
    ),
  )
  .map_err(|msg| format!("Can't open output file {}: {}", print.output(), msg))?;
  Ok(())
}

fn writeln(output: &String, message: String) -> std::io::Result<()> {
  match output.as_ref() {
    "stdout" => writeln!(std::io::stdout(), "{}", message),
    "stderr" => writeln!(std::io::stderr(), "{}", message),
    "none" | "/dev/null" => Ok(()),
    _ => {
      let file = OpenOptions::new().create(true).append(true).open(output)?;
      writeln!(&file, "{}", message)
    }
  }
}
