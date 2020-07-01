use crate::commands::run::Run;
use structopt::StructOpt;

mod run;

#[derive(Debug, StructOpt)]
pub enum Command {
  /// Run tasks.
  #[structopt(name = "run")]
  Run(Run),
}

impl Command {
  pub fn exec(&self) {
    match self {
      Command::Run(executable) => executable.exec(),
    }
  }
}
