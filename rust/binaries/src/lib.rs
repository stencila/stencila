//! Utilities for managing and calling external binaries

use binary::{
    eyre::{bail, Result},
    Binary, BinaryInstallation, BinaryTrait,
};
use once_cell::sync::Lazy;
use std::collections::{BTreeMap, HashMap};
use tokio::sync::RwLock;

///! A module for locating, running and installing third party binaries.
///!
///! Binaries may be used as runtimes for plugins (e.g. Node.js, Python) or
///! are used in created (e.g Pandoc by the `codec-pandoc` crate).

/// A global store of binaries
///
/// This is an immutable, lazily initialized list of "registered" binaries
/// that Stencila knows how to install, get the version for etc. However,
/// the functions below can be used for any other binary as well.
static BINARIES: Lazy<BTreeMap<String, Box<dyn BinaryTrait>>> = Lazy::new(|| {
    let mut map: BTreeMap<String, Box<dyn BinaryTrait>> = BTreeMap::new();

    #[cfg(feature = "binary-chrome")]
    {
        let bin = binary_chrome::ChromeBinary {};
        map.insert(bin.spec().name, Box::new(bin));
    }

    #[cfg(feature = "binary-node")]
    {
        let bin = binary_node::NodeBinary {};
        map.insert(bin.spec().name, Box::new(bin));
    }

    #[cfg(feature = "binary-pandoc")]
    {
        let bin = binary_pandoc::PandocBinary {};
        map.insert(bin.spec().name, Box::new(bin));
    }

    #[cfg(feature = "binary-python")]
    {
        let bin = binary_python::PythonBinary {};
        map.insert(bin.spec().name, Box::new(bin));
    }

    map
});

/// A cache of installations used to memoize calls to `installation`.
static INSTALLATIONS: Lazy<RwLock<HashMap<String, BinaryInstallation>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Get an installation
///
/// This is a relatively expensive function, even if the binary is already installed,
/// because it searches the file system and executes commands to get their version.
/// Therefore, this function memoizes installations in `INSTALLATIONS` for each `name` and `semver`.
/// Each cached result is removed if the binary is installed or uninstalled.
pub async fn installation(name: &str, semver: &str) -> Result<BinaryInstallation> {
    let name_semver = [name, "@", semver].concat();

    if let Some(installation) = INSTALLATIONS.read().await.get(&name_semver) {
        return Ok(installation.clone());
    }

    let unregistered: Box<dyn BinaryTrait> = Box::new(Binary::new(name, &[], &[]));
    let binary = BINARIES.get(name).unwrap_or(&unregistered);

    let semver = if semver == "*" {
        None
    } else {
        Some(semver.into())
    };

    if let Some(installation) = binary.installed(semver)? {
        INSTALLATIONS
            .write()
            .await
            .insert(name_semver, installation.clone());
        Ok(installation)
    } else {
        bail!("No matching installation found")
    }
}

/// Is a binary installation meeting semantic versioning requirements installed?
pub async fn installed(name: &str, semver: &str) -> bool {
    installation(name, semver).await.is_ok()
}

/// Get a binary installation meeting semantic versioning requirements.
///
/// If the binary is already available, or automatic installs are configured, returns
/// a `BinaryInstallation` that can be used to run commands. Otherwise, errors
/// with a message that the required binary is not yet installed, or failed to install.
pub async fn require(name: &str, semver: &str) -> Result<BinaryInstallation> {
    if let Ok(installation) = installation(name, semver).await {
        return Ok(installation);
    }

    // TODO: Use an env var to set this?
    let auto = true;
    if auto {
        let name_semver = [name, "@", semver].concat();
        let semver = if semver == "*" {
            None
        } else {
            Some(semver.into())
        };

        let binary = match BINARIES.get(name) {
            Some(binary) => binary,
            None => bail!("Unable to automatically install binary `{}`", name),
        };
        binary.install(semver.clone(), None, None).await?;

        if let Some(installation) = binary.installed(semver)? {
            let mut installations = INSTALLATIONS.write().await;
            installations.insert(name_semver, installation.clone());
            Ok(installation)
        } else {
            bail!("Failed to automatically install binary `{}`", name)
        }
    } else {
        bail!("Required binary `{}` is not installed", name)
    }
}

#[cfg(feature = "cli")]
pub mod commands {
    use super::*;
    use cli_utils::structopt::StructOpt;
    use cli_utils::{async_trait::async_trait, result, Result, Run};

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Manage helper binaries",
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
        Show(Show),
        Installable(Installable),
        Install(Install),
        Uninstall(Uninstall),
        Run(Run_),
    }

    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            let Self { action } = self;
            match action {
                Action::Show(action) => action.run().await,
                Action::Installable(action) => action.run().await,
                Action::Install(action) => action.run().await,
                Action::Uninstall(action) => action.run().await,
                Action::Run(action) => action.run().await,
            }
        }
    }

    /// Show information on a binary
    ///
    /// Searches for the binary on your path and in Stencila's "binaries"
    /// folder for versions that are installed. Use the `semver` argument
    /// to show the latest version that meets the semantic version requirement.
    ///
    /// This command should find any binary that is on your PATH
    /// (i.e. including those not in the `stencila binaries installable` list).
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Show {
        /// The name of the binary e.g. pandoc
        pub name: String,

        /// The semantic version requirement for the binary e.g. >2
        ///
        /// If this is supplied and the binary does not meet the semver
        /// requirement nothing will be shown
        pub semver: Option<String>,
    }

    #[async_trait]
    impl Run for Show {
        async fn run(&self) -> Result {
            // Try to get registered binary (because has potential aliases and extracting versions) but fall
            // back to unregistered for others
            let unregistered: Box<dyn BinaryTrait> = Box::new(Binary::new(&self.name, &[], &[]));
            let binary = BINARIES.get(&self.name).unwrap_or(&unregistered);
            if self.semver.is_some() {
                if let Ok(installation) = binary.installed(self.semver.clone()) {
                    result::value(installation)
                } else {
                    tracing::info!(
                        "No matching binary found. Perhaps use `stencila binaries install`?"
                    );
                    result::nothing()
                }
            } else {
                let installations = binary.installations();
                result::value(installations)
            }
        }
    }

    /// List binaries (and their versions) that can be installed using Stencila
    ///
    /// The returned list is a list of the binaries/versions that Stencila knows how to install.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Installable {}

    #[async_trait]
    impl Run for Installable {
        async fn run(&self) -> Result {
            let list: Vec<serde_json::Value> = BINARIES
                .values()
                .map(|binary| {
                    let spec = binary.spec();
                    serde_json::json!({
                        "name": spec.name,
                        "versions": spec.installable
                    })
                })
                .collect();
            result::value(list)
        }
    }

    /// Install a binary
    ///
    /// Installs the latest version of the binary, that also meets any
    /// semantic version requirement supplied, into the Stencila "binaries"
    /// folder.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Install {
        /// The name of the binary (must be a registered binary name)
        pub name: String,

        /// The semantic version requirement (the most recent matching version will be installed)
        pub semver: Option<String>,

        /// The operating system to install for (defaults to the current)
        #[structopt(short, long, possible_values = &OS_VALUES )]
        pub os: Option<String>,

        /// The architecture to install for (defaults to the current)
        #[structopt(short, long, possible_values = &ARCH_VALUES)]
        pub arch: Option<String>,
    }

    const OS_VALUES: [&str; 3] = ["macos", "windows", "linux"];
    const ARCH_VALUES: [&str; 3] = ["x86", "x86_64", "arm"];

    #[async_trait]
    impl Run for Install {
        async fn run(&self) -> Result {
            match BINARIES.get(&self.name) {
                Some(binary) => {
                    binary
                        .install(self.semver.clone(), self.os.clone(), self.arch.clone())
                        .await?;
                    tracing::info!("📦 Installed {}", self.name);
                }
                None => {
                    tracing::error!("Stencila is unable to install `{}`", self.name);
                }
            }
            result::nothing()
        }
    }

    /// Uninstall a binary
    ///
    /// Removes the binary (optionally, just a specific version) from the Stencila
    /// "binaries" folder. No other installations of the binary on the system will
    /// will be removed.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Uninstall {
        /// The name of the binary (must be a registered binary name)
        pub name: String,

        /// The specific version of the binary to uninstall
        ///
        /// If this is not provided, all versions will be removed.
        pub version: Option<String>,
    }
    #[async_trait]
    impl Run for Uninstall {
        async fn run(&self) -> Result {
            // Fallback to unregistered since that is sufficient for uninstall
            let unregistered: Box<dyn BinaryTrait> = Box::new(Binary::new(&self.name, &[], &[]));
            let binary = BINARIES.get(&self.name).unwrap_or(&unregistered);
            binary.uninstall(self.version.clone()).await?;

            tracing::info!("🗑️ Uninstalled {}", self.name);
            result::nothing()
        }
    }

    /// Run a command using a binary
    ///
    /// Pass arguments and options to the binary after the `--` flag.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Run_ {
        /// The name of the binary e.g. node
        pub name: String,

        /// The semantic version requirement e.g. 16
        pub semver: Option<String>,

        /// The arguments and options to pass to the binary
        #[structopt(raw(true))]
        pub args: Vec<String>,
    }

    #[async_trait]
    impl Run for Run_ {
        async fn run(&self) -> Result {
            let installation = require(
                &self.name,
                &self.semver.clone().unwrap_or_else(|| "*".to_string()),
            )
            .await?;
            let output = installation.run(&self.args).await?;

            use std::io::Write;
            std::io::stdout().write_all(output.stdout.as_ref())?;
            std::io::stderr().write_all(output.stderr.as_ref())?;

            result::nothing()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // End to end CLI test that install, show and uninstall
    // the latest version of each binary. Intended as a coarse
    // tests of installation of each binary. These tests are
    // tagged with #[ignore] because they are slow, so in development
    // you don't want to run them, and because if they are run in
    // parallel with other tests that use `require()` they can cause deadlocks
    // and other on-disk conflicts.

    // Run this test at the start of CI tests using
    //   cargo test binaries::tests::install -- --ignored --nocapture
    #[cfg(feature = "cli")]
    #[tokio::test]
    #[ignore]
    async fn install() -> Result<()> {
        use super::commands::{Install, Installable, Show};
        use cli_utils::Run;
        use test_utils::assert_json_eq;

        Installable {}.run().await?;

        for name in BINARIES.keys() {
            eprintln!("Testing {}", name);

            Install {
                name: name.clone(),
                semver: None,
                os: None,
                arch: None,
            }
            .run()
            .await?;

            let display = Show {
                name: name.clone(),
                semver: None,
            }
            .run()
            .await?;

            let value = if let Some(value) = display.value {
                value
            } else {
                bail!("Expected value")
            };
            assert_json_eq!(value.get("name"), Some(name.clone()));
            assert!(!value
                .get("installs")
                .expect("To have installs")
                .as_array()
                .expect("To be array")
                .is_empty());
        }

        Ok(())
    }

    // Run this test at the end of CI tests using
    //   cargo test binaries::tests::uninstall -- --ignored --nocapture
    #[cfg(feature = "cli")]
    #[tokio::test]
    #[ignore]
    async fn uninstall() -> Result<()> {
        use super::commands::Uninstall;
        use cli_utils::Run;

        for name in BINARIES.keys() {
            Uninstall {
                name: name.clone(),
                version: None,
            }
            .run()
            .await?;
        }

        Ok(())
    }
}
