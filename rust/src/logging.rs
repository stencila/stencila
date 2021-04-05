use anyhow::Result;
use defaults::Defaults;
use serde::{Deserialize, Serialize};
use std::path::Path;
use strum::ToString;
use validator::Validate;

#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Serialize, ToString)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
    Never,
}

#[derive(Debug, PartialEq, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Plain,
    Pretty,
    Json,
}

#[cfg(feature = "config")]
pub mod config {
    use super::*;
    use crate::util::dirs;

    #[derive(Debug, Defaults, PartialEq, Clone, Copy, Deserialize, Serialize, Validate)]
    pub struct StdErr {
        /// The maximum log level to emit
        #[def = "Level::Info"]
        pub level: Level,

        /// The format for the logs entries
        #[def = "Format::Pretty"]
        pub format: Format,
    }

    #[derive(Debug, Defaults, PartialEq, Clone, Deserialize, Serialize, Validate)]
    pub struct File {
        /// The path of the log file
        #[def = "default_file_path()"]
        pub path: String,

        /// The maximum log level to emit
        #[def = "Level::Debug"]
        pub level: Level,
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

    #[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize, Validate)]
    pub struct Config {
        pub stderr: StdErr,
        pub file: File,
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

/// Initialize a logging subscriber based on passed args and read config.
pub fn init(
    level: Option<Level>,
    config: &config::Config,
) -> Result<[tracing_appender::non_blocking::WorkerGuard; 2]> {
    use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter};

    let config::Config { stderr, file } = config;

    let level = level.unwrap_or(stderr.level);

    let (stderr_writer, stderr_guard) = if level != Level::Never {
        tracing_appender::non_blocking(std::io::stderr())
    } else {
        tracing_appender::non_blocking(std::io::sink())
    };
    let stderr_layer = fmt::Layer::new()
        .pretty()
        .without_time()
        .with_writer(stderr_writer);

    let (file_writer, file_guard) = if file.level != Level::Never {
        let path = Path::new(&file.path);
        let file_appender =
            tracing_appender::rolling::daily(&path.parent().unwrap(), &path.file_name().unwrap());
        tracing_appender::non_blocking(file_appender)
    } else {
        tracing_appender::non_blocking(std::io::sink())
    };
    let file_layer = fmt::Layer::new().json().with_writer(file_writer);

    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::new(level.to_string()))
        .with(stderr_layer)
        .with(file_layer);

    tracing::subscriber::set_global_default(subscriber)?;

    Ok([stderr_guard, file_guard])
}
