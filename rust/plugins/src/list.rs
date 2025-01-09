use std::{
    fs::{self, read_dir},
    time::Duration,
};

use app::{get_app_dir, DirType};
use cli_utils::table::{self, Attribute, Cell, Color, Table};
use common::{
    clap::{self, Args},
    eyre::Result,
    futures::future,
    itertools::Itertools,
    serde_json,
    tokio::fs::{read_to_string, write},
    tracing,
};

use crate::{Plugin, PluginEnabled, PluginStatus};

/// The number of seconds before the cache of plugin manifests expires
const CACHE_EXPIRY_SECS: u64 = 6 * 60 * 60;

/// Get a list of plugins
///
/// Fetches the `plugins.toml` file at the root of the Stencila repository
/// and expands it into a list of plugin manifests by fetching each individual
/// manifest.
///
/// Caches the generated list of manifests. Use of cache can be overridden using
/// the `options.refresh`.
///
/// Filtering the list is possible, currently only using `options.installed`
/// (but in the future may allow for text matching "search" filtering)
pub async fn list(args: ListArgs) -> Result<Vec<Plugin>> {
    let plugins_dir = get_app_dir(DirType::Plugins, true)?;
    let cache = plugins_dir.join("manifests.json");

    let cache_has_expired = || {
        if let Ok(metadata) = fs::metadata(&cache) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    return elapsed > Duration::from_secs(CACHE_EXPIRY_SECS);
                }
            }
        }
        false
    };
    let plugins = if !args.refresh && cache.exists() && !cache_has_expired() {
        // If no errors reading or deserializing (e.g. due to change in fields in plugins) then
        // use cached list
        read_to_string(&cache)
            .await
            .ok()
            .and_then(|json| serde_json::from_str(&json).ok())
            .unwrap_or_default()
    } else {
        vec![]
    };

    let mut plugins = if plugins.is_empty() {
        tracing::debug!("Refreshing list of plugins and their manifests");

        // Fetch the plugins list from the Stencila repo
        let plugins = Plugin::fetch_registry().await?;

        // Fetch each of the plugin manifests in parallel, logging debug messages
        // on any errors (this avoids any one plugin with an invalid manifest from
        // breaking the entire fetch or alarming the user with a blaring warning message, while still
        // providing some visibility)
        let futures = plugins
            .iter()
            .map(|(name, url)| async move { Plugin::fetch_manifest_with(name, url).await });

        let plugins = future::join_all(futures)
            .await
            .into_iter()
            .filter_map(|result| match result {
                Ok(plugin) => Some(plugin),
                Err(error) => {
                    tracing::debug!("{error}");
                    None
                }
            })
            .collect_vec();

        // Write the list to cache
        tracing::debug!("Caching plugin manifests to {}", cache.display());
        write(cache, serde_json::to_string_pretty(&plugins)?).await?;

        plugins
    } else {
        plugins
    };

    // Add plugins in plugins directory that are not in the registry list
    // (i.e. installed from URL or using symlink)
    for entry in read_dir(plugins_dir)?
        .flatten()
        .filter(|entry| entry.path().is_dir() || entry.path().is_symlink())
    {
        let plugin_name = entry.file_name().to_string_lossy().to_string();
        let linked = entry.path().is_symlink();
        let unregistered = !plugins.iter().any(|plugin| plugin.name == plugin_name);

        if linked || unregistered {
            match Plugin::read_manifest(&plugin_name) {
                Ok(mut plugin) => {
                    plugin.unregistered = unregistered;
                    plugin.linked = linked;
                    plugins.push(plugin)
                }
                Err(error) => {
                    tracing::warn!("Error reading manifest for plugin `{plugin_name}`: {error}");
                    continue;
                }
            }
        }
    }

    // Filter according to args
    let plugins = plugins
        .into_iter()
        .filter(|plugin| {
            let (status, enabled) = plugin.availability();

            (!args.installed
                || matches!(
                    status,
                    PluginStatus::InstalledLatest(..) | PluginStatus::InstalledOutdated(..)
                ))
                && (!args.installable || matches!(status, PluginStatus::Installable))
                && (!args.outdated || matches!(status, PluginStatus::InstalledOutdated(..)))
                && (!args.enabled || matches!(enabled, PluginEnabled::Yes))
        })
        .collect_vec();

    Ok(plugins)
}

/// List plugins
#[derive(Debug, Default, Args)]
pub struct ListArgs {
    /// Force refresh of plugin manifests
    #[arg(long, short)]
    refresh: bool,

    /// Only list installed plugins
    #[arg(long)]
    installed: bool,

    /// Only list installable plugins
    #[arg(long)]
    installable: bool,

    /// Only list installed but outdated plugins
    #[arg(long, short)]
    outdated: bool,

    /// Only list installed and enabled plugins
    #[arg(long, short)]
    enabled: bool,
}

impl ListArgs {
    pub async fn run(self) -> Result<Table> {
        let list = list(self).await?;

        let mut table = table::new();
        table.set_header(["Name", "Description", "Home", "Version", "Enabled"]);

        for plugin in list {
            let (status, enabled) = plugin.availability();

            let suffix = if plugin.unregistered && plugin.linked {
                Some("(unregistered, linked)")
            } else if plugin.unregistered {
                Some("(unregistered)")
            } else if plugin.linked {
                Some("(linked)")
            } else {
                None
            };

            table.add_row([
                if let Some(suffix) = suffix {
                    Cell::new([&plugin.name, "\n", suffix].concat())
                } else {
                    Cell::new(&plugin.name).add_attribute(Attribute::Bold)
                },
                Cell::new(&plugin.description),
                Cell::new(&plugin.home).fg(Color::Blue),
                match status {
                    PluginStatus::InstalledLatest(version) => Cell::new(version).fg(Color::Green),
                    PluginStatus::InstalledOutdated(installed, latest) => {
                        Cell::new(format!("{installed} → {latest}")).fg(Color::DarkYellow)
                    }
                    PluginStatus::Installable => Cell::new(status).fg(Color::Cyan),
                    PluginStatus::UnavailableRuntime => Cell::new(status).fg(Color::DarkGrey),
                    PluginStatus::UnavailablePlatform => Cell::new(status).fg(Color::Red),
                },
                match enabled {
                    PluginEnabled::NotApplicable => Cell::new(""),
                    PluginEnabled::Yes => Cell::new("yes").fg(Color::Green),
                    PluginEnabled::No => Cell::new("no").fg(Color::DarkGrey),
                },
            ]);
        }

        Ok(table)
    }
}
