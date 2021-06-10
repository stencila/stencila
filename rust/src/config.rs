use crate::{logging, plugins, projects, serve, telemetry, upgrade, utils::schemas};
use eyre::{bail, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use validator::Validate;

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
    pub upgrade: upgrade::config::UpgradeConfig,
}

/// Get the JSON Schema for the configuration
pub fn schema() -> Result<serde_json::Value> {
    schemas::generate::<Config>()
}

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

const CONFIG_FILE: &str = "config.toml";

/// Get the path of the configuration file
#[tracing::instrument]
fn path() -> Result<PathBuf> {
    #[cfg(not(test))]
    return Ok(dir(true)?.join(CONFIG_FILE));

    // When running tests, avoid messing with users existing config
    #[cfg(test)]
    {
        let path = std::env::temp_dir()
            .join("stencila")
            .join("test")
            .join(CONFIG_FILE);
        fs::create_dir_all(path.parent().unwrap())?;
        Ok(path)
    }
}

/// Read a config from the configuration file
#[tracing::instrument]
pub fn read() -> Result<Config> {
    let config_file = path()?;

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
pub fn write(config: &Config) -> Result<()> {
    let config_file = path()?;

    let mut file = fs::File::create(config_file)?;
    file.write_all(toml::to_string(&config)?.as_bytes())?;
    Ok(())
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

/// Get a config property
#[tracing::instrument]
pub fn get(config: &Config, pointer: Option<String>) -> Result<serde_json::Value> {
    match pointer {
        None => Ok(serde_json::to_value(config)?),
        Some(pointer) => {
            let config = serde_json::to_value(config)?;
            if let Some(part) = config.pointer(json_pointer(&pointer).as_str()) {
                let json = serde_json::to_string(part)?;
                let part: toml::Value = serde_json::from_str(&json)?;
                Ok(serde_json::to_value(part)?)
            } else {
                bail!("No configuration value at pointer: {}", pointer)
            }
        }
    }
}

/// Validate a configuration
#[tracing::instrument]
pub fn validate(config: &Config) -> Result<()> {
    if let Err(errors) = config.validate() {
        bail!(
            "Invalid configuration value/s:\n\n{}",
            serde_json::to_string_pretty(&errors)?
        )
    } else {
        Ok(())
    }
}

/// Set a config property
#[tracing::instrument]
pub fn set(config: &Config, pointer: &str, value: &str) -> Result<Config> {
    let mut config = serde_json::to_value(config)?;
    if let Some(property) = config.pointer_mut(json_pointer(&pointer).as_str()) {
        let value = match serde_json::from_str(&value) {
            Ok(value) => value,
            Err(_) => serde_json::Value::String(value.into()),
        };
        *property = value;
    } else {
        bail!("No configuration value at pointer: {}", pointer)
    };

    let config: Config = serde_json::from_value(config)?;
    validate(&config)?;
    Ok(config)
}

/// Reset a config property
#[tracing::instrument]
pub fn reset(config: &Config, property: &str) -> Result<Config> {
    Ok(match property {
        "all" => Default::default(),
        "logging" => Config {
            logging: Default::default(),
            ..config.clone()
        },
        "plugins" => Config {
            plugins: Default::default(),
            ..config.clone()
        },
        "serve" => Config {
            serve: Default::default(),
            ..config.clone()
        },
        "upgrade" => Config {
            upgrade: Default::default(),
            ..config.clone()
        },
        _ => bail!("No top level configuration property named: {}", property),
    })
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

    pub fn run(args: Command, config: &mut Config) -> display::Result {
        let Command { action } = args;
        match action {
            Action::Get(action) => {
                let Get { pointer } = action;
                let value = get(config, pointer)?;
                display::value(value)
            }
            Action::Set(action) => {
                let Set { pointer, value } = action;
                *config = set(config, &pointer, &value)?;
                write(config)?;
                display::nothing()
            }
            Action::Reset(action) => {
                let Reset { property } = action;
                *config = reset(config, &property)?;
                write(config)?;
                display::nothing()
            }
            Action::Dirs => {
                let config_dir = dir(false)?.display().to_string();
                let logs_dir = crate::logging::config::dir(false)?.display().to_string();
                let plugins_dir = crate::plugins::config::dir(false)?.display().to_string();
                let value = serde_json::json!({
                    "config": config_dir,
                    "logs": logs_dir,
                    "plugins": plugins_dir
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
        let path = path()?;
        assert!(path.starts_with(std::env::temp_dir()));
        Ok(())
    }

    #[test]
    fn test_json_pointer() {
        assert_eq!(json_pointer("a"), "/a");
        assert_eq!(json_pointer("a/b"), "/a/b");
        assert_eq!(json_pointer("/a.b"), "/a/b");
        assert_eq!(json_pointer("a.b.c"), "/a/b/c");
    }

    #[test]
    fn test_display() -> Result<()> {
        let config = Config {
            ..Default::default()
        };

        let _all = get(&config, None)?;
        //assert!(all.contains("[serve]\n"));
        //assert!(all.contains("[upgrade]\n"));

        let _serve = get(&config, Some("serve".to_string()))?;
        //assert!(!serve.contains("[serve]\n"));
        //assert!(serve.contains("url = "));

        let _upgrade = get(&config, Some("upgrade".to_string()))?;
        //assert!(!upgrade.contains("[upgrade]\n"));
        //assert!(upgrade.contains("auto = "));

        assert_eq!(
            get(&config, Some("foo.bar".to_string()))
                .unwrap_err()
                .to_string(),
            "No configuration value at pointer: foo.bar"
        );

        Ok(())
    }

    #[test]
    fn test_set() -> Result<()> {
        let config = Config {
            ..Default::default()
        };

        let result = set(&config, "upgrade.auto", "off")?;
        assert_eq!(result.upgrade.auto, "off");

        let result = set(&config, "upgrade.verbose", "true")?;
        assert_eq!(result.upgrade.verbose, true);

        assert_eq!(
            set(&config, "foo.bar", "baz").unwrap_err().to_string(),
            "No configuration value at pointer: foo.bar"
        );

        Ok(())
    }

    #[test]
    fn test_reset() -> Result<()> {
        let config = serde_json::from_value(json!({
            "upgrade": {
                "auto": "off",
                "verbose": true,
                "confirm": true,
            }
        }))?;

        let default = Config {
            ..Default::default()
        };

        let result = reset(&config, "upgrade")?;
        assert_eq!(result.upgrade.auto, default.upgrade.auto);
        assert_eq!(result.upgrade.verbose, default.upgrade.verbose);

        let result = reset(&config, "all")?;
        assert_eq!(result, default);

        assert_eq!(
            reset(&config, "foo").unwrap_err().to_string(),
            "No top level configuration property named: foo"
        );

        Ok(())
    }

    #[test]
    fn test_cli() -> Result<()> {
        use super::cli::*;

        let mut config = Config {
            ..Default::default()
        };

        run(
            Command {
                action: Action::Get(Get { pointer: None }),
            },
            &mut config,
        )?;

        run(
            Command {
                action: Action::Set(Set {
                    pointer: "upgrade.confirm".to_string(),
                    value: "true".to_string(),
                }),
            },
            &mut config,
        )?;

        run(
            Command {
                action: Action::Reset(Reset {
                    property: "upgrade".to_string(),
                }),
            },
            &mut config,
        )?;

        run(
            Command {
                action: Action::Dirs,
            },
            &mut config,
        )?;

        Ok(())
    }
}
