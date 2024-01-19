use kernel::common::{
    clap::{self, Parser},
    eyre::Result,
};

/// A command line interface for kernels
///
/// Allows for listing and testing of kernels.
#[derive(Parser)]
pub struct Cli {}

impl Cli {
    // Run the CLI
    pub async fn run(self) -> Result<()> {
        Ok(())
    }
}
