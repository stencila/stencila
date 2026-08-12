//! Runtime configuration for the convert server
//!
//! Every setting keeps the value that was previously hard-coded as its default,
//! so the public converter behaves exactly as before unless an operator opts in
//! by setting the corresponding environment variable.

use std::{
    env, fs, io,
    path::{Path, PathBuf},
    time::Duration,
};

/// The number of bytes in a mebibyte
const BYTES_PER_MIB: usize = 1024 * 1024;

/// The default maximum size of an uploaded file
const DEFAULT_MAX_UPLOAD_BYTES: usize = 25 * BYTES_PER_MIB;

/// The additional request bytes allowed on top of the upload limit
///
/// Covers multipart boundaries, headers, and other form fields.
const REQUEST_OVERHEAD_BYTES: usize = BYTES_PER_MIB;

/// The default time limit for a single conversion
const DEFAULT_CONVERSION_TIMEOUT: Duration = Duration::from_secs(60);

/// The default port that the server binds to
const DEFAULT_PORT: u16 = 8080;

/// The environment variable for [`ServerConfig::conversion_timeout`]
const TIMEOUT_SECONDS_VAR: &str = "STENCILA_CONVERT_TIMEOUT_SECONDS";

/// The environment variable for [`ServerConfig::max_upload_bytes`]
const MAX_UPLOAD_MB_VAR: &str = "STENCILA_CONVERT_MAX_UPLOAD_MB";

/// The environment variable for [`ServerConfig::max_concurrency`]
const MAX_CONCURRENCY_VAR: &str = "STENCILA_CONVERT_MAX_CONCURRENCY";

/// The environment variable for [`ServerConfig::artifacts_dir`]
const ARTIFACTS_DIR_VAR: &str = "STENCILA_CONVERT_ARTIFACTS_DIR";

/// The environment variable for [`ServerConfig::port`]
const PORT_VAR: &str = "STENCILA_CONVERT_PORT";

/// The configuration of a convert server
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerConfig {
    /// The time limit for a single conversion
    pub conversion_timeout: Duration,

    /// The maximum size of an uploaded file
    pub max_upload_bytes: usize,

    /// The maximum number of conversions to run at the same time
    ///
    /// `None` means unlimited, which is the default and matches the previous
    /// behaviour of the server.
    pub max_concurrency: Option<usize>,

    /// The directory under which decoding artifacts (e.g. cached OCR output)
    /// should be retained
    ///
    /// `None`, the default, disables artifact caching entirely: each conversion
    /// is decoded with `no_artifacts` and `ignore_artifacts` set.
    pub artifacts_dir: Option<PathBuf>,

    /// The port to bind to
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            conversion_timeout: DEFAULT_CONVERSION_TIMEOUT,
            max_upload_bytes: DEFAULT_MAX_UPLOAD_BYTES,
            max_concurrency: None,
            artifacts_dir: None,
            port: DEFAULT_PORT,
        }
    }
}

impl ServerConfig {
    /// Read the configuration from the process environment
    ///
    /// Any variable that is unset, empty, or invalid falls back to the default.
    pub fn from_env() -> Self {
        Self::from_vars(|name| env::var(name).ok())
    }

    /// Read the configuration using a variable lookup function
    ///
    /// Separated from [`Self::from_env`] so that it can be tested without
    /// mutating the process environment.
    pub(crate) fn from_vars(get: impl Fn(&str) -> Option<String>) -> Self {
        let defaults = Self::default();

        let conversion_timeout = parse_var(&get, TIMEOUT_SECONDS_VAR, |value| {
            value
                .parse::<u64>()
                .ok()
                .filter(|seconds| *seconds > 0)
                .map(Duration::from_secs)
        })
        .unwrap_or(defaults.conversion_timeout);

        let max_upload_bytes = parse_var(&get, MAX_UPLOAD_MB_VAR, |value| {
            value
                .parse::<usize>()
                .ok()
                .filter(|megabytes| *megabytes > 0)
                .and_then(|megabytes| megabytes.checked_mul(BYTES_PER_MIB))
        })
        .unwrap_or(defaults.max_upload_bytes);

        let max_concurrency = parse_var(&get, MAX_CONCURRENCY_VAR, |value| {
            value.parse::<usize>().ok().filter(|limit| *limit > 0)
        })
        .or(defaults.max_concurrency);

        let artifacts_dir = parse_var(&get, ARTIFACTS_DIR_VAR, |value| Some(PathBuf::from(value)))
            .or(defaults.artifacts_dir);

        let port = parse_var(&get, PORT_VAR, |value| {
            value.parse::<u16>().ok().filter(|port| *port > 0)
        })
        .unwrap_or(defaults.port);

        Self {
            conversion_timeout,
            max_upload_bytes,
            max_concurrency,
            artifacts_dir,
            port,
        }
    }

    /// The maximum size of an entire request body
    pub(crate) fn max_request_bytes(&self) -> usize {
        self.max_upload_bytes.saturating_add(REQUEST_OVERHEAD_BYTES)
    }

    /// The maximum upload size, in whole mebibytes, for use in error messages
    pub(crate) fn max_upload_mb(&self) -> usize {
        self.max_upload_bytes / BYTES_PER_MIB
    }

    /// Whether decoding artifacts should be created and reused
    pub(crate) fn artifacts_enabled(&self) -> bool {
        self.artifacts_dir.is_some()
    }

    /// Create the `.stencila/artifacts` tree in the configured artifacts directory
    ///
    /// Creating it up front means that the upward walk done by
    /// `stencila_dirs::closest_stencila_dir` stops at the configured directory,
    /// rather than at some `.stencila` directory above it.
    ///
    /// Returns `None` when no artifacts directory is configured.
    fn ensure_artifacts_dir(&self) -> io::Result<Option<&Path>> {
        let Some(dir) = &self.artifacts_dir else {
            return Ok(None);
        };

        fs::create_dir_all(dir.join(".stencila").join("artifacts"))?;

        Ok(Some(dir))
    }

    /// Prepare the process for artifact caching
    ///
    /// Codecs resolve their artifact cache from the closest `.stencila`
    /// directory to the process working directory (see
    /// `stencila_dirs::closest_artifacts_for`), and the cache key is a hash of
    /// the input content, not its path. So making the configured artifacts
    /// directory the process working directory is enough for uploads landing in
    /// different temporary directories to share one cache.
    ///
    /// Must be called once, at startup, before the server begins accepting
    /// requests, because it changes process-global state. Does nothing when no
    /// artifacts directory is configured. If the directory can not be prepared,
    /// logs a warning and disables artifact retention.
    pub(crate) fn prepare_artifacts_dir(&mut self) {
        let Some(dir) = self.artifacts_dir.clone() else {
            return;
        };

        let result = self
            .ensure_artifacts_dir()
            .and_then(|dir| dir.map(env::set_current_dir).transpose())
            .map(|_| ());

        if let Err(error) = result {
            tracing::warn!(
                "Ignoring unusable value for `{ARTIFACTS_DIR_VAR}` (`{}`): {error}",
                dir.display()
            );
            self.artifacts_dir = None;
        }
    }
}

/// Parse an environment variable, warning, and falling back to the default, if invalid
fn parse_var<T>(
    get: impl Fn(&str) -> Option<String>,
    name: &str,
    parse: impl Fn(&str) -> Option<T>,
) -> Option<T> {
    let value = get(name)?;
    let value = value.trim();
    if value.is_empty() {
        tracing::warn!("Ignoring empty value for `{name}`");
        return None;
    }

    let parsed = parse(value);
    if parsed.is_none() {
        tracing::warn!("Ignoring invalid value for `{name}`: {value}");
    }

    parsed
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn config_from(vars: &[(&str, &str)]) -> ServerConfig {
        let vars: HashMap<String, String> = vars
            .iter()
            .map(|(name, value)| ((*name).to_string(), (*value).to_string()))
            .collect();
        ServerConfig::from_vars(|name| vars.get(name).cloned())
    }

    #[test]
    fn falls_back_to_the_previous_hard_coded_values() {
        let config = config_from(&[]);

        assert_eq!(config.conversion_timeout, Duration::from_secs(60));
        assert_eq!(config.max_upload_bytes, 25 * BYTES_PER_MIB);
        assert_eq!(config.max_request_bytes(), 26 * BYTES_PER_MIB);
        assert_eq!(config.max_concurrency, None);
        assert_eq!(config.artifacts_dir, None);
        assert!(!config.artifacts_enabled());
        assert_eq!(config.port, 8080);
        assert_eq!(config, ServerConfig::default());
    }

    #[test]
    fn reads_settings_from_the_environment() {
        let config = config_from(&[
            (TIMEOUT_SECONDS_VAR, "300"),
            (MAX_UPLOAD_MB_VAR, "200"),
            (MAX_CONCURRENCY_VAR, "4"),
            (ARTIFACTS_DIR_VAR, "/data/artifacts"),
            (PORT_VAR, "9090"),
        ]);

        assert_eq!(config.conversion_timeout, Duration::from_secs(300));
        assert_eq!(config.max_upload_bytes, 200 * BYTES_PER_MIB);
        assert_eq!(config.max_request_bytes(), 201 * BYTES_PER_MIB);
        assert_eq!(config.max_upload_mb(), 200);
        assert_eq!(config.max_concurrency, Some(4));
        assert_eq!(config.artifacts_dir, Some(PathBuf::from("/data/artifacts")));
        assert!(config.artifacts_enabled());
        assert_eq!(config.port, 9090);
    }

    #[test]
    fn creates_the_artifacts_tree_only_when_configured() -> io::Result<()> {
        assert!(ServerConfig::default().ensure_artifacts_dir()?.is_none());

        let temp_dir = tempfile::tempdir()?;
        let dir = temp_dir.path().join("artifacts");
        let config = ServerConfig {
            artifacts_dir: Some(dir.clone()),
            ..Default::default()
        };

        assert_eq!(config.ensure_artifacts_dir()?, Some(dir.as_path()));
        assert!(dir.join(".stencila").join("artifacts").is_dir());

        // Idempotent, so restarting a server with an existing cache is fine
        assert!(config.ensure_artifacts_dir()?.is_some());

        Ok(())
    }

    #[test]
    fn ignores_empty_and_invalid_values() {
        let config = config_from(&[
            (TIMEOUT_SECONDS_VAR, "0"),
            (MAX_UPLOAD_MB_VAR, "  "),
            (MAX_CONCURRENCY_VAR, "not-a-number"),
            (ARTIFACTS_DIR_VAR, ""),
            (PORT_VAR, "99999"),
        ]);

        assert_eq!(config, ServerConfig::default());
    }

    #[test]
    fn ignores_zero_port() {
        let config = config_from(&[(PORT_VAR, "0")]);

        assert_eq!(config.port, DEFAULT_PORT);
    }

    #[test]
    fn falls_back_from_an_unusable_artifacts_directory() -> io::Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let file = temp_dir.path().join("file");
        fs::write(&file, [])?;

        let mut config = ServerConfig {
            artifacts_dir: Some(file.join("artifacts")),
            ..Default::default()
        };
        config.prepare_artifacts_dir();

        assert_eq!(config.artifacts_dir, None);
        Ok(())
    }
}
