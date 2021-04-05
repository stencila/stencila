use crate::util;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use validator::Validate;

#[derive(Debug, Default, PartialEq, Clone, Deserialize, Serialize, Validate)]
pub struct Config {
    #[serde(default)]
    #[validate]
    pub logging: crate::logging::config::Config,

    #[serde(default)]
    #[validate]
    pub serve: crate::serve::config::Config,

    #[serde(default)]
    #[validate]
    pub plugins: crate::plugins::config::Config,

    #[serde(default)]
    #[validate]
    pub upgrade: crate::upgrade::config::Config,
}

const CONFIG_FILE: &str = "config.toml";

/// Get the path of the configuration file
#[tracing::instrument]
fn path() -> Result<PathBuf> {
    #[cfg(not(test))]
    return Ok(util::dirs::config(true)?.join(CONFIG_FILE));

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
fn write(config: &Config) -> Result<()> {
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

/// Display a config property
#[tracing::instrument]
pub fn display(config: &Config, pointer: Option<String>) -> Result<String> {
    match pointer {
        None => Ok(toml::to_string(config)?),
        Some(pointer) => {
            let config = serde_json::to_value(config)?;
            if let Some(part) = config.pointer(json_pointer(&pointer).as_str()) {
                let json = serde_json::to_string(part)?;
                let part: toml::Value = serde_json::from_str(&json)?;
                let toml = toml::to_string(&part)?;
                Ok(toml)
            } else {
                bail!("No configuration value at pointer: {}", pointer)
            }
        }
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
    if let Err(errors) = config.validate() {
        bail!(
            "Invalid configuration value/s:\n\n{}",
            serde_json::to_string_pretty(&errors)?
        )
    }
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
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Manage configuration options",
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Args {
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

        #[structopt(about = "Get the directories used for config, cache etc")]
        Dirs,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(about = "Get configuration properties")]
    pub struct Get {
        /// A pointer to a config property e.g. `upgrade.auto`
        pub pointer: Option<String>,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Set configuration properties",
        setting = structopt::clap::AppSettings::TrailingVarArg,
        setting = structopt::clap::AppSettings::AllowLeadingHyphen
    )]
    pub struct Set {
        /// A pointer to a config property e.g. `upgrade.auto`
        pub pointer: String,

        /// The value to set the property to
        pub value: String,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(about = "Reset configuration properties to their defaults")]
    pub struct Reset {
        /// The config property to reset. Use 'all' to reset the entire config.
        pub property: String,
    }

    pub fn run(args: Args, config: &Config) -> Result<Config> {
        let Args { action } = args;
        match action {
            Action::Get(action) => {
                let Get { pointer } = action;
                println!("{}", super::display(config, pointer)?);
                Ok(config.clone())
            }
            Action::Set(action) => {
                let Set { pointer, value } = action;
                let config = super::set(config, &pointer, &value)?;
                write(&config)?;
                Ok(config)
            }
            Action::Reset(action) => {
                let Reset { property } = action;
                let config = super::reset(config, &property)?;
                write(&config)?;
                Ok(config)
            }
            Action::Dirs => {
                let config_dir = util::dirs::config(false)?.display().to_string();
                let plugins_dir = util::dirs::plugins(false)?.display().to_string();
                println!("config: {}\nplugins: {}", config_dir, plugins_dir);
                Ok(config.clone())
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

        let all = display(&config, None)?;
        assert!(all.contains("[serve]\n"));
        assert!(all.contains("[upgrade]\n"));

        let serve = display(&config, Some("serve".to_string()))?;
        assert!(!serve.contains("[serve]\n"));
        assert!(serve.contains("url = "));

        let upgrade = display(&config, Some("upgrade".to_string()))?;
        assert!(!upgrade.contains("[upgrade]\n"));
        assert!(upgrade.contains("auto = "));

        assert_eq!(
            display(&config, Some("foo.bar".to_string()))
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

        let config = Config {
            ..Default::default()
        };

        run(
            Args {
                action: Action::Get(Get { pointer: None }),
            },
            &config,
        )?;

        run(
            Args {
                action: Action::Set(Set {
                    pointer: "upgrade.confirm".to_string(),
                    value: "true".to_string(),
                }),
            },
            &config,
        )?;

        run(
            Args {
                action: Action::Reset(Reset {
                    property: "upgrade".to_string(),
                }),
            },
            &config,
        )?;

        run(
            Args {
                action: Action::Dirs,
            },
            &config,
        )?;

        Ok(())
    }
}
