use crate::commands::Command;
use structopt::StructOpt;

mod commands;
mod config;
mod fst;
mod notification;
mod utils;

#[derive(Debug, StructOpt)]
#[structopt(name = "runtasktic", author, about)]
pub struct ApplicationArguments {
  #[structopt(subcommand)]
  pub command: Command,
}

fn main() {
  let opt = ApplicationArguments::from_args();

  opt.command.exec();
}
