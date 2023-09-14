use common::{clap::Parser, eyre::Result, tokio};

use cli::{errors, logging, Cli};

/// Main entry function
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    errors::setup(&cli.error_details, cli.error_link)?;
    logging::setup(cli.log_level, &cli.log_filter, cli.log_format)?;
    cli.run().await
}
