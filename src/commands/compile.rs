use clap::{ArgMatches, Command};
use color_eyre::Result;

use super::base::CliCommand;

pub struct CompileCommand;

impl CliCommand for CompileCommand {
    fn build(&self) -> Command {
        Command::new("compile").about("Compile a given LaTeX project")
    }

    fn run(&self, matches: &ArgMatches) -> Result<()> {
        dbg!(&matches);
        Ok(())
    }
}
