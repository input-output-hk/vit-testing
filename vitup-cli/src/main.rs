use structopt::StructOpt;
use crate::commands::VitupCliCommand;
use crate::error::Result;

pub fn main() -> Result<()> {
    VitupCliCommand::from_args().exec()
}
