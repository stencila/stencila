use common::{
    clap::{self, Args},
    eyre::{bail, Result},
    tokio::fs::{create_dir_all, remove_dir_all, write},
    toml, tracing,
};

use crate::Plugin;

/// Install a plugin
#[tracing::instrument]
pub async fn install(name: &str) -> Result<()> {
    tracing::debug!("Installing plugin `{name}`");

    // Get the latest manifest for the plugin
    let registry = Plugin::fetch_registry().await?;
    let Some(url) = registry.get(name) else {
        bail!("Plugin `{name}` not in registry");
    };
    let plugin = Plugin::fetch_manifest(name, url).await?;

    // If the plugin directory already exists then uninstall it
    let dir = Plugin::plugin_dir(name, true)?;
    if dir.exists() {
        remove_dir_all(&dir).await?;
        create_dir_all(&dir).await?;
    }

    // Do the install using the first compatible runtime
    for (runtime, version_req) in &plugin.runtimes {
        if !runtime.is_available() {
            continue;
        }

        // Check that runtime version requirement for the plugin is met
        let runtime_version = runtime.version()?;
        if !version_req.matches(&runtime_version) {
            bail!("Unable to install plugin `{name}`: it requires {runtime}{version_req} but only {runtime_version} is available")
        }

        // Dispatch to the runtime to do the installation
        runtime.install(&plugin.install, &dir).await?;
    }

    // Write the manifest into the dir
    // Do this last, when everything else has succeeded, because if this
    // file is present the plugin is assumed to be installed
    let manifest = dir.join("manifest.toml");
    write(&manifest, toml::to_string(&plugin)?).await?;

    tracing::info!(
        "🚀 Successfully installed plugin `{}` version `{}`",
        plugin.name,
        plugin.version,
    );

    Ok(())
}

#[derive(Debug, Default, Args)]
pub struct InstallArgs {
    /// The name of the plugin to install
    pub name: String,
}
