use std::path::{Path, PathBuf};
use std::time::Duration;

use clap::{ArgMatches, Command, arg, value_parser};
use color_eyre::Result;
use color_eyre::eyre::ContextCompat;
use indicatif::ProgressBar;

use crate::commands::base::CliCommand;
use crate::compilation::engine::{EngineArgs, LatexEngine};

pub struct CompileCommand {
    pub engine: Box<dyn LatexEngine>,
}

impl CompileCommand {
    /// Parse the matched arguments into the arguments required by the LaTeX engine.
    fn parse_args(matches: &ArgMatches) -> Result<EngineArgs<'_>> {
        let target_arg: Option<&PathBuf> = matches.get_one("target");
        let target: &Path = match target_arg {
            Some(t) => t,
            None => [Path::new("main.tex"), Path::new("Main.tex")]
                .iter()
                .find(|p| p.exists())
                .context("no target path was provided and no main.tex file was found")?,
        };

        let output: Option<&Path> = matches.get_one::<PathBuf>("output").map(|p| p.as_path());
        Ok(EngineArgs { target, output })
    }

    fn get_page_count_text(output: &Path) -> Result<String> {
        let page_count = lopdf::Document::load_metadata(output)?.page_count;
        match page_count {
            1 => Ok("1 page".to_string()),
            n => Ok(format!("{n} pages")),
        }
    }
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
        let args = Self::parse_args(matches)?;
        let spinner = ProgressBar::new_spinner().with_message("Compiling...");
        spinner.enable_steady_tick(Duration::from_millis(100));
        let output_path = self.engine.compile(args)?;
        let page_count: String =
            Self::get_page_count_text(&output_path).unwrap_or("unknown page count".to_string());
        spinner.finish_and_clear();
        println!(
            "\u{2705} Project compiled successfully to {} ({page_count})",
            output_path.display()
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeEngine;

    impl LatexEngine for FakeEngine {
        fn compile(&self, _args: EngineArgs) -> Result<PathBuf> {
            Ok(PathBuf::new())
        }
    }

    fn matches_from<const N: usize>(args: [&str; N]) -> ArgMatches {
        let command = CompileCommand {
            engine: Box::new(FakeEngine),
        };
        command.build().get_matches_from(args)
    }

    #[test]
    fn test_parse_args_resolves_target_and_output() -> Result<()> {
        let matches = matches_from(["compile", "-t", "doc.tex", "-o", "out.pdf"]);
        let args = CompileCommand::parse_args(&matches)?;
        assert_eq!(args.target, Path::new("doc.tex"));
        assert_eq!(args.output, Some(Path::new("out.pdf")));
        Ok(())
    }

    #[test]
    fn test_parse_args_leaves_output_unset_when_absent() -> Result<()> {
        let matches = matches_from(["compile", "-t", "doc.tex"]);
        let args = CompileCommand::parse_args(&matches)?;
        assert_eq!(args.target, Path::new("doc.tex"));
        assert_eq!(args.output, None);
        Ok(())
    }
}
