use crate::cli::Command;
use crate::util;
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;

/// Get configuration options for a command or all commands
pub fn get(command: &str) -> Result<String> {
    let lines = read()?;

    if command == "all" {
        return Ok(lines.join("\n"));
    }

    for line in lines {
        if line.starts_with(command) {
            return Ok(line.strip_prefix(command).unwrap().trim().to_string());
        }
    }

    Ok("".to_string())
}

/// Set configuration options for a command
pub fn set(command: &str, args: &[String]) -> Result<()> {
    let mut lines = read()?;

    let mut found = false;
    for (index, line) in lines.iter().enumerate() {
        if line.starts_with(command) {
            // Merge existing and new args
            let config = line.strip_prefix(command).unwrap().trim().to_string();
            let merged = merge(command, &config, &args)?;
            lines[index] = format!("{} {}", command, merged.join(" "));
            found = true;
            break;
        }
    }

    if !found {
        // If not found, still call merge to ensure args are checked
        // and in a consistent format
        let merged = merge(command, "", &args)?;
        let line = format!("{} {}", command, merged.join(" "));
        lines.push(line)
    }

    write(lines)
}

/// Clear configuration options for a command or all commands
pub fn clear(command: &str) -> Result<()> {
    if command == "all" {
        write(vec![])
    } else {
        let mut lines = read()?;
        for (index, line) in lines.iter().enumerate() {
            if line.starts_with(command) {
                lines.remove(index);
                break;
            }
        }
        write(lines)
    }
}

/// Merge args into an existing configuration
pub fn merge(command: &str, config: &str, args: &[String]) -> Result<Vec<String>> {
    // Get a `clap::App` for the command that will be used to
    // validate the args being set and merge values with existing one
    let app = crate::cli::Args::clap();

    // Extract message from a `clap` error (excludes usage etc)
    fn arg_error_message(error: &structopt::clap::Error) -> String {
        let start = match error.message.find("error: ") {
            Some(found) => found + 7,
            None => 0,
        };
        let end = error
            .message
            .find("\n\n")
            .unwrap_or_else(|| error.message.len());
        error.message.as_str()[start..end].to_string()
    }

    // Check that the existing config is valid. If it is not, ignore it with warning.
    let config_matches = match app.clone().get_matches_from_safe(
        format!("stencila {} {}", command, config)
            .trim()
            .split_whitespace(),
    ) {
        Ok(matches) => matches,
        Err(error) => {
            tracing::warn!(
                "Existing config for {} is invalid and will be ignored: {}. Use `stencila config clear {}` or edit {}.",
                command, arg_error_message(&error), command, path()?.to_string_lossy()
            );
            structopt::clap::ArgMatches::new()
        }
    };

    // Parse the args, this will exit the process if there are errors or --help was used.
    let args_matches =
        app.get_matches_from([&["stencila".to_string(), command.to_string()], args].concat());

    // Do merge
    let mut args_map = HashMap::<&str, String>::new();
    if let Some(config) = config_matches.subcommand {
        for (name, arg) in config.matches.args {
            let value = if arg.vals.is_empty() {
                "".to_string()
            } else {
                format!("={}", arg.vals[0].to_str().unwrap_or_default())
            };
            args_map.insert(name, value.to_string());
        }
    }
    if let Some(args) = args_matches.subcommand {
        for (name, arg) in args.matches.args {
            let value = if arg.vals.is_empty() {
                "".to_string()
            } else {
                format!("={}", arg.vals[0].to_str().unwrap_or_default())
            };
            args_map.insert(name, value.to_string());
        }
    }

    // Create new vector or ags, sorted for determinism
    let mut args_vec: Vec<String> = args_map
        .iter()
        .map(|(name, value)| format!("--{}{}", name, value))
        .collect();
    args_vec.sort();

    Ok(args_vec)
}

/// Get the path of the configuration file
fn path() -> Result<PathBuf> {
    #[cfg(not(test))]
    return Ok(util::dirs::config(true)?.join("cli.txt"));

    // When running tests, avoid messing with users existing config
    #[cfg(test)]
    {
        let path = std::env::temp_dir()
            .join("stencila")
            .join("test")
            .join("cli.txt");
        fs::create_dir_all(path.parent().unwrap())?;
        return Ok(path);
    }
}

/// Read lines from the configuration file
fn read() -> Result<Vec<String>> {
    let config_file = path()?;
    let lines = fs::read_to_string(config_file)
        .unwrap_or_else(|_| "".to_string())
        .lines()
        .map(|line| line.to_string())
        .collect();
    Ok(lines)
}

/// Write lines to the configuration file
fn write(lines: Vec<String>) -> Result<()> {
    let config_file = path()?;
    let mut file = fs::File::create(config_file)?;
    file.write_all(lines.join("\n").as_bytes())?;
    file.sync_all()?;
    Ok(())
}

/// CLI options for the `config` command
#[cfg(feature = "cli")]
pub mod cli {
    use super::*;
    use structopt::StructOpt;
    use strum::VariantNames;

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
        Clear(Clear),

        #[structopt(about = "Get the directories used for config, cache etc")]
        Dirs,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(about = "Get configuration options for a command")]
    pub struct Get {
        /// The command to get the config for. Use 'all' to get config for all commands.
        #[structopt(possible_values = [Command::VARIANTS, &["all"]].concat().as_slice())]
        pub command: String,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Set configuration options for a command",
        setting = structopt::clap::AppSettings::TrailingVarArg,
        setting = structopt::clap::AppSettings::AllowLeadingHyphen
    )]
    pub struct Set {
        /// The command to set the config for.
        #[structopt(possible_values = &Command::VARIANTS)]
        pub command: String,

        /// The options and flags to set for the command
        #[structopt(multiple = true)]
        pub args: Vec<String>,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(about = "Clear configuration options for a command")]
    pub struct Clear {
        /// The command to clear the config for. Use 'all' to clear the config for all commands.
        #[structopt(possible_values = [Command::VARIANTS, &["all"]].concat().as_slice())]
        pub command: String,
    }

    pub fn config(args: Args) -> Result<()> {
        let Args { action } = args;
        match action {
            Action::Get(action) => {
                let Get { command } = action;
                let config = super::get(&command)?;
                println!("{}", config)
            }
            Action::Set(action) => {
                let Set { command, args } = action;
                super::set(&command, &args)?;
            }
            Action::Clear(action) => {
                let Clear { command } = action;
                super::clear(&command)?;
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
    fn test_merge() -> Result<()> {
        assert_eq!(merge("serve", "", &[])?, Vec::<String>::new());

        assert_eq!(
            merge("serve", "", &["--key=bar".to_string()])?,
            vec!["--key=bar".to_string()]
        );

        assert_eq!(
            merge("serve", "--key=bar", &["--key=baz".to_string()])?,
            vec!["--key=baz".to_string()]
        );

        assert_eq!(
            // Bad existing config just gets ignored
            merge("serve", "--bad", &[])?,
            Vec::<String>::new()
        );

        //merge("serve", "", &["--bad".to_string()]);

        Ok(())
    }

    #[test]
    fn test_set_get_clear() -> Result<()> {
        // This is a single test because otherwise there can
        // be conflicting writes to the test config file if
        // tests are run in parallel.

        clear("all")?;
        assert_eq!(get("all")?, "");

        set("serve", &vec!["--key=foo".to_string()])?;
        assert_eq!(get("serve")?, "--key=foo");

        set("serve", &vec!["--insecure".to_string()])?;
        assert_eq!(get("serve")?, "--insecure --key=foo");

        set("serve", &vec!["--key=bar".to_string()])?;
        assert_eq!(get("serve")?, "--insecure --key=bar");

        clear("serve")?;
        assert_eq!(get("serve")?, "");

        cli::config(cli::Args {
            action: cli::Action::Get(cli::Get {
                command: "serve".to_string(),
            }),
        })?;

        cli::config(cli::Args {
            action: cli::Action::Set(cli::Set {
                command: "serve".to_string(),
                args: vec![],
            }),
        })?;

        cli::config(cli::Args {
            action: cli::Action::Clear(cli::Clear {
                command: "serve".to_string(),
            }),
        })?;

        cli::config(cli::Args {
            action: cli::Action::Dirs,
        })
    }
}
