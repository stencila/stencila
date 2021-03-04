use crate::util;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use validator::Validate;

#[derive(Debug, PartialEq, Deserialize, Serialize, Validate)]
pub struct Config {
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

impl Default for Config {
    fn default() -> Self {
        Config {
            serve: Default::default(),
            plugins: Default::default(),
            upgrade: Default::default(),
        }
    }
}

const CONFIG_FILE: &str = "config.toml";

/// Get the path of the configuration file
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
        return Ok(path);
    }
}

/// Read the config from the configuration file
fn read() -> Result<Config> {
    let config_file = path()?;
    let content = fs::read_to_string(config_file).unwrap_or_else(|_| "".to_string());
    let config = toml::from_str(content.as_str())?;
    Ok(config)
}

/// Write the config to the configuration file
fn write(config: Config) -> Result<()> {
    let config_file = path()?;
    let mut file = fs::File::create(config_file)?;
    file.write_all(toml::to_string(&config)?.as_bytes())?;
    file.sync_data()?;
    Ok(())
}

/// Validate the config
pub fn validate(config: &Config) -> Result<()> {
    if let Err(error) = config.validate() {
        bail!(
            "Invalid configuration; use `config set`, `config reset` or edit {} to fix.\n\n{:#?}",
            path()?.display(),
            error
        )
    }
    Ok(())
}

/// Read and validate the config
pub fn get() -> Result<Config> {
    let config = read()?;
    validate(&config)?;
    Ok(config)
}

/// Ensure that a string is a valid JSON pointer
///
/// Replaces dots (`.`) with slashes (`/`) and ensures a
/// leading slash.
pub fn json_pointer(pointer: &str) -> String {
    let pointer = pointer.replace(".", "/");
    if pointer.starts_with('/') {
        pointer
    } else {
        format!("/{}", pointer)
    }
}

/// Display a config property
pub fn display(pointer: Option<String>) -> Result<String> {
    let config = get()?;
    match pointer {
        None => Ok(toml::to_string(&config)?),
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
pub fn set(pointer: String, value: String) -> Result<()> {
    // Use `read` to avoid validation (user may fix any errors and we
    // validate below anyway).
    let config = read()?;

    let mut config = serde_json::to_value(config)?;
    if let Some(property) = config.pointer_mut(json_pointer(&pointer).as_str()) {
        let value = match serde_json::from_str(&value) {
            Ok(value) => value,
            Err(_) => serde_json::Value::String(value),
        };
        *property = value;
    } else {
        bail!("No configuration value at pointer: {}", pointer)
    };

    let config: Config = serde_json::from_value(config)?;
    validate(&config)?;

    write(config)
}

/// Reset a config property
pub fn reset(property: String) -> Result<()> {
    let config = get()?;

    let config: Config = match property.as_str() {
        "all" => Default::default(),
        "serve" => Config {
            serve: Default::default(),
            ..config
        },
        "upgrade" => Config {
            upgrade: Default::default(),
            ..config
        },
        _ => bail!("No configuration property named: {}", property),
    };

    write(config)
}

/// CLI options for the `config` command
#[cfg(feature = "cli")]
pub mod cli {
    use super::*;
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Manage configuration options",
        setting = structopt::clap::AppSettings::DeriveDisplayOrder
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

    pub fn config(args: Args) -> Result<()> {
        let Args { action } = args;
        match action {
            Action::Get(action) => {
                let Get { pointer } = action;
                let config = super::display(pointer)?;
                println!("{}", config)
            }
            Action::Set(action) => {
                let Set { pointer, value } = action;
                super::set(pointer, value)?;
            }
            Action::Reset(action) => {
                let Reset { property } = action;
                super::reset(property)?;
            }
            Action::Dirs => {
                let config = util::dirs::config(false)?.display().to_string();
                let plugins = util::dirs::plugins(false)?.display().to_string();
                println!("config: {}\nplugins: {}", config, plugins);
            }
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
        let all = display(None)?;
        assert!(all.contains("[serve]\n"));
        assert!(all.contains("[upgrade]\n"));

        let serve = display(Some("serve".to_string()))?;
        assert!(!serve.contains("[serve]\n"));
        assert!(serve.contains("url = "));

        let upgrade = display(Some("upgrade".to_string()))?;
        assert!(!upgrade.contains("[upgrade]\n"));
        assert!(upgrade.contains("auto = "));

        assert_eq!(
            display(Some("foo.bar".to_string()))
                .unwrap_err()
                .to_string(),
            "No configuration value at pointer: foo.bar"
        );

        Ok(())
    }

    #[test]
    fn test_get_set_reset() -> Result<()> {
        // Do all this in one test to avoid individual tests
        // clobbering each other

        let default = Config {
            ..Default::default()
        };

        reset("all".to_string())?;
        assert_eq!(get()?, default);

        set("upgrade.auto".to_string(), "off".to_string())?;
        assert_eq!(get()?.upgrade.auto, "off");

        set("upgrade.verbose".to_string(), "true".to_string())?;
        assert_eq!(get()?.upgrade.verbose, true);

        assert_eq!(
            set("foo.bar".to_string(), "baz".to_string())
                .unwrap_err()
                .to_string(),
            "No configuration value at pointer: foo.bar"
        );

        reset("upgrade".to_string())?;
        let upgrade = get()?.upgrade;
        assert_eq!(upgrade.auto, default.upgrade.auto);
        assert_eq!(upgrade.verbose, default.upgrade.verbose);

        reset("serve".to_string())?;

        assert_eq!(
            reset("foo".to_string()).unwrap_err().to_string(),
            "No configuration property named: foo"
        );

        use super::cli::*;

        config(Args {
            action: Action::Get(Get { pointer: None }),
        })?;

        config(Args {
            action: Action::Set(Set {
                pointer: "upgrade.confirm".to_string(),
                value: "true".to_string(),
            }),
        })?;

        config(Args {
            action: Action::Reset(Reset {
                property: "upgrade".to_string(),
            }),
        })?;

        config(Args {
            action: Action::Dirs,
        })?;

        Ok(())
    }
}
