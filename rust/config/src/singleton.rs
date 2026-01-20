//! Global configuration singleton
//!
//! Provides a process-wide cached configuration that is lazily initialized
//! and automatically updated when config files change.
//!
//! # Usage
//!
//! ```ignore
//! // Get the global config (initializes lazily from CWD on first call)
//! let config = stencila_config::get()?;
//!
//! // Access config fields
//! if let Some(site) = &config.site {
//!     println!("Site root: {:?}", site.root);
//! }
//!
//! // Subscribe to config changes
//! let mut rx = stencila_config::subscribe()?;
//! tokio::spawn(async move {
//!     while let Ok(event) = rx.recv().await {
//!         println!("Config updated at {:?}", event.timestamp);
//!     }
//! });
//! ```

use std::{
    path::{Path, PathBuf},
    sync::{Arc, OnceLock, RwLock},
};

use eyre::{Result, eyre};
use tokio::sync::{broadcast, mpsc};

use crate::{CONFIG_FILENAME, Config, utils::build_figment, watch};

/// Internal state for the config singleton
struct ConfigSingleton {
    /// The cached configuration
    config: Arc<RwLock<Config>>,

    /// Broadcast channel for change notifications
    change_sender: broadcast::Sender<ConfigChangeEvent>,
}

/// Event emitted when configuration changes
#[derive(Clone, Debug)]
pub struct ConfigChangeEvent {
    /// Immutable snapshot of the updated config
    pub config: Arc<Config>,

    /// When the change was detected
    pub timestamp: std::time::Instant,
}

/// The global singleton instance
static CONFIG_SINGLETON: OnceLock<ConfigSingleton> = OnceLock::new();

/// Load and validate configuration from a path
///
/// Searches up the directory tree from the path for `stencila.toml` and
/// `stencila.local.toml` files, merges them, and validates the result.
///
/// This is useful for validating config files without initializing the
/// singleton (e.g., in tests or CLI tools that validate config).
pub fn load_and_validate(workspace_dir: &Path) -> Result<Config> {
    let figment = build_figment(workspace_dir, true)?;
    let mut config: Config = figment.extract().map_err(|error| eyre!("{error}"))?;

    // Set the workspace directory on the config
    config.workspace_dir = workspace_dir.to_path_buf();

    // Validate workspace configuration
    if let Some(workspace) = &config.workspace {
        workspace.validate()?;
    }

    // Validate site configuration
    if let Some(site) = &config.site {
        site.validate()?;
    }

    // Validate site navigation items (must be internal routes)
    if let Some(site) = &config.site
        && let Some(nav) = &site.nav
    {
        for item in nav {
            item.validate()?;
        }
    }

    // Validate all route configurations
    if let Some(site) = &config.site
        && let Some(routes) = &site.routes
    {
        for (path_key, target) in routes {
            target.validate(path_key)?;
        }
    }

    // Validate all remote configurations
    if let Some(remotes) = &config.remotes {
        for (path_key, value) in remotes {
            value.validate(path_key)?;
        }
    }

    // Validate all output configurations
    if let Some(outputs) = &config.outputs {
        for (path_key, target) in outputs {
            target.validate(path_key)?;
        }
    }

    Ok(config)
}

/// Find workspace root by searching up for stencila.toml or stencila.local.toml
fn find_workspace_root(start: &Path) -> Result<PathBuf> {
    use crate::CONFIG_LOCAL_FILENAME;

    // Search up from start path for config file (either stencila.toml or stencila.local.toml)
    // Try stencila.toml first, then stencila.local.toml
    if let Some(config_path) = crate::find_config_file(start, CONFIG_FILENAME)
        && let Some(parent) = config_path.parent()
    {
        return Ok(parent.to_path_buf());
    }

    if let Some(config_path) = crate::find_config_file(start, CONFIG_LOCAL_FILENAME)
        && let Some(parent) = config_path.parent()
    {
        return Ok(parent.to_path_buf());
    }

    // No workspace config found, fall back to user config directory
    if let Ok(user_config_dir) = stencila_dirs::get_app_dir(stencila_dirs::DirType::Config, false)
        && (user_config_dir.join(CONFIG_FILENAME).exists()
            || user_config_dir.join(CONFIG_LOCAL_FILENAME).exists())
    {
        return Ok(user_config_dir);
    }

    // Default to the start path if no config exists anywhere
    // This allows the singleton to work even without a config file
    Ok(start.to_path_buf())
}

/// Initialize the singleton from a path
fn init_singleton(path: &Path) -> Result<()> {
    // Find workspace root by searching for config file
    let workspace_dir = find_workspace_root(path)?;

    // Load and validate initial configuration
    let initial_config = load_and_validate(&workspace_dir)?;

    // Create broadcast channel (capacity 16 is reasonable for config changes)
    let (change_sender, _) = broadcast::channel(16);

    // Create the singleton
    let singleton = ConfigSingleton {
        config: Arc::new(RwLock::new(initial_config)),
        change_sender,
    };

    // Try to set the singleton (returns Err if already set)
    // This is fine - another thread may have initialized it
    let _ = CONFIG_SINGLETON.set(singleton);

    // Start the file watcher in the background
    let config_arc = CONFIG_SINGLETON
        .get()
        .map(|s| Arc::clone(&s.config))
        .ok_or_else(|| eyre!("Failed to get singleton after initialization"))?;
    let change_sender = CONFIG_SINGLETON
        .get()
        .map(|s| s.change_sender.clone())
        .ok_or_else(|| eyre!("Failed to get singleton after initialization"))?;

    tokio::spawn(async move {
        run_watcher_loop(workspace_dir, config_arc, change_sender).await;
    });

    Ok(())
}

/// The main watcher loop
async fn run_watcher_loop(
    workspace_dir: PathBuf,
    config_arc: Arc<RwLock<Config>>,
    change_sender: broadcast::Sender<ConfigChangeEvent>,
) {
    // Get the watch receiver using existing infrastructure
    let receiver: Option<mpsc::Receiver<Result<Config>>> = match watch::watch(&workspace_dir).await
    {
        Ok(Some(rx)) => Some(rx),
        Ok(None) => {
            tracing::debug!("No config file to watch at {:?}", workspace_dir);
            return;
        }
        Err(e) => {
            tracing::warn!("Failed to start config watcher: {e}");
            return;
        }
    };

    let Some(mut receiver) = receiver else {
        return;
    };

    while let Some(result) = receiver.recv().await {
        match result {
            Ok(mut new_config) => {
                // Set workspace_dir on the reloaded config
                new_config.workspace_dir = workspace_dir.clone();

                // Atomically update cached config
                {
                    let mut config = config_arc.write().expect("config lock poisoned");
                    *config = new_config.clone();
                }

                // Notify subscribers (ignore errors if no receivers)
                let event = ConfigChangeEvent {
                    config: Arc::new(new_config),
                    timestamp: std::time::Instant::now(),
                };
                let _ = change_sender.send(event);

                tracing::debug!("Config reloaded from file change");
            }
            Err(e) => {
                tracing::warn!("Config reload failed, keeping cached: {e}");
            }
        }
    }

    tracing::debug!("Config watcher stopped");
}

/// Get the global configuration
///
/// On first call, loads config from the current working directory and starts
/// a background file watcher. Subsequent calls return the cached config.
///
/// # Errors
///
/// Returns an error if:
/// - The current working directory cannot be determined
/// - The config file cannot be loaded or parsed
/// - Config validation fails
///
/// # Example
///
/// ```ignore
/// let config = stencila_config::get()?;
/// if let Some(site) = &config.site {
///     println!("Site domain: {:?}", site.domain);
/// }
/// ```
pub fn get() -> Result<Config> {
    // Fast path: already initialized
    if let Some(singleton) = CONFIG_SINGLETON.get() {
        return Ok(singleton
            .config
            .read()
            .expect("config lock poisoned")
            .clone());
    }

    // Slow path: initialize from CWD
    let cwd = std::env::current_dir()?;
    init_singleton(&cwd)?;

    // Now it's initialized
    CONFIG_SINGLETON
        .get()
        .map(|s| s.config.read().expect("config lock poisoned").clone())
        .ok_or_else(|| eyre!("Failed to initialize config singleton"))
}

/// Subscribe to configuration change notifications
///
/// Returns a receiver that will emit [`ConfigChangeEvent`] whenever
/// the configuration is updated from file changes.
///
/// On first call (if `get()` hasn't been called yet), this will lazily
/// initialize the config from the current working directory.
///
/// # Errors
///
/// Returns an error if the config cannot be initialized.
///
/// # Example
///
/// ```ignore
/// let mut rx = stencila_config::subscribe()?;
/// tokio::spawn(async move {
///     while let Ok(event) = rx.recv().await {
///         println!("Config updated: {:?}", event.config.site);
///     }
/// });
/// ```
pub fn subscribe() -> Result<broadcast::Receiver<ConfigChangeEvent>> {
    // Ensure initialized (triggers lazy init if needed)
    let _ = get()?;

    CONFIG_SINGLETON
        .get()
        .map(|s| s.change_sender.subscribe())
        .ok_or_else(|| eyre!("Config singleton not available"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_find_workspace_root_with_config() -> Result<()> {
        let temp = TempDir::new()?;
        let config_path = temp.path().join(CONFIG_FILENAME);
        std::fs::write(&config_path, "# Test config\n")?;

        let result = find_workspace_root(temp.path())?;
        assert_eq!(result, temp.path());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_workspace_root_no_config() -> Result<()> {
        let temp = TempDir::new()?;

        // Should fall back to the start path
        let result = find_workspace_root(temp.path());
        assert!(result.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_load_and_validate_empty_config() -> Result<()> {
        let temp = TempDir::new()?;
        let config_path = temp.path().join(CONFIG_FILENAME);
        std::fs::write(&config_path, "# Empty config\n")?;

        let config = load_and_validate(temp.path())?;
        assert_eq!(config.workspace_dir, temp.path());
        assert!(config.workspace.is_none());
        assert!(config.site.is_none());

        Ok(())
    }
}
