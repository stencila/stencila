#![recursion_limit = "256"]

use std::{env::set_var, process::exit};

use cli_utils::message;
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

    if cli.no_color {
        set_var("NO_COLOR", "1");
    } else if cli.force_color {
        set_var("FORCE_COLOR", "1");
    }

    let (log_level, log_format, error_details) = if cli.debug {
        (
            LoggingLevel::Debug,
            LoggingFormat::Pretty,
            "all".to_string(),
        )
    } else if cli.trace {
        (
            LoggingLevel::Trace,
            LoggingFormat::Pretty,
            "all".to_string(),
        )
    } else {
        (cli.log_level, cli.log_format, cli.error_details.clone())
    };

    if matches!(cli.command, Command::Lsp) {
        lsp::run(log_level.into(), &cli.log_filter).await?
    } else {
        errors::setup(&error_details, cli.error_link)?;
        logging::setup(log_level, &cli.log_filter, log_format)?;
        ask::setup_cli(cli.assume_answer()).await?;

        let skip_upgrade = matches!(cli.command, Command::Upgrade(..));
        if !skip_upgrade {
            upgrade::check(false);
        }

        if let Err(error) = cli.run().await {
            if error_details == "none" || (error_details == "auto" && !cfg!(debug_assertions)) {
                message(&error.to_string(), Some("💥"));
                exit(1);
            } else {
                return Err(error);
            }
        }

        if !skip_upgrade {
            upgrade::notify();
        }
    }

    Ok(())
}
