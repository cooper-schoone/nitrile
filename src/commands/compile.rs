use std::path::{Path, PathBuf};
use std::time::Duration;

use clap::{ArgAction, ArgMatches, Command, arg, value_parser};
use color_eyre::Result;
use color_eyre::eyre::ContextCompat;
use indicatif::ProgressBar;

use crate::commands::base::CliCommand;
use crate::commands::flags::{Flag, parse_key_val};
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

        let flags: Vec<Flag> = matches
            .get_many::<String>("flag")
            .map_or(Ok(vec![]), |flags| {
                flags
                    .into_iter()
                    .map(|flag| parse_key_val(flag))
                    .collect::<Result<Vec<Flag>>>()
            })?;
        Ok(EngineArgs {
            target,
            output,
            flags,
        })
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
            .arg(
                arg!(-f --flag <FLAG> "boolean or string flag to be passed to the compiler for conditional compilation or overrides").action(ArgAction::Append)
            )
    }

    fn run(&self, matches: &ArgMatches) -> Result<()> {
        let args = Self::parse_args(matches)?;
        let spinner = ProgressBar::new_spinner().with_message("Compiling...");
        spinner.enable_steady_tick(Duration::from_millis(100));
        let output_path = self.engine.compile(args)?;
        spinner.finish_and_clear();
        println!(
            "\u{2705} Project compiled successfully to {}",
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
        assert_eq!(args.flags, vec![]);
        Ok(())
    }

    #[test]
    fn test_parse_args_leaves_output_unset_when_absent() -> Result<()> {
        let matches = matches_from(["compile", "-t", "doc.tex"]);
        let args = CompileCommand::parse_args(&matches)?;
        assert_eq!(args.target, Path::new("doc.tex"));
        assert_eq!(args.output, None);
        assert_eq!(args.flags, vec![]);
        Ok(())
    }

    #[test]
    fn test_parse_args_resolves_flags_correctly() -> Result<()> {
        let matches = matches_from([
            "compile",
            "-f",
            "name=John Doe",
            "-f",
            "no-show-summary",
            "-t",
            "doc.tex",
        ]);
        let args = CompileCommand::parse_args(&matches)?;
        assert_eq!(args.target, Path::new("doc.tex"));
        assert_eq!(args.output, None);
        assert_eq!(
            args.flags,
            vec![
                Flag::String {
                    key: "name".to_string(),
                    value: "John Doe".to_string()
                },
                Flag::Boolean {
                    key: "show-summary".to_string(),
                    value: false
                },
            ]
        );
        Ok(())
    }
}
