#![recursion_limit = "256"]

use common::{clap::Parser, eyre::Result, tokio};

use cli::{
    errors,
    logging::{self, LoggingFormat, LoggingLevel},
    upgrade, Cli, Command,
};

/// Main entry function
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let (log_level, log_format, error_details) = if cli.debug {
        (LoggingLevel::Debug, LoggingFormat::Pretty, "all")
    } else if cli.trace {
        (LoggingLevel::Trace, LoggingFormat::Pretty, "all")
    } else {
        (cli.log_level, cli.log_format, cli.error_details.as_str())
    };

    if matches!(cli.command, Command::Lsp) {
        lsp::run(log_level.into(), &cli.log_filter).await?
    } else {
        errors::setup(error_details, cli.error_link)?;
        logging::setup(log_level, &cli.log_filter, log_format)?;
        ask::setup_cli().await?;

        let skip_upgrade = matches!(cli.command, Command::Upgrade(..));
        if !skip_upgrade {
            upgrade::check(false);
        }

        cli.run().await?;

        if !skip_upgrade {
            upgrade::notify();
        }
    }

    Ok(())
}
