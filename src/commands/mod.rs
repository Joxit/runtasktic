use crate::commands::run::Run;
use crate::commands::dot::Dot;
use structopt::StructOpt;

mod run;
mod dot;

#[derive(Debug, StructOpt)]
pub enum Command {
  /// Run tasks.
  #[structopt(name = "run")]
  Run(Run),
  /// Export the configuration to a graph (needs graphviz/dot).
  #[structopt(name = "dot")]
  Dot(Dot),
}

impl Command {
  pub fn exec(&self) {
    match self {
      Command::Run(executable) => executable.exec(),
      Command::Dot(executable) => executable.exec(),
    }
  }
}
