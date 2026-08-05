use std::path::PathBuf;
use std::str::FromStr;

use clap::{ArgMatches, Command, arg};
use color_eyre::Result;
use color_eyre::eyre::ensure;

use crate::commands::base::CliCommand;
use crate::repo::service::TemplateSource;

pub struct InitCommand {}

impl CliCommand for InitCommand {
    fn build(&self) -> Command {
        Command::new("init")
            .about("Initialize a new LaTeX project")
            .arg(
                arg!([DIR] "directory to initialize the project in (defaults to current directory)")
                .value_parser(|input: &str| -> Result<PathBuf> {
                    let path = PathBuf::from_str(input)?;
                    ensure!(!path.is_file(), "path points to an existing file");
                    Ok(path)
                })
            )
            .arg(
                arg!(-T --template <DIR> "template project to use as base")
                .value_parser(|input: &str| -> Result<TemplateSource> {
                    Ok(TemplateSource::from(input.to_string()))
                })
            )
            .arg(arg!(--no-git "skip initializing project as a git repository"))
    }

    fn run(&self, _matches: &ArgMatches) -> Result<()> {
        todo!()
    }
}
