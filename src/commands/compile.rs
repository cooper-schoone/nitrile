use std::path::{Path, PathBuf};
use std::time::Duration;

use clap::{ArgMatches, Command, arg, value_parser};
use color_eyre::Result;
use color_eyre::eyre::ContextCompat;
use indicatif::ProgressBar;

use crate::commands::base::CliCommand;
use crate::compilation::engine::LatexEngine;

pub struct CompileCommand {
    pub engine: Box<dyn LatexEngine>,
}

impl CliCommand for CompileCommand {
    fn build(&self) -> Command {
        Command::new("compile")
            .about("Compile a given LaTeX project")
            .arg(
                arg!(-t --target <FILE> "the target file to compile")
                    .value_parser(value_parser!(PathBuf)),
            )
            .arg(
                arg!(-o --output <FILE> "filepath ending in .pdf to which the compiled PDF will be saved")
                    .value_parser(value_parser!(PathBuf)),
            )
    }

    fn run(&self, matches: &ArgMatches) -> Result<()> {
        let target_arg: Option<&PathBuf> = matches.get_one("target");
        let target: &Path = match target_arg {
            Some(t) => t,
            None => [Path::new("main.tex"), Path::new("Main.tex")]
                .iter()
                .find(|p| p.exists())
                .context("no target path was provided and no main.tex file was found")?,
        };

        let output: Option<&PathBuf> = matches.get_one("output");
        let spinner = ProgressBar::new_spinner().with_message("Compiling...");
        spinner.enable_steady_tick(Duration::from_millis(100));
        let output_path = self.engine.compile(target, output.map(|p| p.as_path()))?;
        spinner.finish_and_clear();
        println!(
            "\u{2705} Project compiled successfully to {}",
            output_path.display()
        );
        Ok(())
    }
}
