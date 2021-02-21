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
/// - `confirm`: Prompt the user to confirm an upgrade
/// - `verbose`: Print information on the upgrade process
/// - `current_version`: The current version (used mainly for testing)
///
/// # Examples
///
/// ```
/// use stencila::upgrade::upgrade;
/// upgrade(true, false, None);
/// ```
pub fn upgrade(confirm: bool, verbose: bool, current_version: Option<String>) -> Result<()> {
    // This is run in a separate thread so that is does not slow down the
    // command currently being run by the user (when doing an auto-update).
    // In any case it needs to be because `self_update` creates a new `tokio`
    // runtime (which can not be nested within our main `tokio` runtime).
    thread::spawn(move || -> Result<()> {
        let result = self_update::backends::github::Update::configure()
            .repo_owner("stencila")
            .repo_name("stencila")
            .bin_name("stencila")
            .current_version(
                current_version
                    .unwrap_or(env!("CARGO_PKG_VERSION").to_string())
                    .as_str(),
            )
            .no_confirm(!confirm)
            .show_output(verbose)
            .show_download_progress(verbose)
            .build()?
            .update();
        match result {
            Ok(_) => Ok(()),
            Err(error) => {
                if format!("{}", error).contains("Update aborted") {
                    // If the user said no to the confirmation, that's `Ok`.
                    Ok(())
                } else {
                    Err(anyhow!(error))
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
        /// Prompt the user to confirm an upgrade
        #[structopt(short, long)]
        pub confirm: bool,

        /// Print information on the upgrade process
        #[structopt(short, long)]
        pub verbose: bool,
    }

    pub fn upgrade(args: Args) -> Result<()> {
        let Args { confirm, verbose } = args;
        super::upgrade(confirm, verbose, None)
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
        upgrade(false, false, Some("100.0.0".to_string()))
    }

    #[test]
    fn test_cli() -> Result<()> {
        cli::upgrade(cli::Args {
            confirm: false,
            verbose: false,
        })
    }
}
