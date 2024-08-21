use common::{clap::Parser, eyre::Result, tokio};

use cli::{errors, logging, upgrade, Cli, Command};

/// Main entry function
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    errors::setup(&cli.error_details, cli.error_link)?;
    logging::setup(cli.log_level, &cli.log_filter, cli.log_format)?;

    prompts::update_builtin();

    let skip_upgrade = matches!(cli.command, Command::Lsp);
    if !skip_upgrade {
        upgrade::check();
    }

    cli.run().await?;

    if !skip_upgrade {
        upgrade::notify();
    }

    Ok(())
}
