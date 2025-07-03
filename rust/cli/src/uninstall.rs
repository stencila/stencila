use cli_utils::confirm;
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
    #[allow(clippy::print_stderr)]
    pub fn run(self) -> Result<()> {
        if !confirm("Are you sure you want to uninstall Stencila CLI?")? {
            return Ok(());
        }

        uninstall()?;
        eprintln!("😢 Successfully uninstalled");

        Ok(())
    }
}
