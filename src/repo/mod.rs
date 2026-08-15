use std::process::Command;

use color_eyre::{Result, eyre::ensure};

pub mod git;
pub mod repository;
pub mod service;

/// Runs a prepared `git` command with its stdout and stderr captured, so its output cannot
/// interleave with an active spinner. On a non-zero exit, the returned error is annotated with
/// the captured stderr; on success the output is discarded.
pub(crate) fn run_git_captured(mut command: Command, failure_context: &str) -> Result<()> {
    let output = command.output()?;
    ensure!(
        output.status.success(),
        "{failure_context}{}",
        format_git_stderr(&output.stderr)
    );
    Ok(())
}

/// Renders captured stderr as an error suffix: an empty capture contributes nothing, otherwise the
/// trimmed output is appended on its own line.
fn format_git_stderr(stderr: &[u8]) -> String {
    let text = String::from_utf8_lossy(stderr);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        String::new()
    } else {
        format!("\n{trimmed}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case(b"", "")]
    #[case(b"   \n  ", "")]
    #[case(b"fatal: not a git repository", "\nfatal: not a git repository")]
    #[case(b"  fatal: boom\n", "\nfatal: boom")]
    fn test_format_git_stderr(#[case] stderr: &[u8], #[case] expected: &str) {
        assert_eq!(format_git_stderr(stderr), expected);
    }
}
