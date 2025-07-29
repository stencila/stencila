use cli_utils::message;
use common::{
    clap::{self, Args},
    eyre::{Result, bail},
    tokio::fs::{create_dir_all, remove_dir_all, write},
    toml, tracing,
};

use crate::{MANIFEST_FILENAME, Plugin};

/// Install a plugin
#[tracing::instrument]
pub async fn install(name: &str) -> Result<Plugin> {
    tracing::debug!("Installing plugin `{name}`");

    // Get the latest manifest for the plugin
    let plugin = if name.starts_with("http://") || name.starts_with("https://") {
        Plugin::fetch_manifest(name).await?
    } else {
        let registry = Plugin::fetch_registry().await?;
        let Some(url) = registry.get(name) else {
            bail!("Plugin `{name}` not in registry");
        };
        Plugin::fetch_manifest_with(name, url).await?
    };

    // If the plugin directory already exists then uninstall it
    let dir = Plugin::plugin_dir(&plugin.name, false)?;
    if dir.exists() {
        remove_dir_all(&dir).await?;
    }

    // Do the install using the first compatible runtime
    for (runtime, version_req) in &plugin.runtimes {
        if !runtime.is_available(None) {
            continue;
        }

        // Check that runtime version requirement for the plugin is met
        let runtime_version = runtime.version(None)?;
        if !version_req.matches(&runtime_version) {
            bail!(
                "Unable to install plugin `{name}`: it requires {runtime}{version_req} but only {runtime_version} is available"
            )
        }

        // Make sure the directory is present
        create_dir_all(&dir).await?;

        // Dispatch to the runtime to do the installation
        runtime.install(&plugin.install, &dir).await?;
    }

    // Write the manifest into the dir
    // Do this last, when everything else has succeeded, because if this
    // file is present the plugin is assumed to be installed
    let manifest = dir.join(MANIFEST_FILENAME);
    write(&manifest, toml::to_string(&plugin)?).await?;

    Ok(plugin)
}

/// Install a plugin
#[derive(Debug, Default, Args)]
pub struct Install {
    /// The name or URL of the plugin to install
    ///
    /// If a URL is supplied it should be a URL to the manifest TOML file of the plugin.
    /// e.g. https://example.org/plugin/stencila-plugin.toml
    pub name: String,
}

impl Install {
    pub async fn run(self) -> Result<()> {
        let plugin = install(&self.name).await?;

        message!(
            "🚀 Successfully installed plugin `{}` version `{}`",
            plugin.name,
            plugin.version
        );

        Ok(())
    }
}
