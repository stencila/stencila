use std::collections::HashMap;

use app::{get_app_dir, DirType};
use cli_utils::{
    table::{self, Attribute, Cell},
    ToStdout,
};
use common::{
    clap::{self, Args},
    derive_more::Deref,
    eyre::{bail, eyre, Report, Result},
    futures::future,
    itertools::Itertools,
    reqwest::get,
    serde::{Deserialize, Serialize},
    serde_json,
    tokio::fs::{read_to_string, write},
    toml, tracing,
};

use crate::Plugin;

// TODO: change URL to point to `main` before PR is merged
const PLUGINS_TOML_URL: &str =
    "https://raw.githubusercontent.com/stencila/stencila/feature/plugins/plugins.toml";

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
pub async fn list(options: ListOptions) -> Result<PluginList> {
    tracing::info!("Refreshing list of plugins and their manifests");

    let cache = get_app_dir(DirType::Plugins, true)?.join("manifests.json");

    if !options.refresh && cache.exists() {
        let json = read_to_string(cache).await?;
        let list = serde_json::from_str(&json)?;
        return Ok(list);
    }

    // Fetch the plugins list from the Stencila repo
    let response = get(PLUGINS_TOML_URL).await?;
    if let Err(error) = response.error_for_status_ref() {
        let message = response.text().await?;
        bail!("{error}: {message}");
    }
    let toml = response.text().await?;
    let plugins: HashMap<String, String> = toml::from_str(&toml)?;

    // Fetch each of the plugin manifests in parallel, logging debug messages
    // on any errors (this avoids any one plugin with an invalid manifest from
    // breaking the entire fetch or alarming the user with a blaring warning message, while still
    // providing some visibility)
    let futures = plugins.iter().map(|(name, url)| async move {
        {
            let response = get(url).await?;
            if let Err(error) = response.error_for_status_ref() {
                let message = response.text().await?;
                bail!("While fetching plugin `{name}` from `{url}`: {error}: {message}");
            }

            let toml = response.text().await?;
            let plugin: Plugin = toml::from_str(&toml)
                .map_err(|error| eyre!("While deserializing plugin `{name}`: {error}"))?;

            if &plugin.name != name {
                bail!(
                    "Plugin name is not the same as in plugin list: `{}` != `{}`",
                    plugin.name,
                    name
                )
            }

            Ok::<Plugin, Report>(plugin)
        }
        .map_err(|error| eyre!("Error with plugin `{name}`: {error}"))
    });
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
    write(cache, serde_json::to_string(&plugins)?).await?;

    Ok(PluginList(plugins))
}

#[derive(Debug, Default, Args)]
pub struct ListOptions {
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
        table.set_header(["Name"]);
        for plugin in self.iter() {
            table.add_row([Cell::new(&plugin.name).add_attribute(Attribute::Bold)]);
        }
        table
    }
}
