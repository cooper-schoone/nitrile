use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use clap::{ArgAction, ArgMatches, Command, arg, value_parser};
use color_eyre::Result;
use color_eyre::eyre::{ContextCompat, ensure};
use indicatif::ProgressBar;

use crate::commands::base::CliCommand;
use crate::commands::flags::{Flag, parse_key_val};
use crate::compilation::engine::{EngineArgs, LatexEngine};

pub struct CompileCommand {
    pub engine: Box<dyn LatexEngine>,
}

impl CompileCommand {
    fn default_output(target: &Path) -> Result<PathBuf> {
        let stem = target
            .file_stem()
            .context("could not determine output file name from target")?;
        Ok(Path::new("build").join(stem).with_extension("pdf"))
    }

    /// Parse the matched arguments into the arguments required by the LaTeX engine.
    fn parse_args(matches: &ArgMatches) -> Result<EngineArgs> {
        let target_arg: Option<&PathBuf> = matches.get_one("target");
        let target: &Path = match target_arg {
            Some(t) => {
                ensure!(
                    t.extension().is_some_and(|e| e == "tex"),
                    "target must be a .tex file",
                );
                t
            }
            None => [Path::new("main.tex"), Path::new("Main.tex")]
                .iter()
                .find(|p| p.exists())
                .context("no target path was provided and no main.tex file was found")?,
        };

        let output: PathBuf = match matches.get_one::<PathBuf>("output") {
            Some(p) => p.to_path_buf(),
            None => Self::default_output(target)?,
        };

        let flags: Vec<Flag> = matches
            .get_many::<String>("flag")
            .map_or(Ok(vec![]), |flags| {
                flags
                    .into_iter()
                    .map(|flag| parse_key_val(flag))
                    .collect::<Result<Vec<Flag>>>()
            })?;

        let verbose: bool = *matches.get_one::<bool>("verbose").unwrap_or(&false);
        Ok(EngineArgs {
            target: target.to_path_buf(),
            output,
            flags,
            verbose,
        })
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
                arg!(-o --output <FILE> "filepath ending in .pdf to which the compiled PDF will be saved (defaults to build/<target>.pdf)")
                    .value_parser(value_parser!(PathBuf)),
            )
            .arg(
                arg!(-f --flag <FLAG> "boolean or string flag to be passed to the compiler for conditional compilation or overrides").action(ArgAction::Append)
            )
            .arg(
                arg!(-v --verbose "show latex engine output during compilation")
            )
    }

    fn run(&self, matches: &ArgMatches) -> Result<()> {
        let args = Self::parse_args(matches)?;
        let spinner = ProgressBar::new_spinner().with_message("Compiling...");
        spinner.enable_steady_tick(Duration::from_millis(100));
        let start = Instant::now();
        let output_path = self.engine.compile(args)?;
        let elapsed = start.elapsed();
        let page_count: String =
            Self::get_page_count_text(&output_path).unwrap_or("unknown page count".to_string());
        spinner.finish_and_clear();
        println!(
            "\u{2705} Project compiled successfully to {} ({page_count}, {:.2} seconds)",
            output_path.display(),
            elapsed.as_secs_f32(),
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
        assert_eq!(args.output, Path::new("out.pdf"));
        assert_eq!(args.flags, vec![]);
        Ok(())
    }

    #[test]
    fn test_parse_args_defaults_output_when_absent() -> Result<()> {
        let matches = matches_from(["compile", "-t", "doc.tex"]);
        let args = CompileCommand::parse_args(&matches)?;
        assert_eq!(args.target, Path::new("doc.tex"));
        assert_eq!(args.output, Path::new("build/doc.pdf"));
        assert_eq!(args.flags, vec![]);
        Ok(())
    }

    #[test]
    fn test_parse_args_defaults_output_to_build_dir_ignoring_target_dir() -> Result<()> {
        let matches = matches_from(["compile", "-t", "src/doc.tex"]);
        let args = CompileCommand::parse_args(&matches)?;
        assert_eq!(args.target, Path::new("src/doc.tex"));
        assert_eq!(args.output, Path::new("build/doc.pdf"));
        Ok(())
    }

    #[test]
    fn test_parse_args_rejects_non_tex_target() {
        let matches = matches_from(["compile", "-t", "doc.pdf"]);
        assert!(CompileCommand::parse_args(&matches).is_err());
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
        assert_eq!(args.output, Path::new("build/doc.pdf"));
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
