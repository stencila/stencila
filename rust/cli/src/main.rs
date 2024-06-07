use common::{clap::Parser, eyre::Result, tokio};

use cli::{errors, logging, upgrade, Cli, Command};

/// Main entry function
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // For some commands avoid the normal logging & error reporting setup
    let run_only = matches!(cli.command, Command::Lsp);

    if !run_only {
        errors::setup(&cli.error_details, cli.error_link)?;
        logging::setup(cli.log_level, &cli.log_filter, cli.log_format)?;
        upgrade::check();
    }

    cli.run().await?;

    if !run_only {
        upgrade::notify();
    }

    Ok(())
}
