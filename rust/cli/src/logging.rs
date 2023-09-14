use tracing_subscriber::prelude::*;

use common::{
    clap::{self, ValueEnum},
    eyre::Result,
    strum::AsRefStr,
    tracing::metadata::LevelFilter,
};

/// Setup logging
///
/// # Arguments
///
/// - `level`: The minimum log level for log entries emitted by Stencila
/// - `filter`: The filter to apply to log entries emitted by other crates
/// - `format`: The format to output log entries
#[cfg(not(feature = "console-subscriber"))]
pub fn setup(level: LoggingLevel, filter: &str, format: LoggingFormat) -> Result<()> {
    use common::eyre::{bail, Context};
    use is_terminal::IsTerminal;
    use tracing_error::ErrorLayer;
    use tracing_subscriber::{fmt, registry, EnvFilter};

    let format = match format {
        LoggingFormat::Auto => {
            if std::io::stderr().is_terminal() {
                if cfg!(debug_assertions) {
                    LoggingFormat::Pretty
                } else {
                    LoggingFormat::Simple
                }
            } else {
                LoggingFormat::Json
            }
        }
        _ => format,
    };

    let filter = format!(
        "{}{}{}",
        level.as_ref(),
        if filter.is_empty() { "" } else { "," },
        filter
    );
    let filter_layer = EnvFilter::builder()
        .parse(&filter)
        .wrap_err_with(|| format!("Unable to parse logging filter: {filter}"))?;
    let error_layer = ErrorLayer::default();
    let registry = registry().with(filter_layer).with(error_layer);

    match format {
        LoggingFormat::Simple => registry
            .with(
                fmt::layer()
                    .without_time()
                    .with_thread_ids(false)
                    .with_thread_names(false)
                    .with_target(false)
                    .compact(),
            )
            .init(),
        LoggingFormat::Compact => registry.with(fmt::layer().compact()).init(),
        LoggingFormat::Pretty => registry.with(fmt::layer().pretty()).init(),
        LoggingFormat::Full => registry.with(fmt::layer()).init(),
        LoggingFormat::Json => registry.with(fmt::layer().json()).init(),
        _ => bail!("Unhandled log format `{}`", format.as_ref()),
    };

    common::tracing::trace!("Logging set up finished");

    Ok(())
}

/// Setup logging with the `console-subscriber` crate
///
/// This sets up the `console-subscriber` crate (which requires TRACE level filtering
/// and lots and lots of `tokio` emitted trace entries) with stderr output filtered
/// to `level`. Usually you'll want to use at least the `debug` level to avoid clogging up
/// stderr with all the `tokio` trace entries. e.g.
///
/// ```sh
/// cargo run --bin cli --features=console-subscriber -- --log-level=debug ...
/// ```
#[cfg(feature = "console-subscriber")]
pub fn setup(level: LoggingLevel, _filter: &str, _format: LoggingFormat) -> Result<()> {
    let console_layer = console_subscriber::spawn();
    let format_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_filter(LevelFilter::from(level));
    tracing_subscriber::registry()
        .with(console_layer)
        .with(format_layer)
        .init();

    Ok(())
}

/// A `tracing` log level
#[derive(Debug, Copy, Clone, ValueEnum, AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum LoggingLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LoggingLevel> for LevelFilter {
    fn from(value: LoggingLevel) -> Self {
        use LoggingLevel::*;
        match value {
            Trace => LevelFilter::TRACE,
            Debug => LevelFilter::DEBUG,
            Info => LevelFilter::INFO,
            Warn => LevelFilter::WARN,
            Error => LevelFilter::ERROR,
        }
    }
}

/// A `tracing-subscriber` format
///
/// See https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/index.html#formatters
#[derive(Debug, Copy, Clone, ValueEnum, AsRefStr)]
pub enum LoggingFormat {
    Auto,
    Simple,
    Compact,
    Pretty,
    Full,
    Json,
}
