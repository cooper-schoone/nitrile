use crate::repo::service::{RepositoryService, TemplateSource};
use crate::sys::file::copy_dir_recursive;
use color_eyre::{Result, eyre::ensure};
use std::fs;
use std::path::Path;
use std::process::Command;

const GITIGNORE_TEMPLATE: &str = include_str!("../../assets/gitignore_template.txt");
const NITRILE_STY: &str = include_str!("../../assets/nitrile.sty");
const MAIN_TEX: &str = include_str!("../../assets/main.tex");

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

/// Returns whether or not the specified directory contains a `.git` directory.
fn is_git_repo(location: &Path) -> bool {
    location.join(".git").exists()
}

/// Removes the `.git` directory from a materialized project, if present, so the
/// resulting tree carries no template history.
fn strip_git_dir(location: &Path) -> Result<()> {
    if is_git_repo(location) {
        fs::remove_dir_all(location.join(".git"))?;
    }
    Ok(())
}

/// Clone a template repository from a given source into the specified directory, using Git
/// purely as transport, and strip the resulting `.git` directory.
fn clone_and_strip(location: &Path, template_source: &TemplateSource) -> Result<()> {
    fs::create_dir_all(location)?;
    let clone_status = clone_command(location, template_source).status()?;
    ensure!(
        clone_status.success(),
        "failed to clone template repository from {}",
        template_source.as_ref().to_string_lossy()
    );
    strip_git_dir(location)
}

/// Materializes LaTeX projects into a directory from the default preset or a template.
///
/// This service does not initialize a Git repository in the directory; its only job is to
/// initialize the project directory, clone the template project (if any), and strip `.git`, if
/// present. Initialization as a Git repository is delegated to [`GitService`].
///
/// [`GitService`]: crate::repo::git::GitService
pub struct DefaultRepositoryService;

impl RepositoryService for DefaultRepositoryService {
    fn init_new(&self, location: &Path) -> Result<()> {
        fs::create_dir_all(location)?;
        fs::write(location.join("nitrile.sty"), NITRILE_STY)?;
        fs::write(location.join(".gitignore"), GITIGNORE_TEMPLATE)?;
        fs::write(location.join("main.tex"), MAIN_TEX)?;
        Ok(())
    }

    fn init_from_template(&self, location: &Path, template_source: TemplateSource) -> Result<()> {
        match &template_source {
            TemplateSource::Local(path) => {
                ensure!(path.is_dir(), "local template path must point to directory");
                copy_dir_recursive(path, location)?;
                strip_git_dir(location)?;
            }
            TemplateSource::Remote(_) => clone_and_strip(location, &template_source)?,
        }
        Ok(())
    }
}
