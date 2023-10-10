use crate::commands::Command;
use clap::{Args, CommandFactory, Parser};

mod commands;
mod config;
mod fst;
mod notification;
mod utils;

#[derive(Parser, Debug)]
#[command(name = "runtasktic", author, version, about)]
pub struct Runtasktic {
  #[command(subcommand)]
  pub command: Command,
}

impl Runtasktic {
  pub fn display_help(cmd: &str) {
    let clap = Self::augment_args(Self::command());
    let args = format!("{} {} --help", clap, cmd);
    clap.get_matches_from(args.split(" "));
  }
}

fn main() {
  let opt = Runtasktic::parse();

  opt.command.exec();
}
