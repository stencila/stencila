use ask::{ask_with_default, Answer};
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
    pub async fn run(self) -> Result<()> {
        if !ask_with_default(
            "Are you sure you want to uninstall Stencila CLI?",
            Answer::Yes,
        )
        .await?
        .is_yes()
        {
            return Ok(());
        }

        uninstall()?;
        eprintln!("😢 Successfully uninstalled");

        Ok(())
    }
}
