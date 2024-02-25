use std::io::{self, Write};

use common::{
    clap::{self, Parser},
    eyre::Result,
};

/// Uninstall the Stencila CLI
pub fn uninstall() -> Result<()> {
    self_replace::self_delete()?;

    Ok(())
}

/// Uninstall this command line tool
#[derive(Debug, Parser)]
pub struct Cli {}

impl Cli {
    pub fn run(self) -> Result<()> {
        print!("Are you sure you want to uninstall Stencila CLI? (y/n): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            return Ok(());
        }

        uninstall()?;
        println!("😢 Successfully uninstalled");

        Ok(())
    }
}
