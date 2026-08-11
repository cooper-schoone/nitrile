use color_eyre::Result;

use nitrile::commands::{CliCommand, build_cli, compile::CompileCommand, init::InitCommand};
use nitrile::compilation::pdflatex::PdflatexEngine;
use nitrile::repo::git::CliGitService;
use nitrile::repo::repository::DefaultRepositoryService;
use nitrile::sys::environment::SystemEnvironment;

fn main() -> Result<()> {
    color_eyre::install()?;

    // Construct the CLI
    let commands: Vec<Box<dyn CliCommand>> = vec![
        Box::new(CompileCommand {
            engine: Box::new(PdflatexEngine {
                environment: SystemEnvironment,
            }),
        }),
        Box::new(InitCommand::new(
            DefaultRepositoryService,
            CliGitService,
            SystemEnvironment,
        )),
    ];
    let (cli, index) = build_cli(&commands);

    // Execute the matched subcommand
    if let Some((name, arg_matches)) = cli.get_matches().subcommand() {
        match index.get(name) {
            Some(&i) => commands[i].run(arg_matches)?,
            _ => unreachable!("a subcommand is required"),
        }
    }

    Ok(())
}
