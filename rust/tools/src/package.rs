use crate::{
    command::{AsyncToolCommand, ToolStdio},
    packages::Renv,
    tool::Tool,
};
use common::eyre::{bail, Result};

/// Get a list of packages used by Stencila
pub fn list_packages() -> Vec<Box<dyn Package>> {
    vec![Box::new(Renv)]
}

/// Get a package by name
pub fn get_package(name: &str) -> Option<Box<dyn Package>> {
    list_packages()
        .into_iter()
        .find(|package| package.name() == name)
}

/// Ensure a package is installed, installing it if necessary
///
/// This is a convenience function that checks if the package is installed and
/// installs it if not, with proper error handling and progress output.
#[allow(clippy::print_stderr)]
pub async fn ensure_package(package: &dyn Package) -> Result<()> {
    let is_installed = match package.is_installed() {
        Some(mut cmd) => cmd
            .status()
            .await
            .map(|status| status.success())
            .unwrap_or_default(),
        None => false,
    };

    if !is_installed {
        eprintln!("📥 Installing {}...", package.name());
        if let Some(mut cmd) = package.install() {
            let status = cmd
                .stdout(ToolStdio::Inherit)
                .stderr(ToolStdio::Inherit)
                .status()
                .await?;
            if !status.success() {
                bail!("Failed to install {}", package.name());
            }
        } else {
            bail!("No install command available for {}", package.name());
        }
    }

    Ok(())
}

/// Trait for packages that are installed within runtime environments
///
/// Unlike `Tool` which represents standalone executables on PATH, `Package` represents
/// dependencies that are installed within specific runtime environments (like R packages,
/// Python packages, etc.). These packages are typically not available as standalone
/// executables but are libraries/modules within their respective runtimes.
pub trait Package: Sync + Send {
    /// The name of the package
    fn name(&self) -> &'static str;

    /// A URL for the package
    fn url(&self) -> &'static str;

    /// A description of the package
    fn description(&self) -> &'static str;

    /// Configuration files that indicate this package is needed
    fn config_files(&self) -> Vec<&'static str> {
        vec![]
    }

    /// The package manager [`Tool`] that can install this package
    fn package_manager(&self) -> Box<dyn Tool>;

    /// Get the command to check if the package is installed
    ///
    /// This default implementation delegates to the package manager
    /// but can be overridden if necessary.
    fn is_installed(&self) -> Option<AsyncToolCommand> {
        self.package_manager().is_package_installed(self.name())
    }

    /// Get the command to install this package
    ///
    /// Returns an AsyncToolCommand that will automatically handle environment detection
    /// and tool wrapping (mise, npm, etc.) when executed.
    ///
    /// This default implementation delegates to the package manager
    /// but can be overridden if necessary.
    fn install(&self) -> Option<AsyncToolCommand> {
        self.package_manager().install_package(self.name())
    }
}
