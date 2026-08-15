use std::{env::current_dir, path::PathBuf};

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
}
