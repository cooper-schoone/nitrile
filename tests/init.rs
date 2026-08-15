use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use nitrile::commands::CliCommand;
use nitrile::commands::init::InitCommand;
use nitrile::repo::git::CliGitService;
use nitrile::repo::repository::DefaultRepositoryService;
use nitrile::sys::environment::SystemEnvironment;

use color_eyre::Result;
use color_eyre::eyre::ensure;
use tempfile::TempDir;

/// Serializes tests that read or mutate the process-global current working directory, so the
/// `.`-defaulting test cannot change the directory out from under the relative-path test.
static CWD_LOCK: Mutex<()> = Mutex::new(());

fn cwd_lock() -> std::sync::MutexGuard<'static, ()> {
    // Recover a poisoned lock so that test errors don't cascade into other tests
    CWD_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Restores the original working directory on drop, so a panicking assertion cannot leave the
/// process in a temporary directory that later gets removed.
struct CwdGuard {
    original: PathBuf,
}

impl CwdGuard {
    fn change_to(dir: &Path) -> Result<Self> {
        let original = std::env::current_dir()?;
        std::env::set_current_dir(dir)?;
        Ok(Self { original })
    }
}

impl Drop for CwdGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.original);
    }
}

const fn real_command() -> InitCommand<DefaultRepositoryService, CliGitService, SystemEnvironment> {
    InitCommand::new(DefaultRepositoryService, CliGitService, SystemEnvironment)
}

fn run_init(args: &[&str]) -> Result<()> {
    let command = real_command();
    let matches = command.build().get_matches_from(args);
    command.run(&matches)
}

/// Name of a sentinel file planted inside the template's `.git` directory. A fresh `git init`
/// never reproduces this file, so its absence in the target proves the template history was
/// stripped rather than merely left in place.
const GIT_HISTORY_MARKER: &str = "TEMPLATE_HISTORY_MARKER";

/// Builds a local template directory holding a sentinel file and a `.git` directory, so tests
/// can assert both that template contents are copied and that template history is stripped.
fn make_local_template() -> Result<TempDir> {
    let template = tempfile::tempdir()?;
    fs::write(template.path().join("template-marker.tex"), "template body")?;
    fs::create_dir_all(template.path().join("chapters"))?;
    fs::write(
        template.path().join("chapters").join("intro.tex"),
        "nested body",
    )?;
    fs::create_dir_all(template.path().join(".git"))?;
    fs::write(
        template.path().join(".git").join("HEAD"),
        "ref: refs/heads/main\n",
    )?;
    fs::write(
        template.path().join(".git").join(GIT_HISTORY_MARKER),
        "stale template history",
    )?;
    Ok(template)
}

fn ensure_template_history_stripped(dir: &Path) -> Result<()> {
    ensure!(
        !dir.join(".git").join(GIT_HISTORY_MARKER).exists(),
        "template git history was not stripped"
    );
    Ok(())
}

fn ensure_default_assets(dir: &Path) -> Result<()> {
    ensure!(
        dir.join("nitrile.sty").is_file(),
        "nitrile.sty was not written"
    );
    ensure!(
        dir.join(".gitignore").is_file(),
        ".gitignore was not written"
    );
    ensure!(dir.join("main.tex").is_file(), "main.tex was not written");
    Ok(())
}

fn ensure_is_git_repo(dir: &Path) -> Result<()> {
    ensure!(
        dir.join(".git").join("HEAD").is_file(),
        ".git/HEAD was not created"
    );
    Ok(())
}

fn ensure_not_git_repo(dir: &Path) -> Result<()> {
    ensure!(!dir.join(".git").exists(), ".git unexpectedly exists");
    Ok(())
}

// 1. Validate behavior with no template and relative path provided from CWD

#[test_with::executable(git)]
#[test]
fn test_init_no_template_default_writes_assets_and_git() -> Result<()> {
    let target = tempfile::tempdir()?;

    let result: Result<()> = (|| {
        run_init(&["init", target.path().to_str().unwrap()])?;
        ensure_default_assets(target.path())?;
        ensure_is_git_repo(target.path())?;
        Ok(())
    })();

    target.close()?;
    result
}

#[test_with::executable(git)]
#[test]
fn test_init_no_template_no_git_writes_assets_without_git() -> Result<()> {
    let target = tempfile::tempdir()?;

    let result: Result<()> = (|| {
        run_init(&["init", "--no-git", target.path().to_str().unwrap()])?;
        ensure_default_assets(target.path())?;
        ensure_not_git_repo(target.path())?;
        Ok(())
    })();

    target.close()?;
    result
}

// 2. Validate behavior with template repository provided

#[test_with::executable(git)]
#[test]
fn test_init_local_template_no_git_copies_and_strips_git() -> Result<()> {
    let template = make_local_template()?;
    let target = tempfile::tempdir()?;

    let result: Result<()> = (|| {
        run_init(&[
            "init",
            "-T",
            template.path().to_str().unwrap(),
            "--no-git",
            target.path().to_str().unwrap(),
        ])?;
        ensure!(
            target.path().join("template-marker.tex").is_file(),
            "template contents were not copied"
        );
        ensure!(
            target.path().join("chapters").join("intro.tex").is_file(),
            "nested template contents were not copied"
        );
        // Default files should not be added to cloned template repos
        ensure!(
            !target.path().join("nitrile.sty").exists(),
            "default preset asset nitrile.sty should not exist for a templated init"
        );
        ensure!(
            !target.path().join("main.tex").exists(),
            "default preset asset main.tex should not exist for a templated init"
        );
        ensure_not_git_repo(target.path())?;
        Ok(())
    })();

    target.close()?;
    template.close()?;
    result
}

#[test_with::executable(git)]
#[test]
fn test_init_local_template_with_git_copies_and_reinitializes_git() -> Result<()> {
    let template = make_local_template()?;
    let target = tempfile::tempdir()?;

    let result: Result<()> = (|| {
        run_init(&[
            "init",
            "-T",
            template.path().to_str().unwrap(),
            target.path().to_str().unwrap(),
        ])?;
        ensure!(
            target.path().join("template-marker.tex").is_file(),
            "template contents were not copied"
        );
        // The stripped template `.git` should be replaced by a fresh repository
        ensure_is_git_repo(target.path())?;
        ensure_template_history_stripped(target.path())?;
        Ok(())
    })();

    target.close()?;
    template.close()?;
    result
}

// 3. Validate target directory behavior relative to CWD

#[test_with::executable(git)]
#[test]
fn test_init_relative_path_target() -> Result<()> {
    let _lock = cwd_lock();
    let target_dir = tempfile::tempdir_in("tests")?;

    let result: Result<()> = (|| {
        let target = target_dir.path().join("project");
        let relative = target.strip_prefix(std::env::current_dir()?)?;

        run_init(&["init", relative.to_str().unwrap()])?;
        ensure_default_assets(&target)?;
        ensure_is_git_repo(&target)?;
        Ok(())
    })();

    target_dir.close()?;
    result
}

#[test_with::executable(git)]
#[test]
fn test_init_current_working_directory_target() -> Result<()> {
    let _lock = cwd_lock();
    let target = tempfile::tempdir()?;

    let result: Result<()> = (|| {
        let _cwd = CwdGuard::change_to(target.path())?;
        // Target should default to "."
        run_init(&["init"])?;
        ensure_default_assets(target.path())?;
        ensure_is_git_repo(target.path())?;
        Ok(())
    })();

    target.close()?;
    result
}
