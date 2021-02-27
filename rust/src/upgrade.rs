use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use std::fs;
use std::thread;
use structopt::StructOpt;

/// Upgrade the application
///
/// Checks for a higher version on [GitHub releases](https://github.com/stencila/stencila/releases)
/// and downloads the binary for the current platform if one is found.
///
/// # Arguments
///
/// - `current_version`: The current version (used mainly for testing)
/// - `wanted_version`: The version that is wanted (other than latest)
/// - `confirm`: Prompt the user to confirm an upgrade
/// - `verbose`: Print information on the upgrade process
pub fn upgrade(
    current_version: Option<String>,
    wanted_version: Option<String>,
    confirm: bool,
    verbose: bool,
) -> Result<()> {
    let mut builder = self_update::backends::github::Update::configure();
    builder
        .repo_owner("stencila")
        .repo_name("stencila")
        .bin_name("stencila")
        .current_version(
            current_version
                .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string())
                .as_str(),
        )
        .no_confirm(!confirm)
        .show_output(verbose)
        .show_download_progress(verbose);

    if let Some(version) = wanted_version {
        builder.target_version_tag(format!("v{}", version).as_str());
    }

    // Fail silently unless `verbose` is true.
    match builder.build()?.update() {
        Ok(_) => {}
        Err(error) => {
            let message = error.to_string();
            if !message.contains("Update aborted") && verbose {
                println!("Error attempting to upgrade: {}", message)
            }
        }
    };

    Ok(())
}

/// Do a upgrade check automatically if not within the configured interval
/// since the last check.
///
/// Runs in a separate thread so that is does not slow down the
/// command currently being run by the user.
pub fn upgrade_auto() -> std::thread::JoinHandle<Result<()>> {
    thread::spawn(move || -> Result<()> {
        // Get the upgrade configuration
        let config = crate::config::get("upgrade")?;
        let cli::Args {
            auto,
            confirm,
            verbose,
            ..
        } = cli::Args::from_iter(format!("upgrade {}", config).split_whitespace());

        // Go no further if auto upgrade is not enabled
        let auto = auto.unwrap_or_else(|| "1 day".to_string());
        if auto == "off" {
            return Ok(());
        }

        // Check if within the time since the last check was done
        let upgrade_file = crate::util::dirs::config(true)?.join("cli-upgrade.txt");
        let last = match fs::read_to_string(upgrade_file.clone()) {
            Ok(date) => DateTime::parse_from_rfc3339(date.as_str())?.with_timezone(&Utc),
            Err(_) => Utc::now(),
        };
        let duration = Duration::from_std(humantime::parse_duration(auto.as_str())?)?;
        if Utc::now() < last + duration {
            return Ok(());
        }

        // Attempt an upgrade
        upgrade(None, None, confirm, verbose)?;

        // Record the time of the upgrade check, so another check
        // is not made within the `auto`.
        let now = Utc::now();
        fs::write(upgrade_file, now.to_rfc3339())?;

        Ok(())
    })
}

/// CLI options for the `upgrade` command
#[cfg(feature = "cli")]
pub mod cli {
    use super::*;
    use structopt::StructOpt;
    #[derive(Debug, StructOpt)]
    #[structopt(about = "Upgrade stencila to the latest version")]
    pub struct Args {
        /// Version to upgrade (or downgrade) to (defaults to the latest)
        #[structopt(short, long)]
        pub to: Option<String>,

        /// Prompt the user to confirm an upgrade
        #[structopt(short, long)]
        pub confirm: bool,

        /// Print information on the upgrade process
        #[structopt(short, long)]
        pub verbose: bool,

        /// The interval between automatic upgrade checks (defaults to "1 day").
        /// Only used when for configuration. Set to "off" for no automatic checks.
        #[structopt(short, long, validator = validate_auto)]
        pub auto: Option<String>,
    }

    /// Validate that a string is a valid duration or "off"
    pub fn validate_auto(value: String) -> Result<(), String> {
        if value == *"off" {
            return Ok(());
        }
        if let Err(error) = humantime::parse_duration(value.as_str()) {
            return Err(error.to_string());
        }
        Ok(())
    }

    pub fn upgrade(args: Args) -> Result<()> {
        let Args {
            to,
            confirm,
            verbose,
            ..
        } = args;

        // This is run in a separate thread because `self_update` creates a new `tokio`
        // runtime (which can not be nested within our main `tokio` runtime).
        thread::spawn(move || -> Result<()> { super::upgrade(None, to, confirm, verbose) })
            .join()
            .map_err(|_| anyhow!("Error joining thread"))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note that these tests are a little funky in that they make requests to the
    // GitHub API. They currently also have few assertions.
    // They use an artificially high `current_version` to avoid any binaries
    // from being downloaded.

    #[test]
    fn test_upgrade() -> Result<()> {
        upgrade(Some("100.0.0".to_string()), None, false, false)
    }

    #[test]
    fn test_upgrade_auto() -> Result<()> {
        upgrade_auto().join().expect("Failed")
    }

    #[test]
    fn test_cli() -> Result<()> {
        cli::upgrade(cli::Args {
            to: None,
            confirm: false,
            verbose: false,
            auto: None,
        })
    }

    #[test]
    fn test_validate_auto() {
        assert_eq!(cli::validate_auto("off".to_string()), Ok(()));
        assert_eq!(cli::validate_auto("1 day".to_string()), Ok(()));
        assert_eq!(
            cli::validate_auto("2 weeks 3 days 1 hr".to_string()),
            Ok(())
        );

        assert_eq!(
            cli::validate_auto("foo".to_string()),
            Err("expected number at 0".to_string())
        );
    }
}
