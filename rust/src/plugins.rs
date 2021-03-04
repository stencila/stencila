use crate::util::dirs;
use anyhow::{bail, Result};
use futures::StreamExt;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use strum::{Display, EnumString, EnumVariantNames};

#[derive(Debug, Display, EnumString, EnumVariantNames, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Kind {
    #[cfg(any(feature = "plugins-docker"))]
    Docker,
    #[cfg(any(feature = "plugins-binary"))]
    Binary,
    #[cfg(any(feature = "plugins-package"))]
    Package,
}

/// Split a plugin spec (ie. alias/name and, optionally, version) into alias and version
pub fn spec_to_alias_version(spec: &str) -> (String, String) {
    if spec.contains('@') {
        let parts: Vec<&str> = spec.split('@').collect();
        (parts[0].into(), parts[1].into())
    } else {
        (spec.into(), "latest".into())
    }
}

/// Resolve a plugin alias to a plugin name
///
/// If the provided string is a registered alias then returns the corresponding
/// plugin name, otherwise assumes the string is a plugin name
pub fn alias_to_name(alias: &str, aliases: &HashMap<String, String>) -> String {
    match aliases.get(alias) {
        Some(name) => name,
        None => alias,
    }
    .into()
}

/// Get the path to the manifest file of a plugin
pub fn manifest_path(name: &str) -> Result<PathBuf> {
    Ok(dirs::plugins(false)?.join(format!("{}.json", name)))
}

/// Get the manifest for an installed plugin
pub fn manifest(name: &str) -> Result<String> {
    // TODO: deserialize JSON into a manifest object
    match fs::read_to_string(manifest_path(name)?) {
        Ok(json) => Ok(json),
        Err(_) => bail!("Plugin '{}' is not installed", name),
    }
}

/// Test whether a plugin is installed
pub fn is_installed(name: &str) -> Result<bool> {
    Ok(manifest_path(name)?.exists())
}

/// List installed manifests
pub fn list() -> Result<Vec<String>> {
    // TODO: Return a Vec<Manifest>
    let dir = dirs::plugins(true)?;
    let mut plugins: Vec<String> = vec![];
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "json" {
                let name = path.file_stem().unwrap().to_string_lossy().to_string();
                plugins.push(name)
            }
        }
    }
    Ok(plugins)
}

/// Add a plugin as a pulled Docker image
///
/// For this to succeed must be able to connect to the local
/// Docker server and be able to pull an image with corresponding
/// name.
#[cfg(any(feature = "plugins-docker"))]
pub async fn install_docker(name: &str, version: &str) -> Result<()> {
    let docker = bollard::Docker::connect_with_local_defaults()?;

    let image = if name.contains('/') {
        name.to_string()
    } else {
        format!("stencila/{}", name)
    };
    let image = format!("{}:{}", image, version);

    // Pull the image (by creating an image from it)
    let mut stream = docker.create_image(
        Some(bollard::image::CreateImageOptions {
            from_image: image.clone(),
            ..Default::default()
        }),
        None,
        None,
    );
    while let Some(item) = stream.next().await {
        match item {
            Ok(info) => {
                if let Some(error) = info.error {
                    bail!("{}", error)
                } else if let Some(status) = info.status {
                    // TODO display of status and progress could be improved and displayed
                    // per id (layer) as in docker CLI
                    // println!("{:?} {:?} {:?}", info.id, info.progress, info.progress_detail);
                    if let Some(progress) = info.progress {
                        tracing::info!("{} {}", status, progress)
                    } else {
                        tracing::info!("{}", status)
                    }
                }
            }
            Err(error) => match error {
                bollard::errors::Error::DockerResponseNotFoundError { .. } => {
                    bail!("Unable to find Docker image '{}'", image)
                }
                _ => bail!("{}", error),
            },
        }
    }

    // Create a container to obtain manifest from
    let container_name: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(24)
        .map(char::from)
        .collect();
    let response = docker
        .create_container(
            Some(bollard::container::CreateContainerOptions {
                name: &container_name,
            }),
            bollard::container::Config {
                image: Some(image.clone()),
                cmd: Some(vec!["manifest".to_string()]),
                ..Default::default()
            },
        )
        .await?;
    if response.warnings.len() > 0 {
        for warning in response.warnings {
            tracing::warn!("When creating container: {}", warning);
        }
    }

    // Start the container
    docker
        .start_container(
            &container_name,
            None::<bollard::container::StartContainerOptions<String>>,
        )
        .await?;

    // Capture the container output into `manifest`
    let mut stream = docker.logs::<String>(
        &container_name,
        Some(bollard::container::LogsOptions {
            follow: true,
            stderr: true,
            stdout: true,
            ..Default::default()
        }),
    );
    let mut stdout = vec![];
    let mut stderr = vec![];
    while let Some(output) = stream.next().await {
        if let Ok(output) = output {
            match output {
                bollard::container::LogOutput::StdOut { message } => {
                    stdout.extend_from_slice(&message)
                }
                bollard::container::LogOutput::StdErr { message } => {
                    stderr.extend_from_slice(&message)
                }
                _ => {}
            }
        }
    }

    if stderr.len() > 0 {
        tracing::warn!("{}", std::str::from_utf8(&stderr)?);
    }

    let manifest = if stdout.len() > 0 {
        match std::str::from_utf8(&stdout) {
            Ok(stdout) => stdout,
            Err(error) => bail!("Error converting stream to UTF8: {}", error),
        }
    } else {
        bail!(
            "No output from Docker container manifest command; is {} a Stencila plugin?",
            image
        )
    };

    // TODO: desrialize manifest and then serialize to file
    fs::write(manifest_path(name)?, manifest)?;

    Ok(())
}

/// Add a plugin as a downloaded binary
pub fn install_binary(name: &str, version: &str) -> Result<()> {
    // TODO
    bail!("Unable to add plugin '{}@{}' as binary", name, version)
}

/// Add a plugin as a programing language package
pub fn install_package(name: &str, version: &str) -> Result<()> {
    // TODO
    bail!(
        "Unable to add plugin '{}@{}' as programming language package",
        name,
        version
    )
}

/// Add a plugin
pub async fn add(plugin: &str, kinds: &Vec<Kind>, aliases: &HashMap<String, String>) -> Result<()> {
    let (alias, version) = spec_to_alias_version(plugin);
    let name = alias_to_name(&alias, aliases);

    if is_installed(&name)? {
        bail!(
            "Plugin '{}' is already installed, consider using `stencila plugins upgrade {}`",
            name,
            name
        );
    }

    for kind in kinds {
        let result = match kind {
            Kind::Docker => install_docker(&name, &version).await,
            Kind::Binary => install_binary(&name, &version),
            Kind::Package => install_package(&name, &version),
        };
        match result {
            // Success, so just return now
            Ok(_) => return Ok(()),
            // Error, keep trying other kinds, or if there is
            // only one kind, return that error.
            Err(error) => {
                if kinds.len() == 1 {
                    return Err(error);
                }
            }
        }
    }

    bail!(
        "Unable to install plugin '{}', tried kinds {:?}",
        plugin,
        kinds
    )
}

/// Add a list of plugins
pub async fn add_list(
    plugins: Vec<String>,
    kinds: Vec<Kind>,
    aliases: HashMap<String, String>,
) -> Result<()> {
    for plugin in plugins {
        match add(&plugin, &kinds, &aliases).await {
            Ok(_) => tracing::info!("Added plugin {}", plugin),
            Err(error) => tracing::error!("{}", error),
        }
    }
    Ok(())
}

/// Remove a plugin
pub fn remove(plugin: &str, aliases: &HashMap<String, String>) -> Result<()> {
    let (alias, ..) = spec_to_alias_version(plugin);
    let name = alias_to_name(&alias, aliases);

    let file = manifest_path(&name)?;
    if file.exists() {
        fs::remove_file(file)?;
    }

    Ok(())
}

/// Remove a list of plugins
pub fn remove_list(plugins: Vec<String>, aliases: HashMap<String, String>) -> Result<()> {
    for plugin in plugins {
        match remove(&plugin, &aliases) {
            Ok(_) => tracing::info!("Removed plugin {}", plugin),
            Err(error) => tracing::error!("{}", error),
        }
    }
    Ok(())
}

#[cfg(feature = "config")]
pub mod config {
    use super::*;
    use validator::Validate;

    #[derive(Debug, PartialEq, Deserialize, Serialize, Validate)]
    pub struct Config {
        #[serde(default = "default_kinds")]
        pub kinds: Vec<Kind>,

        #[serde(default = "default_aliases")]
        pub aliases: HashMap<String, String>,
    }

    /// Default configuration
    ///
    /// These values are used when `config.toml` does not
    /// contain any config for `upgrade`.
    impl Default for Config {
        fn default() -> Self {
            Config {
                kinds: default_kinds(),
                aliases: default_aliases(),
            }
        }
    }

    /// Get the default value for `kinds`
    pub fn default_kinds() -> Vec<Kind> {
        vec![Kind::Docker, Kind::Binary, Kind::Package]
    }

    /// Get the default value for `aliases`
    pub fn default_aliases() -> HashMap<String, String> {
        let mut aliases = HashMap::new();
        aliases.insert("javascript".to_string(), "jesta".to_string());
        aliases.insert("js".to_string(), "jesta".to_string());
        aliases
    }
}

#[cfg(feature = "cli")]
pub mod cli {
    use super::*;
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Manage plugins",
        setting = structopt::clap::AppSettings::DeriveDisplayOrder
    )]
    pub struct Args {
        #[structopt(subcommand)]
        pub action: Action,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder
    )]
    pub enum Action {
        List,
        Show(Show),
        Add(Add),
        Remove(Remove),
        Upgrade(Upgrade),
    }

    #[derive(Debug, StructOpt)]
    #[structopt(about = "Show the manifest of a plugin")]
    pub struct Show {
        /// The name of the plugin to show
        #[structopt()]
        pub plugin: String,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(about = "Add one or more plugins")]
    pub struct Add {
        /// Attempt to add plugins as Docker image
        #[structopt(short, long)]
        pub docker: bool,

        /// Attempt to add plugins as binary
        #[structopt(short, long)]
        pub binary: bool,

        /// Attempt to add plugins as language package
        #[structopt(short, long)]
        pub package: bool,

        /// The names or aliases of plugins to add
        #[structopt(required = true, multiple = true)]
        pub plugins: Vec<String>,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(about = "Remove one or more plugins")]
    pub struct Remove {
        /// The names or aliases of plugins to remove
        #[structopt(required = true, multiple = true)]
        pub plugins: Vec<String>,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(about = "Upgrade plugins")]
    pub struct Upgrade {
        /// The names or aliases of plugins to upgrade
        /// (omit to upgrade all plugins)
        #[structopt(multiple = true)]
        pub plugins: Vec<String>,
    }

    pub async fn plugins(args: Args) -> Result<()> {
        let Args { action } = args;
        let config::Config { kinds, aliases } = crate::config::get()?.plugins;

        match action {
            Action::List => {
                let plugins = list()?;
                println!("{:?}", plugins);
                Ok(())
            }
            Action::Show(action) => {
                let Show { plugin } = action;

                let manifest = manifest(&plugin)?;
                println!("{}", manifest);
                Ok(())
            }
            Action::Add(action) => {
                let Add {
                    docker,
                    binary,
                    package,
                    plugins,
                } = action;

                let mut kinds_local = vec![];
                if docker {
                    kinds_local.push(Kind::Docker)
                }
                if binary {
                    kinds_local.push(Kind::Binary)
                }
                if package {
                    kinds_local.push(Kind::Package)
                }
                if kinds_local.len() == 0 {
                    kinds_local = kinds
                }

                add_list(plugins, kinds_local, aliases).await
            }
            Action::Remove(action) => {
                let Remove { plugins } = action;

                remove_list(plugins, aliases)
            }
            Action::Upgrade(action) => {
                let Upgrade { plugins } = action;

                let plugins = if plugins.len() == 0 { list()? } else { plugins };

                // Note: Currently, `upgrade` is just an alias for `add`
                // and does not warn user if plugin is not yet installed.
                add_list(plugins, kinds, aliases).await
            }
        }
    }
}
