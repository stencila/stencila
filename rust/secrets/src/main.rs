use common::{clap::Parser, eyre::Result, tokio};

use secrets::cli::Cli;

/// A mini command line interface just for secrets
///
/// Intended for use during development only. Example usage:
///
/// ```sh
/// cargo run -p secrets list
/// ```
///
/// The `Cli` struct is integrated into the main `stencila` CLI
/// as the `secrets` subcommand e.g.
///
/// ```sh
/// stencila secrets list
/// ```
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.run().await
}
