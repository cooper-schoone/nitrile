use std::{
    ffi::{OsStr, OsString},
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::SystemTime,
};

use color_eyre::{Result, eyre::ContextCompat, eyre::ensure};

use crate::{
    attempt,
    compilation::engine::{EngineArgs, LatexEngine},
    compilation::flags::Flag,
    sys::environment::Environment,
};

/// Extracts the jobname (file stem) from a resolved output path.
fn jobname(output: &Path) -> Result<&OsStr> {
    output
        .file_stem()
        .context("could not determine jobname from output path")
}

/// Extracts the output directory from the specified output path.
fn output_directory(output: &Path) -> Option<&Path> {
    output.parent().filter(|&p| !p.as_os_str().is_empty())
}

/// Escapes reserved LaTeX characters (`% \ # & _ $ { } ~ ^`) so a value is safe both to tokenize on
/// the pdflatex command line and to typeset verbatim via `\flag`.
fn escape_latex_reserved(input: &str) -> String {
    input.chars().fold(String::new(), |mut s, c| {
        match c {
            '%' | '#' | '&' | '_' | '$' | '{' | '}' => {
                s.push('\\');
                s.push(c);
            }
            '\\' => s.push_str(r"\textbackslash{}"),
            '~' => s.push_str(r"\textasciitilde{}"),
            '^' => s.push_str(r"\textasciicircum{}"),
            _ => s.push(c),
        }
        s
    })
}

fn format_flag(flag: Flag) -> String {
    let (key, value): (String, String) = match flag {
        Flag::Boolean { key, value } => (key, value.to_string()),
        Flag::String { key, value } => (key, escape_latex_reserved(&value)),
    };
    format!(r"\expandafter\def\csname nitrile@arg@{key}\endcsname{{{value}}}")
}

/// Resolves the path to the PDF that `pdflatex` will produce for the given output path.
fn resolve_output_path(output: &Path) -> Result<PathBuf> {
    let extension = output.extension();
    // pdflatex does not require this, but without this catch the project will silently compile
    // to a file with the wrong extension
    ensure!(
        extension == Some(OsStr::new("pdf")),
        "file must have extension .pdf, got .{}",
        extension.and_then(|e| e.to_str()).unwrap_or_default(),
    );
    let stem = output
        .file_stem()
        .context("could not determine output file name from output path")?;
    let file = Path::new(stem).with_extension("pdf");
    Ok(match output_directory(output) {
        Some(dir) => dir.join(file),
        None => file,
    })
}

fn format_input_arg(target: &Path, flags: Vec<Flag>) -> OsString {
    let flag_prefix = flags.into_iter().map(format_flag).collect::<String>();
    let mut input_arg = OsString::from(flag_prefix);
    input_arg.push(r"\input{");
    input_arg.push(target.as_os_str());
    input_arg.push("}");
    input_arg
}

/// Builds the `pdflatex` command from the given arguments and options.
fn build_command(args: EngineArgs) -> Result<Command> {
    let mut command = Command::new("pdflatex");
    // Run non-interactively so a LaTeX error reports and exits instead of blocking on stdin
    command.arg("-interaction=nonstopmode");
    command.arg("-halt-on-error");

    let output_path = resolve_output_path(&args.output)?;
    let jobname = jobname(output_path.as_path())?;
    let output_directory = output_directory(output_path.as_path());

    command.arg("-jobname");
    command.arg(jobname);
    if let Some(dir) = output_directory {
        command.arg("-output-directory");
        command.arg(dir);
    }
    command.arg(format_input_arg(&args.target, args.flags));

    if !args.verbose {
        command.stdout(Stdio::null());
        command.stderr(Stdio::null());
    }

    Ok(command)
}

/// Returns the message to be displayed if the project fails to compile. Describes the reason for
/// the failure (signal termination or failure with an exit code) and whether or not a corresponding
/// log file was found.
///
/// This function checks two conditions to determine if a compilation log is present:
/// 1. Whether a log file with the same file stem as the output file exists in the same directory as
///    the output file
/// 2. If that log file exists, whether it was modified after the `pdflatex` command was called
fn get_compilation_fail_msg(
    code: Option<i32>,
    output: &Path,
    command_started: SystemTime,
) -> String {
    let mut msg = String::from("compilation failed ");
    msg.push_str(&code.map_or_else(
        || "– terminated by signal\n".to_string(),
        |c| format!("with exit code {c}\n"),
    ));
    let log_file = output.with_extension("log");
    let log_string: Result<String> = attempt!({
        ensure!(log_file.try_exists()?);
        let metadata = fs::metadata(&log_file)?;
        ensure!(metadata.modified()? >= command_started);
        Ok(format!(
            "see compilation log at {}",
            log_file.to_string_lossy()
        ))
    });
    msg.push_str(&log_string.unwrap_or_else(|_| "no compilation logs found".to_string()));
    msg
}

pub struct PdflatexEngine<E: Environment> {
    pub environment: E,
}

impl<E: Environment> LatexEngine for PdflatexEngine<E> {
    fn compile(&self, args: EngineArgs) -> Result<PathBuf> {
        ensure!(args.target.is_file(), "target path must be a file");
        ensure!(
            self.environment.is_on_path("pdflatex"),
            "pdflatex not found on PATH",
        );
        let target_ext = args.target.extension();
        ensure!(
            target_ext == Some(OsStr::new("tex")),
            "expected target to have extension .tex, got .{}",
            target_ext.map_or("", |ext| ext.to_str().unwrap_or_default())
        );
        let output_path = resolve_output_path(&args.output)?;

        // pdflatex will not create the output directory itself
        if let Some(dir) = output_directory(&output_path) {
            std::fs::create_dir_all(dir)?;
        }

        let mut command = build_command(args)?;

        let start_time = SystemTime::now();
        let exit_status = command.status()?;
        ensure!(
            exit_status.success(),
            get_compilation_fail_msg(exit_status.code(), &output_path, start_time),
        );
        Ok(output_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn test_jobname_extracts_stem_correctly() -> Result<()> {
        let path = Path::new("dir1/dir2/output-file.pdf");
        let jobname = jobname(path)?;
        assert_eq!(jobname, OsStr::new("output-file"));
        Ok(())
    }

    #[test]
    fn test_jobname_handles_top_level_file_correctly() -> Result<()> {
        let path = Path::new("output-file.pdf");
        assert_eq!(jobname(path)?, OsStr::new("output-file"));
        Ok(())
    }

    #[test]
    fn test_output_dir_extracts_dir_correctly() {
        let path = Path::new("dir1/dir2/output-file.pdf");
        assert_eq!(output_directory(path), Some(Path::new("dir1/dir2/")));
    }

    #[test]
    fn test_output_dir_handles_top_level_file_correctly() {
        let path = Path::new("output-file.pdf");
        assert_eq!(output_directory(path), None);
    }

    #[rstest]
    #[case(Path::new("out.pdf"), Path::new("out.pdf"))]
    #[case(Path::new("build/out.pdf"), Path::new("build/out.pdf"))]
    #[case(Path::new("dist/report.pdf"), Path::new("dist/report.pdf"))]
    fn test_resolve_output_path(#[case] output: &Path, #[case] expected: &Path) -> Result<()> {
        assert_eq!(resolve_output_path(output)?, expected);
        Ok(())
    }

    #[rstest]
    #[case(Path::new("out.txt"))]
    #[case(Path::new("build/out.txt"))]
    #[case(Path::new("out"))]
    fn test_resolve_output_path_errs_on_non_pdf(#[case] output: &Path) {
        assert!(resolve_output_path(output).is_err());
    }

    #[rstest]
    #[case(
        Path::new("main.tex"),
        Path::new("main.pdf"),
        &["-interaction=nonstopmode", "-halt-on-error", "-jobname", "main", r"\input{main.tex}"],
    )]
    #[case(
        Path::new("main.tex"),
        Path::new("output.pdf"),
        &["-interaction=nonstopmode", "-halt-on-error", "-jobname", "output", r"\input{main.tex}"],
    )]
    #[case(
        Path::new("proj/main.tex"),
        Path::new("proj/output.pdf"),
        &["-interaction=nonstopmode", "-halt-on-error", "-jobname", "output", "-output-directory", "proj", r"\input{proj/main.tex}"],
    )]
    #[case(
        Path::new("nested/dir/main.tex"),
        Path::new("nested/dir/output.pdf"),
        &["-interaction=nonstopmode", "-halt-on-error", "-jobname", "output", "-output-directory", "nested/dir", r"\input{nested/dir/main.tex}"],
    )]
    fn test_command_builds_correctly(
        #[case] target: &Path,
        #[case] output: &Path,
        #[case] expected_args: &[&str],
    ) -> Result<()> {
        let args = EngineArgs {
            target: target.to_path_buf(),
            output: output.to_path_buf(),
            flags: vec![],
            verbose: false,
        };
        let command = build_command(args)?;
        let program = command.get_program();
        assert_eq!(program, OsStr::new("pdflatex"));
        let args: Vec<&OsStr> = command.get_args().collect();
        assert_eq!(args, expected_args);
        Ok(())
    }

    #[test]
    fn test_format_input_arg_without_flags() {
        let input_arg = format_input_arg(Path::new("proj/main.tex"), vec![]);
        assert_eq!(input_arg, OsStr::new(r"\input{proj/main.tex}"));
    }

    #[test]
    fn test_format_input_arg_prepends_flags() {
        let flags = vec![
            Flag::Boolean {
                key: "draft".to_string(),
                value: true,
            },
            Flag::String {
                key: "name".to_string(),
                value: "value".to_string(),
            },
        ];
        let input_arg = format_input_arg(Path::new("main.tex"), flags);
        assert_eq!(
            input_arg,
            OsStr::new(concat!(
                r"\expandafter\def\csname nitrile@arg@draft\endcsname{true}",
                r"\expandafter\def\csname nitrile@arg@name\endcsname{value}",
                r"\input{main.tex}",
            )),
        );
    }

    #[test]
    fn test_command_includes_flags_in_input_arg() -> Result<()> {
        let args = EngineArgs {
            target: Path::new("main.tex").to_path_buf(),
            output: Path::new("main.pdf").to_path_buf(),
            flags: vec![Flag::Boolean {
                key: "draft".to_string(),
                value: true,
            }],
            verbose: false,
        };
        let command = build_command(args)?;
        let command_args: Vec<&OsStr> = command.get_args().collect();
        assert_eq!(
            command_args,
            &[
                OsStr::new("-interaction=nonstopmode"),
                OsStr::new("-halt-on-error"),
                OsStr::new("-jobname"),
                OsStr::new("main"),
                OsStr::new(
                    r"\expandafter\def\csname nitrile@arg@draft\endcsname{true}\input{main.tex}"
                ),
            ],
        );
        Ok(())
    }

    #[rstest]
    #[case(
        Flag::Boolean { key: "bool-flag".to_string(), value: true },
        r"\expandafter\def\csname nitrile@arg@bool-flag\endcsname{true}",
    )]
    #[case(
        Flag::Boolean { key: "bool-flag".to_string(), value: false },
        r"\expandafter\def\csname nitrile@arg@bool-flag\endcsname{false}",
    )]
    #[case(
        Flag::String { key: "string-flag".to_string(), value: "value".to_string() },
        r"\expandafter\def\csname nitrile@arg@string-flag\endcsname{value}",
    )]
    #[case(
        Flag::String { key: "string-flag".to_string(), value: "abc[]%\\#&_${}~^def".to_string() },
        concat!(
            r"\expandafter\def\csname nitrile@arg@string-flag\endcsname",
            r"{abc[]\%\textbackslash{}\#\&\_\$\{\}\textasciitilde{}\textasciicircum{}def}",
        ),
    )]
    fn test_format_flag_formats_correctly(#[case] flag: Flag, #[case] expected: &str) {
        let formatted = format_flag(flag);
        assert_eq!(formatted, expected.to_string());
    }
}
