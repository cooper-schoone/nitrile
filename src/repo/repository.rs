use crate::repo::service::{RepositoryService, TemplateSource};
use crate::sys::file::copy_dir_recursive;
use color_eyre::{Result, eyre::ensure};
use std::ffi::OsStr;
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

/// Copy a local template's contents into `location`, skipping a top-level `.git` directory so
/// none of the template's history is carried into the new project.
///
/// Subdirectories are copied via [`copy_dir_recursive`], which also skips symlinks.
fn copy_template(source: &Path, location: &Path) -> Result<()> {
    fs::create_dir_all(location)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() || entry.file_name() == OsStr::new(".git") {
            continue;
        }
        let dest_path = location.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
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
/// materialize the project directory and, for a template, copy it in without any of the
/// template's own Git history. Initialization as a Git repository is delegated to [`GitService`].
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
                copy_template(path, location)?;
            }
            TemplateSource::Remote(_) => clone_and_strip(location, &template_source)?,
        }
        Ok(())
    }
}
