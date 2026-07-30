use std::{ffi::OsStr, path::Path, process::Command};

use color_eyre::{Result, eyre::ensure};

use crate::compilation::engine::LatexEngine;

/// Extracts the jobname argument from the specified output path.
fn jobname(output: &Path) -> Result<Option<&OsStr>> {
    if let Some(file) = output.file_stem() {
        let extension = output.extension();
        // pdflatex does not require this, but without this catch the project will silently compile
        // to a file with the wrong extension
        ensure!(
            extension == Some(OsStr::new("pdf")),
            "file must have extension .pdf, got .{}",
            extension.and_then(|e| e.to_str()).unwrap_or_default(),
        );
        Ok(Some(file))
    } else {
        Ok(None)
    }
}

/// Extracts the output directory from the specified output path.
fn output_directory(output: &Path) -> Option<&Path> {
    if let Some(p) = output.parent() {
        if p.as_os_str().is_empty() {
            None
        } else {
            Some(p)
        }
    } else {
        None
    }
}

/// Builds the `pdflatex` command from the given arguments and options.
fn build_command(target: &Path, output: Option<&Path>) -> Result<Command> {
    let mut command = Command::new("pdflatex");
    // Run non-interactively so a LaTeX error reports and exits instead of blocking on stdin
    command.arg("-interaction=nonstopmode");
    command.arg("-halt-on-error");

    let (jobname, output_directory) = match output {
        Some(path) => (jobname(path)?, output_directory(path)),
        None => (None, None),
    };
    if let Some(j) = jobname {
        command.arg("-jobname");
        command.arg(j);
    }
    if let Some(dir) = output_directory {
        command.arg("-output-directory");
        command.arg(dir);
    }
    command.arg(target);

    Ok(command)
}

pub struct PdflatexEngine;

impl LatexEngine for PdflatexEngine {
    fn compile(&self, target: &Path, output: Option<&Path>) -> Result<()> {
        ensure!(target.is_file(), "target path must be a file");
        let target_ext = target.extension();
        ensure!(
            target_ext == Some(OsStr::new("tex")),
            "expected target to have extension .tex, got .{}",
            match target_ext {
                Some(ext) => ext.to_str().unwrap_or_default(),
                None => "",
            }
        );
        let mut command = build_command(target, output)?;

        // pdflatex will not create the output directory itself
        if let Some(dir) = output.and_then(output_directory) {
            std::fs::create_dir_all(dir)?;
        }

        let exit_status = command.status()?;
        ensure!(
            exit_status.success(),
            "compilation failed {}",
            match exit_status.code() {
                Some(code) => format!("with exit code {code}"),
                None => "– terminated by signal".to_string(),
            }
        );
        Ok(())
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
        assert_eq!(jobname, Some(OsStr::new("output-file")));
        Ok(())
    }

    #[test]
    fn test_jobname_errs_on_non_pdf() {
        let path = Path::new("dir1/dir2/output-file.txt");
        assert!(jobname(path).is_err());
    }

    #[test]
    fn test_jobname_handles_top_level_file_correctly() -> Result<()> {
        let path = Path::new("output-file.pdf");
        assert_eq!(jobname(path)?, Some(OsStr::new("output-file")));
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
    #[case(
        Path::new("main.tex"),
        None,
        &["-interaction=nonstopmode", "-halt-on-error", "main.tex"],
    )]
    #[case(
        Path::new("main.tex"),
        Some(Path::new("output.pdf")),
        &["-interaction=nonstopmode", "-halt-on-error", "-jobname", "output", "main.tex"],
    )]
    #[case(
        Path::new("proj/main.tex"),
        Some(Path::new("proj/output.pdf")),
        &["-interaction=nonstopmode", "-halt-on-error", "-jobname", "output", "-output-directory", "proj", "proj/main.tex"],
    )]
    #[case(
        Path::new("nested/dir/main.tex"),
        Some(Path::new("nested/dir/output.pdf")),
        &["-interaction=nonstopmode", "-halt-on-error", "-jobname", "output", "-output-directory", "nested/dir", "nested/dir/main.tex"],
    )]
    fn test_command_builds_correctly(
        #[case] target: &Path,
        #[case] output: Option<&Path>,
        #[case] expected_args: &[&str],
    ) -> Result<()> {
        let command = build_command(target, output)?;
        let program = command.get_program();
        assert_eq!(program, OsStr::new("pdflatex"));
        let args: Vec<&OsStr> = command.get_args().collect();
        assert_eq!(args, expected_args);
        Ok(())
    }
}
