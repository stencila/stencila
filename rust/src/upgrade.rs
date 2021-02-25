use anyhow::{anyhow, Result};
use std::env;
use std::thread;

/// Upgrade the application
///
/// Checks for a higher version on [GitHub releases](https://github.com/stencila/stencila/releases)
/// and downloads the binary for the current platform if one is found.
///
/// # Arguments
///
/// - `force`: Force the upgrade (fail on any errors)
/// - `confirm`: Prompt the user to confirm an upgrade
/// - `verbose`: Print information on the upgrade process
/// - `current_version`: The current version (used mainly for testing)
///
/// # Examples
///
/// ```
/// use stencila::upgrade::upgrade;
/// upgrade(false, true, false, None, None);
/// ```
pub fn upgrade(
    force: bool,
    confirm: bool,
    verbose: bool,
    wanted_version: Option<String>,
    current_version: Option<String>,
) -> Result<()> {
    // This is run in a separate thread so that is does not slow down the
    // command currently being run by the user (when doing an auto-update).
    // In any case it needs to be because `self_update` creates a new `tokio`
    // runtime (which can not be nested within our main `tokio` runtime).
    thread::spawn(move || -> Result<()> {
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

        match builder.build()?.update() {
            Ok(_) => Ok(()),
            Err(error) => {
                let message = error.to_string();
                if !force
                    && (message.contains("Update aborted")
                        || message.starts_with("ReqwestError")
                        || message.starts_with("NetworkError"))
                {
                    Ok(())
                } else {
                    Err(anyhow!("Error attempting to upgrade: {}", error))
                }
            }
        }
    })
    .join()
    .expect("Something went wrong")?;

    Ok(())
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

        /// Force upgrade (fail on any errors)
        #[structopt(short, long)]
        pub force: bool,

        /// Prompt the user to confirm an upgrade
        #[structopt(short, long)]
        pub confirm: bool,

        /// Print information on the upgrade process
        #[structopt(short, long)]
        pub verbose: bool,
    }

    pub fn upgrade(args: Args) -> Result<()> {
        let Args {
            to,
            force,
            confirm,
            verbose,
        } = args;

        let current_version = if force {
            Some("0.0.0".to_string())
        } else {
            None
        };
        super::upgrade(force, confirm, verbose, to, current_version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note that these tests are a little funky in that they make requests to the
    // GitHub API. They use an unrealistically high version to avoid any binaries
    // from being downloaded.

    #[test]
    fn test_upgrade() -> Result<()> {
        upgrade(false, false, false, None, Some("100.0.0".to_string()))
    }

    #[test]
    fn test_cli() -> Result<()> {
        cli::upgrade(cli::Args {
            to: None,
            force: false,
            confirm: false,
            verbose: false,
        })
    }
}
