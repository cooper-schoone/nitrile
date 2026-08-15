use std::collections::HashMap;

use clap::{ArgMatches, Command, command};
use color_eyre::Result;

/// Trait for CLI subcommands. Facilitates runtime dependency injection for commands which depend on
/// external services (ex. a LaTeX compiler).
pub trait CliCommand {
    /// Constructs the subcommand associated with this trait.
    fn build(&self) -> Command;

    /// Runs the action associated with this subcommand given the parsed CLI arguments.
    ///
    /// # Errors
    ///
    /// Returns an error if the command fails to execute.
    fn run(&self, matches: &ArgMatches) -> Result<()>;
}

/// Maps a subcommand name to its index in the `commands` slice.
type CommandIndex = HashMap<String, usize>;

/// Constructs the main CLI command and the mapping used to access command runners.
#[must_use]
pub fn build_cli(commands: &[Box<dyn CliCommand>]) -> (Command, CommandIndex) {
    let base = command!()
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true);
    let mut index: CommandIndex = HashMap::new();
    let assembled_cli = commands.iter().enumerate().fold(base, |cli, (i, cmd)| {
        let built = cmd.build();
        index.insert(built.get_name().to_string(), i);
        cli.subcommand(built)
    });
    (assembled_cli, index)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeCommand;

    impl CliCommand for FakeCommand {
        fn build(&self) -> Command {
            Command::new("dbg")
        }

        fn run(&self, _matches: &ArgMatches) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_cli_builds_correctly() {
        let commands: &[Box<dyn CliCommand>] = &[Box::new(FakeCommand {})];
        let (cli, index) = build_cli(commands);
        assert_eq!(index.len(), 1);
        let subcommands: Vec<&Command> = cli.get_subcommands().collect();
        assert_eq!(subcommands.len(), 1);
        assert_eq!(subcommands[0].get_name(), "dbg");
    }
}
