use app::{get_app_dir, DirType};
use cli_utils::{
    table::{self, Attribute, Cell, Color},
    ToStdout,
};
use common::{
    clap::{self, Args},
    derive_more::Deref,
    eyre::Result,
    futures::future,
    itertools::Itertools,
    serde::{Deserialize, Serialize},
    serde_json,
    tokio::fs::{read_to_string, write},
    tracing,
};

use crate::{Plugin, PluginStatus};

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
pub async fn list(options: ListArgs) -> Result<PluginList> {
    let cache = get_app_dir(DirType::Plugins, true)?.join("manifests.json");

    // TODO: check modifcation time of cache and ignore if more than X hrs old
    if !options.refresh && cache.exists() {
        // If no errors reading or deserializing (e.g. due to change in fields in plugins) then
        // return cached list
        if let Some(list) = read_to_string(&cache)
            .await
            .ok()
            .and_then(|json| serde_json::from_str(&json).ok())
        {
            return Ok(list);
        }
    }

    tracing::info!("Refreshing list of plugins and their manifests");

    // Fetch the plugins list from the Stencila repo
    let plugins = Plugin::fetch_registry().await?;

    // Fetch each of the plugin manifests in parallel, logging debug messages
    // on any errors (this avoids any one plugin with an invalid manifest from
    // breaking the entire fetch or alarming the user with a blaring warning message, while still
    // providing some visibility)
    let futures = plugins
        .iter()
        .map(|(name, url)| async move { Plugin::fetch_manifest(name, url).await });
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
    write(cache, serde_json::to_string(&plugins)?).await?;

    Ok(PluginList(plugins))
}

#[derive(Debug, Default, Args)]
pub struct ListArgs {
    /// Whether to force refresh the plugin manifests
    #[arg(long, short)]
    refresh: bool,

    /// Whether to only show installed plugins
    #[arg(long, short)]
    installed: bool,
}

#[derive(Default, Deref, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct PluginList(Vec<Plugin>);

impl ToStdout for PluginList {
    fn to_terminal(&self) -> impl std::fmt::Display {
        let mut table = table::new();
        table.set_header(["Name", "Description", "Home", "Version"]);

        for plugin in self.iter() {
            let availability = plugin.availability();
            table.add_row([
                Cell::new(&plugin.name).add_attribute(Attribute::Bold),
                Cell::new(&plugin.description),
                Cell::new(&plugin.home).fg(Color::Blue),
                match availability {
                    PluginStatus::InstalledLatest(version) => Cell::new(version).fg(Color::Green),
                    PluginStatus::InstalledOutdated(installed, latest) => {
                        Cell::new(format!("{installed} → {latest}")).fg(Color::DarkYellow)
                    }
                    PluginStatus::Installable => Cell::new(availability).fg(Color::Cyan),
                    PluginStatus::UnavailableRuntime => Cell::new(availability).fg(Color::Grey),
                    PluginStatus::UnavailablePlatform => Cell::new(availability).fg(Color::Red),
                },
            ]);
        }
        table
    }
}
