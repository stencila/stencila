use common::{
    clap::{self, Parser, Subcommand},
    eyre::{bail, Result},
    tokio, tracing,
};

mod errors;
mod logging;

use crate::logging::{LoggingFormat, LoggingLevel};

/// Main entry function
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    errors::setup(&cli.error_details, cli.error_link)?;
    logging::setup(cli.log_level, &cli.log_filter, cli.log_format)?;
    run(cli).await
}

/// CLI subcommands and global options
#[derive(Debug, Parser)]
#[command(name = "stencila", author, version, about, long_about)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// The minimum log level to output
    #[arg(long, default_value = "info", global = true)]
    log_level: LoggingLevel,

    /// The filter for log entries from crates other than Stencila
    ///
    /// Default of `error` allows for `ERROR` level log entries from all other
    /// crates. To additionally see lower level entries for a specific crates use
    /// syntax such as `error,tokio_postgres=debug`.
    #[arg(long, default_value = "error", global = true)]
    log_filter: String,

    /// The log format to use
    ///
    /// When `auto`, uses `simple` for terminals and `json`
    /// for non-TTY devices.
    #[arg(long, default_value = "auto", global = true)]
    log_format: LoggingFormat,

    /// The details to include in error reports
    ///
    /// A comma separated list including `location`, `span`, or `env`.
    #[arg(long, default_value = "auto", global = true)]
    error_details: String,

    /// Output a link to more easily report an issue
    #[arg(long, global = true)]
    error_link: bool,
}

#[derive(Debug, Subcommand)]
enum Command {}

/// Run the CLI command
///
/// This function mainly exists to have a top level, instrumented function
/// to call after error reporting and logging have been setup. This is
/// useful because then CLI arguments are captured in span traces.
#[tracing::instrument(skip(cli))]
async fn run(cli: Cli) -> Result<()> {
    bail!("No subcommands yet implemented");
}
