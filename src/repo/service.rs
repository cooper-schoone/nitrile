use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use color_eyre::{
    Result,
    eyre::{ContextCompat, ensure},
};

const fn is_valid_scp_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~'
}

/// Attempts to match a source string to the scp-like url shape `[user@]host:path`.
///
/// Following git, a colon only separates host from path when no slash precedes
/// it, and a single-letter host (e.g. `C:\...`) is read as a Windows drive
/// prefix rather than a remote host.
fn match_scp_shape(source: &str) -> Result<()> {
    let (host_part, path) = source.split_once(':').context("scp url must contain :")?;
    ensure!(!path.is_empty(), "scp url must contain a path after :");

    let host = match host_part.split_once('@') {
        Some((user, host)) => {
            ensure!(
                !user.is_empty() && user.chars().all(is_valid_scp_char),
                "scp url user must be non-empty and valid"
            );
            host
        }
        None => host_part,
    };
    ensure!(
        !host.is_empty() && host.chars().all(is_valid_scp_char),
        "scp url host must be non-empty and valid"
    );

    let is_drive_prefix = !host_part.contains('@')
        && host.len() == 1
        && host.starts_with(|c: char| c.is_ascii_alphabetic());
    ensure!(
        !is_drive_prefix,
        "single-letter host is a drive prefix, not a remote"
    );
    Ok(())
}

/// Location of a repository template, either local or remote.
#[derive(Clone)]
pub enum TemplateSource {
    Local(PathBuf),
    Remote(String),
}

impl From<String> for TemplateSource {
    fn from(source: String) -> Self {
        if source.contains("://") || match_scp_shape(&source).is_ok() {
            Self::Remote(source)
        } else {
            Self::Local(PathBuf::from(source))
        }
    }
}

impl AsRef<OsStr> for TemplateSource {
    fn as_ref(&self) -> &OsStr {
        match self {
            Self::Local(path) => path.as_ref(),
            Self::Remote(url) => url.as_ref(),
        }
    }
}

pub trait RepositoryService {
    /// Initializes a new LaTeX project repository in the given directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the repository cannot be initialized.
    fn init_new(&self, location: &Path) -> Result<()>;

    /// Initializes a new LaTeX project repository in the given directory from a specified template.
    ///
    /// # Errors
    ///
    /// Returns an error if the repository cannot be initialized or if the specified template is not
    /// found.
    fn init_from_template(&self, location: &Path, template_source: TemplateSource) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("git@github.com:user/repo.git")]
    #[case("github.com:user/repo.git")]
    #[case("myserver:repo.git")]
    #[case("git.user_1@host-1.example.com:~/templates/base")]
    #[case("1:path")]
    fn test_scp_shapes_are_matched(#[case] input: &str) {
        assert!(match_scp_shape(input).is_ok());
    }

    #[rstest]
    #[case("C:\\Users\\me\\template")]
    #[case("D:/projects/template")]
    #[case("/home/me/template")]
    #[case("./weird:name")]
    #[case("plaintext")]
    #[case("github.com:")]
    #[case(":path")]
    #[case("@host:path")]
    fn test_non_scp_shapes_are_rejected(#[case] input: &str) {
        assert!(match_scp_shape(input).is_err());
    }

    #[rstest]
    #[case("https://github.com/user/repo.git")]
    #[case("git@github.com:user/repo.git")]
    #[case("myserver:repo.git")]
    fn test_remote_sources_are_classified(#[case] input: &str) {
        let source = TemplateSource::from(input.to_string());
        assert!(matches!(source, TemplateSource::Remote(s) if s == input));
    }

    #[rstest]
    #[case("/home/me/template")]
    #[case("./relative/path")]
    #[case("C:\\Users\\me\\template")]
    fn test_local_sources_are_classified(#[case] input: &str) {
        let source = TemplateSource::from(input.to_string());
        assert!(matches!(source, TemplateSource::Local(p) if p == *input));
    }
}
