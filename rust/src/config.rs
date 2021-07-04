use crate::logging::config::LoggingConfig;
use crate::plugins::config::PluginsConfig;
use crate::serve::config::ServeConfig;
use crate::upgrade::config::UpgradeConfig;
use crate::{binaries, logging, plugins, projects, serve, telemetry, upgrade, utils::schemas};
use eyre::{bail, Result};
use once_cell::sync::Lazy;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};
use validator::Validate;

/// Get the directory where configuration data is stored
pub fn dir(ensure: bool) -> Result<PathBuf> {
    let config_base = dirs_next::config_dir().unwrap_or_else(|| env::current_dir().unwrap());
    let dir = match env::consts::OS {
        "macos" => config_base.join("Stencila"),
        "windows" => config_base.join("Stencila").join("Config"),
        _ => config_base.join("stencila"),
    };
    if ensure {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

#[derive(Debug, Default, PartialEq, Clone, JsonSchema, Deserialize, Serialize, Validate)]
#[serde(default)]
#[schemars(deny_unknown_fields)]
pub struct Config {
    #[validate]
    pub projects: projects::config::ProjectsConfig,

    #[validate]
    pub logging: logging::config::LoggingConfig,

    #[validate]
    pub telemetry: telemetry::config::TelemetryConfig,

    #[validate]
    pub serve: serve::config::ServeConfig,

    #[validate]
    pub plugins: plugins::config::PluginsConfig,

    #[validate]
    pub binaries: binaries::config::BinariesConfig,

    #[validate]
    pub upgrade: upgrade::config::UpgradeConfig,
}

impl Config {
    const CONFIG_FILE: &'static str = "config.toml";

    /// Get the path of the configuration file
    #[tracing::instrument]
    fn path() -> Result<PathBuf> {
        #[cfg(not(test))]
        return Ok(dir(true)?.join(Config::CONFIG_FILE));

        // When running tests, avoid messing with users existing config
        #[cfg(test)]
        {
            let path = std::env::temp_dir()
                .join("stencila")
                .join("test")
                .join(Config::CONFIG_FILE);
            fs::create_dir_all(path.parent().unwrap())?;
            Ok(path)
        }
    }

    /// Ensure that a string is a valid JSON pointer
    ///
    /// Replaces dots (`.`) with slashes (`/`) and ensures a
    /// leading slash.
    #[tracing::instrument]
    pub fn json_pointer(pointer: &str) -> String {
        let pointer = pointer.replace(".", "/");
        if pointer.starts_with('/') {
            pointer
        } else {
            format!("/{}", pointer)
        }
    }

    /// Read a config from the configuration file
    #[tracing::instrument]
    pub fn load() -> Result<Config> {
        let config_file = Config::path()?;

        let content = if config_file.exists() {
            match fs::read_to_string(config_file.clone()) {
                Ok(content) => content,
                // If there was a problem reading the config file warn the user
                // but return an empty string.
                Err(error) => {
                    tracing::warn!("Error while reading config file: {}", error.to_string());
                    String::new()
                }
            }
        } else {
            // If there is no config file just return an empty string
            String::new()
        };

        let config = match toml::from_str(content.as_str()) {
            Ok(config) => config,
            // If there was a problem reading the config file. e.g syntax error, a change
            // in the enum variants for an option, then warn the user and return the
            // defaults instead. This avoids a "corrupt" config file from preventing the
            // user from doing anything.
            Err(error) => {
                tracing::warn!(
                "Error while parsing config file; use `stencila config reset all` or edit {} to fix; defaults will be used: {}",
                config_file.display(),
                error.to_string()
            );
                Config {
                    ..Default::default()
                }
            }
        };

        if let Err(errors) = config.validate() {
            tracing::error!(
            "Invalid config file; use `stencila config set`, `stencila config reset` or edit {} to fix.\n\n{}",
            config_file.display(),
            serde_json::to_string_pretty(&errors)?
        )
        }

        Ok(config)
    }

    /// Write a config to the configuration file
    #[tracing::instrument]
    pub fn write(&self) -> Result<()> {
        self.validate()?;

        let config_file = Config::path()?;
        let mut file = fs::File::create(config_file)?;
        file.write_all(toml::to_string(&self)?.as_bytes())?;
        Ok(())
    }

    /// Get a config property
    #[tracing::instrument]
    pub fn get(&self, pointer: Option<String>) -> Result<serde_json::Value> {
        match pointer {
            None => Ok(serde_json::to_value(self)?),
            Some(pointer) => {
                let config = serde_json::to_value(self)?;
                if let Some(part) = config.pointer(Config::json_pointer(&pointer).as_str()) {
                    let json = serde_json::to_string(part)?;
                    let part: toml::Value = serde_json::from_str(&json)?;
                    Ok(serde_json::to_value(part)?)
                } else {
                    bail!("No configuration value at pointer: {}", pointer)
                }
            }
        }
    }

    /// Set a property and write to disk
    #[tracing::instrument]
    pub fn set(&mut self, pointer: &str, value: &str) -> Result<()> {
        // Serialize self to a JSON value and set property
        let mut config = serde_json::to_value(&self)?;
        if let Some(property) = config.pointer_mut(Config::json_pointer(&pointer).as_str()) {
            let value = match serde_json::from_str(&value) {
                Ok(value) => value,
                Err(_) => serde_json::Value::String(value.into()),
            };
            *property = value;
        } else {
            bail!("No configuration value at pointer: {}", pointer)
        };

        // Now deserialize self from the JSON value
        *self = serde_json::from_value(config)?;

        self.write()
    }

    /// Reset one, or all, properties and write to disk
    #[tracing::instrument]
    pub fn reset(&mut self, property: &str) -> Result<()> {
        match property {
            "all" => *self = Config::default(),
            "logging" => self.logging = LoggingConfig::default(),
            "plugins" => self.plugins = PluginsConfig::default(),
            "serve" => self.serve = ServeConfig::default(),
            "upgrade" => self.upgrade = UpgradeConfig::default(),
            _ => bail!("No top level configuration property named: {}", property),
        }

        self.write()
    }
}

/// A global config store
///
/// Previously, we loaded config on startup and then passed them to various
/// functions in other modules.
/// However, this proved complicated for things like `Project` and `Document`
/// file watching threads when they need access to the current config.
pub static CONFIG: Lazy<Arc<Mutex<Config>>> =
    Lazy::new(|| Arc::new(Mutex::new(Config::load().expect("Unable to read config"))));

/// Lock the global config store
pub async fn lock() -> MutexGuard<'static, Config> {
    CONFIG.lock().await
}

/// Get the JSON Schema for the configuration
pub fn schema() -> Result<serde_json::Value> {
    schemas::generate::<Config>()
}

/// CLI options for the `config` command
#[cfg(feature = "cli")]
pub mod cli {
    use super::*;
    use crate::cli::display;
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Manage configuration settings",
        setting = structopt::clap::AppSettings::ColoredHelp,
        setting = structopt::clap::AppSettings::VersionlessSubcommands
    )]
    pub struct Command {
        #[structopt(subcommand)]
        pub action: Action,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder
    )]
    pub enum Action {
        Get(Get),
        Set(Set),
        Reset(Reset),

        #[structopt(
            about = "Get the directories used for config, cache etc",
            setting = structopt::clap::AppSettings::ColoredHelp
        )]
        Dirs,

        #[structopt(
            about = "Get the JSON Schema for the configuration",
            setting = structopt::clap::AppSettings::ColoredHelp
        )]
        Schema,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Get configuration properties",
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Get {
        /// A pointer to a config property e.g. `upgrade.auto`
        pub pointer: Option<String>,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Set configuration properties",
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Set {
        /// A pointer to a config property e.g. `upgrade.auto`
        pub pointer: String,

        /// The value to set the property to
        pub value: String,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Reset configuration properties to their defaults",
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Reset {
        /// The config property to reset. Use 'all' to reset the entire config.
        pub property: String,
    }

    pub async fn run(args: Command) -> display::Result {
        let Command { action } = args;

        let config = &mut *lock().await;

        match action {
            Action::Get(action) => {
                let Get { pointer } = action;
                let value = config.get(pointer)?;
                display::value(value)
            }
            Action::Set(action) => {
                let Set { pointer, value } = action;
                config.set(&pointer, &value)?;
                display::nothing()
            }
            Action::Reset(action) => {
                let Reset { property } = action;
                config.reset(&property)?;
                display::nothing()
            }
            Action::Dirs => {
                let config_dir = dir(false)?.display().to_string();
                let logs_dir = crate::logging::config::dir(false)?.display().to_string();
                let plugins_dir = crate::plugins::plugins_dir(false)?.display().to_string();
                let binaries_dir = crate::binaries::binaries_dir().display().to_string();
                let value = serde_json::json!({
                    "config": config_dir,
                    "logs": logs_dir,
                    "plugins": plugins_dir,
                    "binaries": binaries_dir
                });
                display::value(value)
            }
            Action::Schema => {
                let value = schema()?;
                display::value(value)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_path() -> Result<()> {
        let path = Config::path()?;
        assert!(path.starts_with(std::env::temp_dir()));
        Ok(())
    }

    #[test]
    fn test_json_pointer() {
        assert_eq!(Config::json_pointer("a"), "/a");
        assert_eq!(Config::json_pointer("a/b"), "/a/b");
        assert_eq!(Config::json_pointer("/a.b"), "/a/b");
        assert_eq!(Config::json_pointer("a.b.c"), "/a/b/c");
    }

    #[test]
    fn test_display() -> Result<()> {
        let config = Config {
            ..Default::default()
        };

        serde_json::from_value::<Config>(config.get(None)?)?;

        serde_json::from_value::<ServeConfig>(config.get(Some("serve".to_string()))?)?;

        serde_json::from_value::<UpgradeConfig>(config.get(Some("upgrade".to_string()))?)?;

        assert_eq!(
            config
                .get(Some("foo.bar".to_string()))
                .unwrap_err()
                .to_string(),
            "No configuration value at pointer: foo.bar"
        );

        Ok(())
    }

    #[test]
    fn test_set() -> Result<()> {
        let mut config = Config {
            ..Default::default()
        };

        config.set("upgrade.auto", "off")?;
        assert_eq!(config.upgrade.auto, "off");

        config.set("upgrade.verbose", "true")?;
        assert_eq!(config.upgrade.verbose, true);

        assert_eq!(
            config.set("foo.bar", "baz").unwrap_err().to_string(),
            "No configuration value at pointer: foo.bar"
        );

        Ok(())
    }

    #[test]
    fn test_reset() -> Result<()> {
        let mut config: Config = serde_json::from_value(json!({
            "upgrade": {
                "auto": "off",
                "verbose": true,
                "confirm": true,
            }
        }))?;

        let default = Config {
            ..Default::default()
        };

        config.reset("upgrade")?;
        assert_eq!(config.upgrade.auto, default.upgrade.auto);
        assert_eq!(config.upgrade.verbose, default.upgrade.verbose);

        config.reset("all")?;
        assert_eq!(config, default);

        assert_eq!(
            config.reset("foo").unwrap_err().to_string(),
            "No top level configuration property named: foo"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_cli() -> Result<()> {
        use super::cli::*;

        run(Command {
            action: Action::Get(Get { pointer: None }),
        })
        .await?;

        run(Command {
            action: Action::Set(Set {
                pointer: "upgrade.confirm".to_string(),
                value: "true".to_string(),
            }),
        })
        .await?;

        run(Command {
            action: Action::Reset(Reset {
                property: "upgrade".to_string(),
            }),
        })
        .await?;

        run(Command {
            action: Action::Dirs,
        })
        .await?;

        Ok(())
    }
}
