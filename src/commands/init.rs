use std::path::Path;
use std::path::PathBuf;
use std::str::FromStr;

use clap::{ArgMatches, Command, arg};
use color_eyre::Result;
use color_eyre::eyre::{bail, ensure};

use crate::commands::base::CliCommand;
use crate::repo::git::GitService;
use crate::repo::service::RepositoryService;
use crate::repo::service::TemplateSource;
use crate::spinner;
use crate::sys::environment::Environment;

pub struct InitCommand<R: RepositoryService, G: GitService, E: Environment> {
    repo_service: R,
    git_service: G,
    environment: E,
}

impl<R: RepositoryService, G: GitService, E: Environment> InitCommand<R, G, E> {
    pub const fn new(repo_service: R, git_service: G, environment: E) -> Self {
        Self {
            repo_service,
            git_service,
            environment,
        }
    }

    /// Get the text informing the user of the directory in which the repository was initialized.
    ///
    /// # Errors
    /// Returns an error if the current directory could not be detected.
    fn get_output_dir_text(&self, dir: &Path) -> Result<String> {
        let current_dir = self.environment.current_dir()?;
        // Resolve the target against the current directory so that a relative `.`
        // (the default) and an explicit absolute cwd both compare equal to it
        if current_dir.join(dir) == current_dir {
            Ok("current working directory".to_string())
        } else {
            Ok(dir.to_string_lossy().to_string())
        }
    }
}

impl<R: RepositoryService, G: GitService, E: Environment> CliCommand for InitCommand<R, G, E> {
    fn build(&self) -> Command {
        Command::new("init")
            .about("Initialize a new LaTeX project")
            .arg(
                arg!([DIR] "directory to initialize the project in (defaults to current directory)")
                .value_parser(|input: &str| -> Result<PathBuf> {
                    let path = PathBuf::from_str(input)?;
                    ensure!(!path.is_file(), "path points to an existing file");
                    Ok(path)
                })
            )
            .arg(
                arg!(-T --template <DIR> "template project to use as base")
                .value_parser(|input: &str| -> Result<TemplateSource> {
                    Ok(TemplateSource::from(input.to_string()))
                })
            )
            .arg(arg!(--"no-git" "skip initializing project as a git repository"))
            .arg(arg!(-F --force "initialize even if the target directory is not empty, overwriting conflicting files"))
    }

    fn run(&self, matches: &ArgMatches) -> Result<()> {
        let dir = matches
            .get_one::<PathBuf>("DIR")
            .cloned()
            .unwrap_or_else(|| PathBuf::from("."));
        let template = matches.get_one::<TemplateSource>("template").cloned();
        let no_git = matches.get_flag("no-git");
        let force = matches.get_flag("force");
        let git_present = self.environment.is_on_path("git");

        // A remote template needs git as transport even when the result won't be versioned
        if matches!(template, Some(TemplateSource::Remote(_))) {
            ensure!(
                git_present,
                "remote templates require git, which was not found on PATH"
            );
        }

        // Fail fast rather than overwrite files in a directory the user is already using
        if !force && self.environment.dir_is_nonempty(&dir)? {
            bail!(
                "target directory ({}) is not empty; pass --force to initialize anyway",
                self.get_output_dir_text(&dir)?
            );
        }

        let init_result: Result<()> = spinner!("Initializing new repository...", {
            match template {
                Some(source) => self.repo_service.init_from_template(&dir, source)?,
                None => self.repo_service.init_new(&dir)?,
            }

            if !no_git && git_present {
                self.git_service.init(&dir)?;
            }
            Ok(())
        });
        init_result?;

        println!(
            "\u{2705} New repository initialized in {}",
            self.get_output_dir_text(&dir)?
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::path::Path;

    use rstest::rstest;

    #[derive(Default)]
    struct FakeRepositoryService {
        init_new_calls: RefCell<Vec<PathBuf>>,
        init_from_template_calls: RefCell<Vec<(PathBuf, TemplateSource)>>,
    }

    impl RepositoryService for FakeRepositoryService {
        fn init_new(&self, location: &Path) -> Result<()> {
            self.init_new_calls
                .borrow_mut()
                .push(location.to_path_buf());
            Ok(())
        }

        fn init_from_template(
            &self,
            location: &Path,
            template_source: TemplateSource,
        ) -> Result<()> {
            self.init_from_template_calls
                .borrow_mut()
                .push((location.to_path_buf(), template_source));
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeGitService {
        init_calls: RefCell<Vec<PathBuf>>,
    }

    impl GitService for FakeGitService {
        fn init(&self, location: &Path) -> Result<()> {
            self.init_calls.borrow_mut().push(location.to_path_buf());
            Ok(())
        }
    }

    struct FakeEnvironment {
        git_present: bool,
        dir_nonempty: bool,
    }

    impl Environment for FakeEnvironment {
        fn is_on_path(&self, exe: &str) -> bool {
            exe == "git" && self.git_present
        }

        fn current_dir(&self) -> Result<PathBuf> {
            Ok(PathBuf::from("/home/user/project"))
        }

        fn dir_is_nonempty(&self, _path: &Path) -> Result<bool> {
            Ok(self.dir_nonempty)
        }
    }

    fn command_with(
        git_present: bool,
    ) -> InitCommand<FakeRepositoryService, FakeGitService, FakeEnvironment> {
        command_with_dir(git_present, false)
    }

    fn command_with_dir(
        git_present: bool,
        dir_nonempty: bool,
    ) -> InitCommand<FakeRepositoryService, FakeGitService, FakeEnvironment> {
        InitCommand::new(
            FakeRepositoryService::default(),
            FakeGitService::default(),
            FakeEnvironment {
                git_present,
                dir_nonempty,
            },
        )
    }

    #[rstest]
    // template source, git present, --no-git → (init_from_template used, git init run)
    #[case(&["init", "proj"], true, false, true)]
    #[case(&["init", "--no-git", "proj"], true, false, false)]
    #[case(&["init", "-T", "/tmp/template", "proj"], true, true, true)]
    #[case(&["init", "-T", "/tmp/template", "proj"], false, true, false)]
    #[case(&["init", "-T", "/tmp/template", "--no-git", "proj"], true, true, false)]
    #[case(&["init", "-T", "git@github.com:u/r.git", "proj"], true, true, true)]
    #[case(&["init", "-T", "git@github.com:u/r.git", "--no-git", "proj"], true, true, false)]
    fn test_run_dispatches_services(
        #[case] args: &[&str],
        #[case] git_present: bool,
        #[case] uses_template: bool,
        #[case] runs_git_init: bool,
    ) -> Result<()> {
        let command = command_with(git_present);
        let matches = command.build().get_matches_from(args.iter().copied());

        command.run(&matches)?;

        let template_calls = command.repo_service.init_from_template_calls.borrow().len();
        let new_calls = command.repo_service.init_new_calls.borrow().len();
        if uses_template {
            assert_eq!((template_calls, new_calls), (1, 0));
        } else {
            assert_eq!((template_calls, new_calls), (0, 1));
        }
        assert_eq!(
            command.git_service.init_calls.borrow().len(),
            usize::from(runs_git_init)
        );
        Ok(())
    }

    #[test]
    fn test_run_fails_for_remote_template_without_git() {
        let command = command_with(false);
        let matches =
            command
                .build()
                .get_matches_from(["init", "-T", "git@github.com:u/r.git", "proj"]);

        assert!(command.run(&matches).is_err());
        assert!(
            command
                .repo_service
                .init_from_template_calls
                .borrow()
                .is_empty()
        );
        assert!(command.repo_service.init_new_calls.borrow().is_empty());
        assert!(command.git_service.init_calls.borrow().is_empty());
    }

    #[test]
    fn test_run_fails_for_nonempty_dir_without_force() {
        let command = command_with_dir(true, true);
        let matches = command.build().get_matches_from(["init", "proj"]);

        assert!(command.run(&matches).is_err());
        assert!(command.repo_service.init_new_calls.borrow().is_empty());
        assert!(
            command
                .repo_service
                .init_from_template_calls
                .borrow()
                .is_empty()
        );
        assert!(command.git_service.init_calls.borrow().is_empty());
    }

    #[rstest]
    #[case(&["init", "--force", "proj"])]
    #[case(&["init", "-F", "proj"])]
    fn test_run_proceeds_for_nonempty_dir_with_force(#[case] args: &[&str]) -> Result<()> {
        let command = command_with_dir(true, true);
        let matches = command.build().get_matches_from(args.iter().copied());

        command.run(&matches)?;

        assert_eq!(command.repo_service.init_new_calls.borrow().len(), 1);
        assert_eq!(command.git_service.init_calls.borrow().len(), 1);
        Ok(())
    }

    #[rstest]
    #[case(Path::new("."), "current working directory")]
    #[case(Path::new("/home/user/project"), "current working directory")]
    #[case(Path::new("./dir/"), "./dir/")]
    #[case(Path::new("/some/other/dir"), "/some/other/dir")]
    fn test_output_dir_text_is_correct(#[case] dir: &Path, #[case] expected: &str) -> Result<()> {
        let command = command_with(true);
        assert_eq!(command.get_output_dir_text(dir)?, expected.to_string());
        Ok(())
    }
}
