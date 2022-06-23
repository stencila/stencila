use std::{env, fs, io::Write, path::PathBuf};

use schemars::{gen::SchemaGenerator, schema::Schema, JsonSchema};
use validator::Validate;

use common::{
    defaults::Defaults,
    dirs,
    eyre::{bail, Result},
    once_cell::sync::Lazy,
    serde::{Deserialize, Serialize},
    serde_json::{self, json},
    tokio::sync::Mutex,
    toml, tracing,
};
use events::publish;

use crate::{
    logging, projects, telemetry,
    utils::{json, schemas},
};

/// Get the directory where configuration data is stored
pub fn dir(ensure: bool) -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .unwrap_or_else(|| env::current_dir().unwrap())
        .join("stencila");

    if ensure {
        fs::create_dir_all(&dir)?;
    }

    Ok(dir)
}

#[derive(Debug, JsonSchema, Serialize)]
#[serde(rename_all = "lowercase", crate = "common::serde")]
enum ConfigEventType {
    Set,
    Reset,
}

/// An event associated with changes to the configuration
#[derive(Debug, JsonSchema, Serialize)]
#[serde(crate = "common::serde")]
#[schemars(deny_unknown_fields)]
struct ConfigEvent {
    /// The type of event
    #[serde(rename = "type")]
    type_: ConfigEventType,

    /// The configuration at the time of the event
    #[schemars(schema_with = "ConfigEvent::schema_config")]
    config: Config,
}

impl ConfigEvent {
    /// Generate the JSON Schema for the `config` property to avoid nesting
    fn schema_config(_generator: &mut SchemaGenerator) -> Schema {
        schemas::typescript("Config", true)
    }

    /// Publish an event.
    pub fn publish(type_: ConfigEventType, config: &Config) {
        let event = ConfigEvent {
            type_,
            config: config.clone(),
        };
        publish("config", &event)
    }
}

#[derive(Debug, Default, PartialEq, Clone, JsonSchema, Deserialize, Serialize, Validate)]
#[serde(default, crate = "common::serde")]
#[schemars(deny_unknown_fields)]
pub struct Config {
    pub projects: projects::config::ProjectsConfig,

    pub logging: logging::config::LoggingConfig,

    pub telemetry: telemetry::config::TelemetryConfig,

    #[cfg(feature = "server")]
    pub server: crate::server::config::ServerConfig,

    #[cfg(feature = "plugins")]
    pub plugins: crate::plugins::config::PluginsConfig,

    pub editors: EditorsConfig,

    #[cfg(feature = "upgrade")]
    pub upgrade: crate::upgrade::config::UpgradeConfig,
}

/// Editors
///
/// Configuration settings for document editors.
#[derive(Debug, Defaults, PartialEq, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase", crate = "common::serde")]
#[schemars(deny_unknown_fields)]
pub struct EditorsConfig {
    /// Default format for new documents
    #[def = "\"md\".to_string()"]
    pub default_format: String,

    /// Show line numbers
    #[def = "false"]
    pub line_numbers: bool,

    /// Enable wrapping of lines
    #[def = "true"]
    pub line_wrapping: bool,
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
                if let Some(part) = config.pointer(json::pointer(&pointer).as_str()) {
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
        if let Some(property) = config.pointer_mut(json::pointer(pointer).as_str()) {
            let value = match serde_json::from_str(value) {
                Ok(value) => value,
                Err(_) => serde_json::Value::String(value.into()),
            };
            *property = value;
        } else {
            bail!("No configuration value at pointer: {}", pointer)
        };

        // Now deserialize self from the JSON value
        *self = serde_json::from_value(config)?;

        ConfigEvent::publish(ConfigEventType::Set, self);
        self.write()
    }

    /// Reset one, or all, properties and write to disk
    #[tracing::instrument]
    pub fn reset(&mut self, property: &str) -> Result<()> {
        match property {
            "all" => *self = Config::default(),

            "logging" => self.logging = crate::logging::config::LoggingConfig::default(),

            #[cfg(feature = "plugins")]
            "plugins" => self.plugins = crate::plugins::config::PluginsConfig::default(),

            #[cfg(feature = "server")]
            "server" => self.server = crate::server::config::ServerConfig::default(),

            #[cfg(feature = "upgrade")]
            "upgrade" => self.upgrade = crate::upgrade::config::UpgradeConfig::default(),

            _ => bail!("No top level configuration property named: {}", property),
        }

        ConfigEvent::publish(ConfigEventType::Reset, self);
        self.write()
    }
}

/// A global config store
///
/// Previously, we loaded config on startup and then passed them to various
/// functions in other modules.
/// However, this proved complicated for things like `Project` and `Document`
/// file watching threads when they need access to the current config.
pub static CONFIG: Lazy<Mutex<Config>> =
    Lazy::new(|| Mutex::new(Config::load().expect("Unable to read config")));

/// Get the JSON Schema for the configuration
pub fn schemas() -> Result<serde_json::Value> {
    let schemas = serde_json::Value::Array(vec![
        schemas::generate::<Config>()?,
        schemas::generate::<ConfigEvent>()?,
    ]);
    Ok(schemas)
}

/// CLI options for the `config` command
#[cfg(feature = "cli")]
pub mod commands {
    use cli_utils::{
        clap::{self, Parser},
        result, Result, Run,
    };
    use common::async_trait::async_trait;

    use super::*;

    /// Manage configuration settings
    #[derive(Parser)]
    pub struct Command {
        #[clap(subcommand)]
        pub action: Action,
    }

    #[derive(Parser)]
    pub enum Action {
        Get(Get),
        Set(Set),
        Reset(Reset),
        /// Get the directories used for config, cache etc
        Dirs,
    }

    /// Get configuration properties
    #[derive(Parser)]
    #[clap(alias = "show")]
    pub struct Get {
        /// A pointer to a config property e.g. `upgrade.auto`
        pub pointer: Option<String>,
    }

    /// Set configuration properties
    #[derive(Parser)]
    pub struct Set {
        /// A pointer to a config property e.g. `upgrade.auto`
        pub pointer: String,

        /// The value to set the property to
        pub value: String,
    }

    /// Reset configuration properties to their defaults
    #[derive(Parser)]
    pub struct Reset {
        /// The config property to reset. Use 'all' to reset the entire config.
        pub property: String,
    }

    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            let config = &mut *CONFIG.lock().await;

            match &self.action {
                Action::Get(action) => {
                    let value = config.get(action.pointer.clone())?;
                    result::value(value)
                }
                Action::Set(action) => {
                    config.set(&action.pointer, &action.value)?;
                    result::nothing()
                }
                Action::Reset(action) => {
                    config.reset(&action.property)?;
                    result::nothing()
                }
                #[allow(unused_mut)]
                Action::Dirs => {
                    let mut value = json!({
                        "config": dir(false)?.display().to_string(),
                        "logs": crate::logging::config::dir(false)?.display().to_string(),
                    });

                    #[cfg(feature = "plugins")]
                    {
                        value["plugins"] =
                            json!(crate::plugins::plugins_dir(false)?.display().to_string());
                    }

                    result::value(value)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use common::tokio;

    use super::*;

    #[test]
    fn test_path() -> Result<()> {
        let path = Config::path()?;
        assert!(path.starts_with(std::env::temp_dir()));
        Ok(())
    }

    #[test]
    fn test_display() -> Result<()> {
        let config = Config {
            ..Default::default()
        };

        serde_json::from_value::<Config>(config.get(None)?)?;

        assert_eq!(
            config
                .get(Some("foo.bar".to_string()))
                .unwrap_err()
                .to_string(),
            "No configuration value at pointer: foo.bar"
        );

        Ok(())
    }

    #[cfg(feature = "upgrade")]
    #[test]
    fn test_set() -> Result<()> {
        let mut config = Config {
            ..Default::default()
        };

        config.set("upgrade.auto", "off")?;
        assert_eq!(config.upgrade.auto, "off");

        config.set("upgrade.verbose", "true")?;
        assert!(config.upgrade.verbose);

        assert_eq!(
            config.set("foo.bar", "baz").unwrap_err().to_string(),
            "No configuration value at pointer: foo.bar"
        );

        Ok(())
    }

    #[cfg(feature = "upgrade")]
    #[test]
    fn test_reset() -> Result<()> {
        let mut config: Config = serde_json::from_value(serde_json::json!({
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

    #[cfg(feature = "cli")]
    #[tokio::test]
    async fn test_cli() -> Result<()> {
        use super::commands::*;
        use cli_utils::Run;

        Command {
            action: Action::Get(Get { pointer: None }),
        }
        .run()
        .await?;

        Command {
            action: Action::Set(Set {
                pointer: "upgrade.confirm".to_string(),
                value: "true".to_string(),
            }),
        }
        .run()
        .await?;

        Command {
            action: Action::Reset(Reset {
                property: "upgrade".to_string(),
            }),
        }
        .run()
        .await?;

        Command {
            action: Action::Dirs,
        }
        .run()
        .await?;

        Ok(())
    }
}
