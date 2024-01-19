use kernel::common::{clap::Parser, eyre::Result, tokio};

use kernels::cli::Cli;

/// A mini command line interface just for kernels
/// 
/// Intended for use during development only. Example usage:
/// 
/// ```sh
/// cargo run -p kernels list
/// ```
/// 
/// The `Cli` struct is integrated into the main `stencila` CLI
/// as the `kernels` subcommand e.g.
/// 
/// ```sh
/// stencila kernels list
/// ```
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.run().await
}
