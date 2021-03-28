use crate::util::dirs;
use anyhow::{anyhow, bail, Result};
use futures::StreamExt;
use jsonschema::JSONSchema;
use once_cell::sync::Lazy;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf, process::Command, sync::Mutex, thread};
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

/// The name of the plugin file within the plugin directory
const PLUGIN_FILE: &str = "codemeta.json";

/// The plugins that have been loaded
type Plugins = HashMap<String, Plugin>;
pub static PLUGINS: Lazy<Mutex<Plugins>> = Lazy::new(|| Mutex::new(HashMap::new()));

type Methods = HashMap<String, Vec<(String, Box<serde_json::Value>, JSONSchema<'static>)>>;
pub static METHODS: Lazy<Mutex<Methods>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// Description of a plugin
///
/// As far as possible using existing properties defined in schema.org
/// [`SoftwareApplication`](https://schema.org/SoftwareApplication) but extensions
/// added where necessary.
///
/// Properties names use the Rust convention of snake_case but are renamed
/// to schema.org camelCase on serialization.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plugin {
    /// The name of the plugin
    name: String,

    /// The version of the plugin
    software_version: String,

    /// A description of the plugin
    description: String,

    /// A list of URLS that the plugin can be installed from
    install_url: Vec<String>,

    /// A list of plugin "features"
    /// Each feature is a `JSONSchema` object describing a method
    /// (including its parameters).
    feature_list: Vec<serde_json::Value>,
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

/// Get the path of the plugin directory
pub fn plugin_dir(name: &str) -> Result<PathBuf> {
    Ok(dirs::plugins(false)?.join(name))
}

/// Get the path of the plugin directory
pub fn plugin_file(name: &str) -> Result<PathBuf> {
    Ok(plugin_dir(name)?.join(PLUGIN_FILE))
}

/// Test whether a plugin is installed
///
/// Note that if a plugin was not successfully installed
/// it may have a directory but no plugin file and `is_installed`
/// will return `false`.
pub fn is_installed(name: &str) -> Result<bool> {
    Ok(plugin_file(name)?.exists())
}

/// Load a plugin from JSON into memory
///
/// Deserialize a plugin from JSON and compile the
/// JSON Schema in each item in its `featureList`.
pub fn load_plugin(json: &str) -> Result<Plugin> {
    let plugin: Plugin = serde_json::from_str(json)?;

    let mut plugins = PLUGINS.lock().expect("Unable to obtain plugins lock");
    plugins.insert(plugin.name.clone(), plugin.clone());

    let mut methods = METHODS.lock().expect("Unable to obtain methods lock");
    for feature in &plugin.feature_list {
        let title = match feature.get("title") {
            None => bail!("JSON Schema is missing 'title' annotation"),
            Some(value) => value.to_string(),
        };

        // The `JSONSchema::compile` function used below wants a reference to a `serde_json::Value`
        // with a lifetime. This is the only way I could figure out how to do that.
        // The `schema_box_droppable` is stored and dropped as part of `unload_plugin` to avoid
        // memory leaks.
        // See https://doc.rust-lang.org/std/boxed/struct.Box.html#method.leak about `Box::leak` and
        // `Box::from_raw`.
        let schema_box = Box::new(feature.clone());
        let schema_static_ref: &'static serde_json::Value = Box::leak(schema_box.clone());
        #[allow(unsafe_code)]
        let schema_box_droppable = unsafe { Box::from_raw(Box::into_raw(schema_box)) };

        // Compile the JSON Schema for this feature
        match JSONSchema::compile(schema_static_ref) {
            Ok(compiled_schema) => {
                methods.entry(title).or_insert_with(Vec::new).push((
                    plugin.name.clone(),
                    schema_box_droppable,
                    compiled_schema,
                ));
            }
            Err(error) => {
                tracing::warn!("Error compiling schema for method '{}' of plugin '{}'; will ignore, please let the plugin maintainer know: {}", title, plugin.name, error)
            }
        };
    }

    Ok(plugin)
}

/// Unload a plugin from memory
///
/// Unregisters the plugin so it will no longer be delegated to
pub fn unload_plugin(name: &str) -> Result<()> {
    let mut plugins = PLUGINS.lock().expect("Unable to obtain plugins lock");
    plugins.remove(name);

    let mut methods = METHODS.lock().expect("Unable to obtain methods lock");
    for implementations in methods.values_mut() {
        implementations.retain(|implementation| implementation.0 != name)
    }

    Ok(())
}

/// Read an installed plugin
pub fn read_plugin(name: &str) -> Result<Plugin> {
    let file = plugin_file(name)?;
    let json = match fs::read_to_string(file) {
        Ok(json) => json,
        Err(_) => bail!("Plugin '{}' is not installed", name),
    };
    let plugin = load_plugin(&json)?;
    Ok(plugin)
}

/// Write a plugin to installation directory
pub fn write_plugin(name: &str, plugin: &Plugin) -> Result<()> {
    let json = serde_json::to_string_pretty(&plugin)?;
    let dir = plugin_dir(name)?;
    fs::create_dir_all(dir)?;
    let file = plugin_file(name)?;
    fs::write(file, json)?;
    Ok(())
}

/// Create a Markdown document describing a plugin
pub fn display_plugin(name: &str, format: &str) -> Result<String> {
    let plugin = read_plugin(name)?;
    let content = match format {
        #[cfg(any(feature = "template-handlebars"))]
        "md" => {
            let template = r#"
# {{name}} {{softwareVersion}}

{{description}}

## Installation options

{{#each installUrl}}
- {{this}}{{/each}}


## Methods

{{#each featureList}}
### {{title}}

{{description}}

{{#each properties}}
- **{{@key}}**: *{{type}}* : {{description}}{{/each}}

{{/each}}

"#;
            use handlebars::Handlebars;
            let hb = Handlebars::new();
            hb.render_template(template, &plugin)?
        }
        _ => serde_json::to_string_pretty(&plugin)?,
    };
    Ok(content)
}

/// Read all the installed plugins
pub fn read_plugins() -> Result<Vec<Plugin>> {
    let dir = dirs::plugins(true)?;
    let mut plugins: Vec<Plugin> = vec![];
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let name = path.display().to_string();
            // Check this directory actually has a plugin file
            if is_installed(&name)? {
                plugins.push(read_plugin(&name)?);
            }
        }
    }
    Ok(plugins)
}

/// Create a Markdown table of all the install plugins
pub fn display_plugins() -> Result<String> {
    let plugins = read_plugins()?;

    if plugins.is_empty() {
        return Ok("No plugins installed. See `stencila plugins install --help`.".to_string());
    }

    let head = r#"
| ---- | ------- | ------------ |
| Name | Version | Description  |
| :--- | ------: | -------------|
"#
    .trim();
    let body = plugins
        .iter()
        .map(|plugin| {
            format!(
                "| **{}** | {} | {} |",
                plugin.name, plugin.software_version, plugin.description
            )
        })
        .collect::<Vec<String>>()
        .join("\n");
    let foot = "|-";
    Ok(format!("{}\n{}\n{}\n", head, body, foot))
}

/// Uninstall and unload (from memory) a plugin
pub fn uninstall_plugin(name: &str) -> Result<()> {
    let dir = plugin_dir(&name)?;
    if dir.exists() || fs::symlink_metadata(&dir).is_ok() {
        if dir.is_file() {
            fs::remove_file(dir)?
        } else {
            // Note that if `dir` is a symlink to a directory that
            // only the directory will be removed.
            fs::remove_dir_all(dir)?
        }
    }

    unload_plugin(name)
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
    if !response.warnings.is_empty() {
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

    if !stderr.is_empty() {
        tracing::warn!("{}", std::str::from_utf8(&stderr)?);
    }

    let json = if !stdout.is_empty() {
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

    // Remove the plugin directory
    uninstall_plugin(&name)?;

    // Load and write the plugin file
    let plugin = load_plugin(json)?;
    write_plugin(name, &plugin)?;

    Ok(())
}

/// Add a plugin as a downloaded binary
#[cfg(any(feature = "plugins-binary"))]
pub fn install_binary(name: &str, version: &str) -> Result<()> {
    let (owner, name) = if name.contains('/') {
        let parts: Vec<&str> = name.split('/').collect();
        (parts[0], parts[1])
    } else {
        ("stencila", name)
    };

    // Get the current version from the manifest (if any)
    let current_version = if let Ok(_manifest) = read_plugin(name) {
        // TODO extract version from manifest
        "0.1.0"
    } else {
        // Not yet installed, so artificially low version
        "0.0.0"
    };

    // Remove the plugin directory
    uninstall_plugin(&name)?;

    // (Re)create the directory where the binary will be downloaded to
    let install_dir = dirs::plugins(false)?.join(name);
    fs::create_dir_all(&install_dir)?;
    let install_path = install_dir.join(name);

    let mut builder = self_update::backends::github::Update::configure();
    builder
        .repo_owner(owner)
        .repo_name(name)
        .bin_name(name)
        .current_version(current_version)
        .bin_install_path(&install_path)
        .show_output(true)
        .show_download_progress(true);
    if version != "latest" {
        builder.target_version_tag(format!("v{}", version).as_str());
    }

    // The download has to be done in another thread because it spawns
    // a new tokio runtime
    thread::spawn(move || -> Result<()> {
        if let Err(error) = builder.build()?.update() {
            match error {
                self_update::errors::Error::Network(message) => {
                    if message.contains("404") {
                        bail!("Could not find repository or corresponding release in repository")
                    } else {
                        bail!(message)
                    }
                }
                _ => bail!(error.to_string()),
            }
        } else {
            Ok(())
        }
    })
    .join()
    .map_err(|_| anyhow!("Error joining thread"))??;

    // Get plugin json manifest
    let json = Command::new(&install_path).arg("manifest").output()?.stdout;
    let json = std::str::from_utf8(&json)?;

    let plugin = load_plugin(json)?;
    write_plugin(name, &plugin)?;

    Ok(())
}

/// Add a plugin as a programing language package
#[cfg(any(feature = "plugins-package"))]
pub fn install_package(name: &str, version: &str) -> Result<()> {
    // TODO
    bail!(
        "Unable to add plugin '{}@{}' as programming language package",
        name,
        version
    )
}

/// Add a plugin as a soft link to a directory on the current machine
///
/// # Arguments
///
/// - `path`: Local file system path to the directory
#[cfg(any(feature = "plugins-link"))]
pub fn install_link(path: &str) -> Result<()> {
    // Make the path absolute (for symlink to work)
    let path = fs::canonicalize(&path)?;

    // Check that the path is a directory
    if !path.is_dir() {
        bail!("Path must be a directory")
    }

    // Check that the directory has a plugin file
    let plugin_file = path.join(PLUGIN_FILE);
    if !plugin_file.is_file() {
        bail!("Directory must contain a '{}' file", PLUGIN_FILE)
    }

    // Check that the plugin's file can be loaded
    let json = fs::read_to_string(plugin_file)?;
    let plugin = load_plugin(&json)?;
    let name = plugin.name;

    // Remove the plugin directory
    uninstall_plugin(&name)?;

    // Create the soft link
    let link = plugin_dir(&name)?;
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    std::os::unix::fs::symlink(path, link)?;
    #[cfg(target_os = "windows")]
    std::os::windows::fs::symlink_dir(path, link)?;

    Ok(())
}

/// Add a plugin
pub async fn install(
    plugin: &str,
    kinds: &[Kind],
    aliases: &HashMap<String, String>,
) -> Result<()> {
    let (alias, version) = spec_to_alias_version(plugin);
    let name = alias_to_name(&alias, aliases);

    if is_installed(&name)? {
        uninstall_plugin(&name)?
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
pub async fn install_list(
    plugins: Vec<String>,
    kinds: Vec<Kind>,
    aliases: HashMap<String, String>,
) -> Result<()> {
    for plugin in plugins {
        match install(&plugin, &kinds, &aliases).await {
            Ok(_) => tracing::info!("Added plugin {}", plugin),
            Err(error) => tracing::error!("{}", error),
        }
    }
    Ok(())
}

/// Remove a plugin
pub fn uninstall(alias: &str, aliases: &HashMap<String, String>) -> Result<()> {
    let name = alias_to_name(&alias, &aliases);
    uninstall_plugin(&name)?;

    Ok(())
}

/// Remove a list of plugins
pub fn uninstall_list(plugins: Vec<String>, aliases: &HashMap<String, String>) -> Result<()> {
    for plugin in plugins {
        match uninstall(&plugin, &aliases) {
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
        #[structopt(about = "List installed plugins")]
        List,
        Show(Show),
        Install(Install),
        Link(Link),
        Upgrade(Upgrade),
        Uninstall(Uninstall),
        Unlink(Unlink)
    }

    #[derive(Debug, StructOpt)]
    #[structopt(about = "Show the details of an installed plugin")]
    pub struct Show {
        /// The name of the plugin to show
        #[structopt()]
        pub plugin: String,

        /// The format
        #[structopt(short, long, default_value = "md")]
        pub format: String,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(about = "Install one or more plugins")]
    pub struct Install {
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
    #[structopt(about = "Link to a local plugins")]
    pub struct Link {
        /// The path of a plugin directory
        #[structopt()]
        pub path: String,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(about = "Upgrade one of more plugins")]
    pub struct Upgrade {
        /// The names or aliases of plugins to upgrade
        /// (omit to upgrade all plugins)
        #[structopt(multiple = true)]
        pub plugins: Vec<String>,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(about = "Uninstall one or more plugins")]
    pub struct Uninstall {
        /// The names or aliases of plugins to uninstall
        #[structopt(required = true, multiple = true)]
        pub plugins: Vec<String>,
    }


    #[derive(Debug, StructOpt)]
    #[structopt(about = "Unlink a local plugins")]
    pub struct Unlink {
        /// The name of the plugin
        #[structopt()]
        pub name: String,
    }

    pub async fn plugins(args: Args) -> Result<()> {
        let Args { action } = args;
        let config::Config { kinds, aliases } = crate::config::get()?.plugins;

        let skin = termimad::MadSkin::default();
        match action {
            Action::List => {
                let md = display_plugins()?;
                println!("{}", skin.term_text(md.as_str()));
                Ok(())
            }
            Action::Show(action) => {
                let Show { plugin, format } = action;

                let content = display_plugin(&plugin, &format)?;
                if format == "json" {
                    println!("{}", content)
                } else {
                    println!("{}", skin.term_text(content.as_str()))
                }
                Ok(())
            }
            Action::Install(action) => {
                let Install {
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
                if kinds_local.is_empty() {
                    kinds_local = kinds
                }

                install_list(plugins, kinds_local, aliases).await
            }
            Action::Link(action) => {
                let Link { path } = action;

                install_link(&path)
            }
            Action::Upgrade(action) => {
                let Upgrade { plugins } = action;

                let plugins = if plugins.is_empty() {
                    read_plugins()?
                        .iter()
                        .map(|plugin| plugin.name.clone())
                        .collect()
                } else {
                    plugins
                };

                // Note: Currently, `upgrade` is just an alias for `install`
                // and does not warn user if plugin is not yet installed.
                install_list(plugins, kinds, aliases).await
            }
            Action::Uninstall(action) => {
                let Uninstall { plugins } = action;

                uninstall_list(plugins, &aliases)
            }
            Action::Unlink(action) => {
                let Unlink { name } = action;

                uninstall_plugin(&name)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cli() -> Result<()> {
        // These tests don't do anything other test that
        // actions run with expected `Ok` or `Err`.

        use super::cli::*;

        plugins(Args {
            action: Action::List,
        })
        .await?;

        plugins(Args {
            action: Action::Show(Show {
                plugin: "foo".to_string(),
                format: "md".to_string(),
            }),
        })
        .await
        .expect_err("Expected an error!");

        plugins(Args {
            action: Action::Install(Install {
                plugins: vec![],
                docker: false,
                binary: false,
                package: false,
            }),
        })
        .await?;

        plugins(Args {
            action: Action::Link(Link {
                path: "../foo".to_string(),
            }),
        })
        .await
        .expect_err("Expected an error!");

        plugins(Args {
            action: Action::Upgrade(Upgrade { plugins: vec![] }),
        })
        .await?;

        plugins(Args {
            action: Action::Uninstall(Uninstall { plugins: vec![] }),
        })
        .await?;

        Ok(())
    }
}
