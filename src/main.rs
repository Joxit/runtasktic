use crate::commands::Command;
use structopt::StructOpt;

mod commands;
mod config;
mod fst;
mod notification;

#[derive(Debug, StructOpt)]
#[structopt(name = "task-scheduler", author, about)]
pub struct ApplicationArguments {
  #[structopt(subcommand)]
  pub command: Command,
}

fn main() {
  let opt = ApplicationArguments::from_args();

  opt.command.exec();
}
