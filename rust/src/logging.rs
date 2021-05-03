use crate::pubsub::publish;
use defaults::Defaults;
use eyre::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;
use strum::ToString;
use tracing::Event;
use validator::Validate;

/// # Logging level
#[derive(
    Debug, PartialEq, PartialOrd, Clone, Copy, JsonSchema, Deserialize, Serialize, ToString,
)]
#[serde(rename_all = "lowercase")]
pub enum LoggingLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Never = 9,
}

impl LoggingLevel {
    pub fn as_tracing_level(&self) -> tracing::Level {
        match self {
            Self::Trace => tracing::Level::TRACE,
            Self::Debug => tracing::Level::DEBUG,
            Self::Info => tracing::Level::INFO,
            Self::Warn => tracing::Level::WARN,
            Self::Error => tracing::Level::ERROR,
            Self::Never => tracing::Level::ERROR,
        }
    }
}

/// # Logging format
#[derive(Debug, PartialEq, Clone, Copy, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LoggingFormat {
    Plain,
    Pretty,
    Json,
}

#[cfg(feature = "config")]
pub mod config {
    use super::*;
    use crate::util::dirs;

    /// # Logging to standard error stream
    ///
    /// Configuration settings for log entries printed to stderr when using the CLI
    #[derive(
        Debug, Defaults, PartialEq, Clone, Copy, JsonSchema, Deserialize, Serialize, Validate,
    )]
    pub struct LoggingStdErrConfig {
        /// The maximum log level to emit
        #[def = "LoggingLevel::Info"]
        pub level: LoggingLevel,

        /// The format for the logs entries
        #[def = "LoggingFormat::Pretty"]
        pub format: LoggingFormat,
    }

    /// # Logging to desktop notifications
    ///
    /// Configuration settings for log entries shown to the user in the desktop
    #[derive(
        Debug, Defaults, PartialEq, Clone, Copy, JsonSchema, Deserialize, Serialize, Validate,
    )]
    pub struct LoggingDesktopConfig {
        /// The maximum log level to emit
        #[def = "LoggingLevel::Info"]
        pub level: LoggingLevel,
    }

    /// # Logging to file
    ///
    /// Configuration settings for logs entries written to file
    #[derive(Debug, Defaults, PartialEq, Clone, JsonSchema, Deserialize, Serialize, Validate)]
    pub struct LoggingFileConfig {
        /// The path of the log file
        #[def = "default_file_path()"]
        pub path: String,

        /// The maximum log level to emit
        #[def = "LoggingLevel::Info"]
        pub level: LoggingLevel,
    }

    /// Get the default value for `logging.file.path`
    pub fn default_file_path() -> String {
        dirs::logs(true)
            .expect("Unable to get logs directory")
            .join("log.json")
            .into_os_string()
            .into_string()
            .expect("Unable to convert path to string")
    }

    /// # Logging
    ///
    /// Configuration settings for logging
    #[derive(Debug, Default, PartialEq, Clone, JsonSchema, Deserialize, Serialize, Validate)]
    pub struct LoggingConfig {
        pub stderr: LoggingStdErrConfig,
        pub desktop: LoggingDesktopConfig,
        pub file: LoggingFileConfig,
    }
}

/// Create a preliminary logging subscriber.
///
/// This can be necessary to ensure that any log events that get emitted during
/// initialization are displayed to the user.
pub fn prelim() -> tracing::subscriber::DefaultGuard {
    let subscriber = tracing_subscriber::fmt()
        .pretty()
        .with_max_level(tracing::Level::INFO)
        .with_writer(std::io::stderr)
        .finish();
    tracing::subscriber::set_default(subscriber)
}

/// Custom tracing_subscriber layer that publishes events
/// under the pubsub "logging" topic as a JSON value.
struct PubSubLayer {
    level: LoggingLevel,
}

impl<S: tracing::subscriber::Subscriber> tracing_subscriber::layer::Layer<S> for PubSubLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {    
        use tracing_serde::AsSerde;
        if *event.metadata().level() <= self.level.as_tracing_level() {
            let value = serde_json::json!(event.as_serde());
            if publish("logging", value).is_err() {
                // Ignore any error in publishing logging event
                // Doing otherwise (e.g. logging another event) could be very circular
            }
        }
    }
}

/// Initialize logging
///
/// This initializes a logging subscriber based on configuration and
/// context (e.g. stderr should not be written to if the context
/// is the desktop application).
///
/// # Arguments
///
/// - `level`: the override stderr logging level for example set by the `--debug` flag
/// - `stderr`: should stderr logging be enabled
/// - `pubsub`: should pubsub logging be enabled (for desktop notifications)
/// - `file`: should file logging be enabled
pub fn init(
    level: Option<LoggingLevel>,
    stderr: bool,
    pubsub: bool,
    file: bool,
    config: &config::LoggingConfig,
) -> Result<[tracing_appender::non_blocking::WorkerGuard; 2]> {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter};

    // Stderr logging layer
    let stderr_level = if level.is_some() || stderr {
        level.unwrap_or(config.stderr.level)
    } else {
        LoggingLevel::Never
    };
    let (stderr_writer, stderr_guard) = if stderr_level != LoggingLevel::Never {
        tracing_appender::non_blocking(std::io::stderr())
    } else {
        tracing_appender::non_blocking(std::io::sink())
    };
    let stderr_layer = fmt::Layer::new().with_writer(stderr_writer);

    // Pubsub logging layer (used for desktop notifications)
    let pubsub_level = if pubsub {
        config.desktop.level
    } else {
        LoggingLevel::Never
    };
    let pubsub_layer = PubSubLayer {
        level: pubsub_level,
    };

    // File logging layer
    let file_level = if file {
        config.file.level
    } else {
        LoggingLevel::Never
    };
    let (file_writer, file_guard) = if file_level != LoggingLevel::Never {
        let path = Path::new(&config.file.path);
        let file_appender =
            tracing_appender::rolling::daily(&path.parent().unwrap(), &path.file_name().unwrap());
        tracing_appender::non_blocking(file_appender)
    } else {
        tracing_appender::non_blocking(std::io::sink())
    };
    let file_layer = fmt::Layer::new().json().with_writer(file_writer);

    // Error reporting layer (necessary for using `eyre` crate)
    let error_layer = ErrorLayer::default();

    // tracing_subscriber does not currently allow for different layers to have different
    // so work out the minimal debug level and filter by that in the root subscriber.
    let mut min_level = LoggingLevel::Never;
    if stderr_level < min_level {
        min_level = stderr_level
    }
    if pubsub_level < min_level {
        min_level = pubsub_level
    }
    if file_level < min_level {
        min_level = file_level
    }

    let registry = tracing_subscriber::registry()
        .with(EnvFilter::new(min_level.to_string()))
        .with(pubsub_layer)
        .with(file_layer)
        .with(error_layer);

    if stderr_level == LoggingLevel::Debug {
        let stderr = stderr_layer.pretty();
        registry.with(stderr).init();
    } else {
        let stderr = stderr_layer
            .without_time()
            .with_thread_names(false)
            .with_thread_ids(false)
            .with_target(false)
            .compact();
        registry.with(stderr).init();
    }

    Ok([stderr_guard, file_guard])
}

/// Generate some test tracing events.
///
/// Can be used for testing that events are propagated
/// to subscribers as expected.
#[tracing::instrument]
pub fn test_events() {
    tracing::trace!("A trace event");
    tracing::debug!("A debug event");
    tracing::info!("An info event");
    tracing::warn!("A warn event");
    tracing::error!("An error event");
}
