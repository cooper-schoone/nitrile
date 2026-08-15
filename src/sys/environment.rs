use std::{
    env::current_dir,
    fs::read_dir,
    path::{Path, PathBuf},
};

use color_eyre::Result;
use which::which;

/// Reads facts about the surrounding execution environment.
///
/// Injected into commands and engines so that PATH-dependent behaviour can be
/// exercised in tests without the real executables installed.
pub trait Environment {
    /// Returns whether the specified executable is present on PATH.
    fn is_on_path(&self, exe: &str) -> bool;

    /// Returns the current working directory.
    ///
    /// # Errors
    /// Returns an error if the current working directory does not exist or cannot be accessed.
    fn current_dir(&self) -> Result<PathBuf>;

    /// Returns whether the given directory exists and contains at least one entry.
    ///
    /// A path that does not exist is treated as empty.
    ///
    /// # Errors
    /// Returns an error if the path exists but its contents cannot be read.
    fn dir_is_nonempty(&self, path: &Path) -> Result<bool>;
}

/// [`Environment`] backed by the host system.
pub struct SystemEnvironment;

impl Environment for SystemEnvironment {
    fn is_on_path(&self, exe: &str) -> bool {
        which(exe).is_ok()
    }

    fn current_dir(&self) -> Result<PathBuf> {
        let dir = current_dir()?;
        Ok(dir)
    }

    fn dir_is_nonempty(&self, path: &Path) -> Result<bool> {
        if !path.exists() {
            return Ok(false);
        }
        Ok(read_dir(path)?.next().is_some())
    }
}
