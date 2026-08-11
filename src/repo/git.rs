use color_eyre::{Result, eyre::ensure};
use std::path::Path;
use std::process::Command;

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
        let status = Command::new("git")
            .current_dir(location)
            .arg("init")
            .status()?;
        ensure!(
            status.success(),
            "failed to initialize git repository in {}",
            location.to_string_lossy()
        );
        Ok(())
    }
}
