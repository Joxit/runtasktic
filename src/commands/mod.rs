use crate::commands::completion::Completion;
use crate::commands::dot::Dot;
use crate::commands::exec::Exec;
use crate::commands::run::Run;
use crate::commands::update::Update;
use anyhow::Result;
use clap::Parser;

mod completion;
mod dot;
mod exec;
mod run;
mod update;

#[derive(Parser, Debug)]
pub enum Command {
  /// Run all tasks from your configuration in background or foreground.
  ///
  /// Set the notification, messages, output files, concurency, working directory and many more options in your configuration.
  #[command(name = "run")]
  Run(Run),
  /// Export the configuration to a graph (needs graphviz/dot).
  #[command(name = "dot")]
  Dot(Dot),
  /// Execute a single command with notification in background or foreground.
  ///
  /// Inherit the notification from a configuration file and set your default one in your home: `~/.runtasktic.yml` or `~/.runtasktic.yaml`.
  #[command(name = "exec")]
  Exec(Exec),
  /// Generate completion script for your shell.
  #[command(name = "completion", subcommand)]
  Completion(Completion),
  /// Self update of the binary.
  #[command(name = "update")]
  Update(Update),
}

impl Command {
  pub fn exec(&self) -> Result<()> {
    match self {
      Command::Run(executable) => executable.exec(),
      Command::Exec(executable) => executable.exec(),
      Command::Dot(executable) => executable.exec(),
      Command::Completion(executable) => executable.exec(),
      Command::Update(executable) => executable.exec(),
    }
  }
}
