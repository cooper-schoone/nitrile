use color_eyre::Result;

use nitrile::commands::{CliCommand, build_cli, compile::CompileCommand};
use nitrile::compilation::pdflatex::PdflatexEngine;

fn main() -> Result<()> {
    color_eyre::install()?;

    // Construct the CLI
    let commands: Vec<Box<dyn CliCommand>> = vec![Box::new(CompileCommand {
        engine: Box::new(PdflatexEngine {}),
    })];
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
