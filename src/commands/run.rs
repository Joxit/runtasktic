use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Run {}

impl Run {
  pub fn exec(&self) {}
}
