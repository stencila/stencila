use common::{clap::Parser, eyre::Result, tokio};

use plugins::cli::Cli;

/// A mini command line interface just for plugins
///
/// Intended for use during development only. Example usage:
///
/// ```sh
/// cargo run -p plugins list
/// ```
///
/// The `Cli` struct is integrated into the main `stencila` CLI
/// as the `plugins` subcommand e.g.
///
/// ```sh
/// stencila plugins list
/// ```
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.run().await
}
