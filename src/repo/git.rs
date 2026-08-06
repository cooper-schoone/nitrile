use crate::environment::is_on_path;
use crate::repo::file::copy_dir_recursive;
use crate::repo::service::{RepositoryService, TemplateSource};
use color_eyre::{Result, eyre::ensure};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Create the `git clone` command used to initialize from a template repository.
fn clone_command(location: &Path, template_source: &TemplateSource) -> Command {
    let mut command = Command::new("git");
    command
        .current_dir(location)
        .arg("clone")
        .arg("--depth")
        .arg("1")
        .arg(template_source.as_ref())
        .arg(".");
    command
}

/// Returns whether or not the specified directory is a Git repository.
fn is_git_repo(location: &Path) -> bool {
    location.join(".git").exists()
}

/// Clone a Git repository from a given source to the specified directory and clear the resulting
/// .git directory.
fn clone_and_clear_git_repo(location: &Path, template_source: &TemplateSource) -> Result<()> {
    fs::create_dir_all(location)?;
    let mut command = clone_command(location, template_source);
    let clone_status = command.status()?;
    ensure!(
        clone_status.success(),
        "failed to clone template repository from {}",
        template_source.as_ref().to_string_lossy()
    );

    fs::remove_dir_all(location.join(".git"))?;
    Ok(())
}

/// Initialize a Git repository in the specified directory.
fn initialize_git_repo(location: &Path) -> Result<()> {
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

const GITIGNORE_TEMPLATE: &str = include_str!("../../assets/gitignore_template.txt");
const NITRILE_STY: &str = include_str!("../../assets/nitrile.sty");
const MAIN_TEX: &str = include_str!("../../assets/main.tex");

/// A Git-based repository service. Initializes new projects as Git repositories with a default
/// .gitignore file.
pub struct GitService {
    // Prevent instantiation without going through `try_new`
    _private: (),
}

impl GitService {
    /// Initialize a new `GitService` instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the Git executable is not present on PATH.
    pub fn try_new() -> Result<Self> {
        ensure!(is_on_path("git"), "git executable not found on PATH");
        Ok(Self { _private: () })
    }
}

impl RepositoryService for GitService {
    fn init_new(&self, location: &Path) -> Result<()> {
        fs::create_dir_all(location)?;
        fs::write(location.join("nitrile.sty"), NITRILE_STY)?;
        fs::write(location.join(".gitignore"), GITIGNORE_TEMPLATE)?;
        fs::write(location.join("main.tex"), MAIN_TEX)?;
        initialize_git_repo(location)?;
        Ok(())
    }

    fn init_from_template(&self, location: &Path, template_source: TemplateSource) -> Result<()> {
        // If the specified directory is a Git repository, use `git clone` and clear the resulting
        // .git directory
        // Otherwise, just copy the directory contents
        match &template_source {
            TemplateSource::Local(path) => {
                ensure!(path.is_dir(), "local template path must point to directory");
                if is_git_repo(path) {
                    clone_and_clear_git_repo(location, &template_source)?;
                } else {
                    copy_dir_recursive(path, location)?;
                }
            }
            TemplateSource::Remote(_) => clone_and_clear_git_repo(location, &template_source)?,
        }

        initialize_git_repo(location)?;

        Ok(())
    }
}
