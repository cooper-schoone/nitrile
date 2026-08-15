use color_eyre::Result;
use std::path::Path;
use std::process::Command;

use crate::repo::run_git_captured;

/// Turns a materialized project directory into a Git repository.
///
/// This is the only place a `.git` directory is created; [`RepositoryService`]
/// implementations leave a clean working tree for it to initialize.
///
/// [`RepositoryService`]: crate::repo::service::RepositoryService
pub trait GitService {
    /// Initializes a Git repository in the given directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the repository cannot be initialized.
    fn init(&self, location: &Path) -> Result<()>;
}

/// [`GitService`] that shells out to the `git` executable.
pub struct CliGitService;

impl GitService for CliGitService {
    fn init(&self, location: &Path) -> Result<()> {
        let mut command = Command::new("git");
        command.current_dir(location).arg("init").arg("--quiet");
        run_git_captured(
            command,
            &format!(
                "failed to initialize git repository in {}",
                location.to_string_lossy()
            ),
        )
    }
}
