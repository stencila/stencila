use crate::util::dirs;
use anyhow::{anyhow, bail, Result};
use chrono::{DateTime, Duration, TimeZone, Utc};
use dirs::plugins;
use futures::StreamExt;
use humantime::format_duration;
use jsonschema::JSONSchema;
use once_cell::sync::Lazy;
use rand::Rng;
use regex::Regex;
use schemars::{schema_for, JsonSchema};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};

/// # Plugin installation method
///
/// Which method to use to install a plugin.
#[derive(
    Debug, Display, Clone, Copy, EnumString, EnumIter, PartialEq, JsonSchema, Deserialize, Serialize,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum PluginInstallation {
    #[cfg(any(feature = "plugins-docker"))]
    Docker,
    #[cfg(any(feature = "plugins-binary"))]
    Binary,
    #[cfg(any(feature = "plugins-javascript"))]
    Javascript,
    #[cfg(any(feature = "plugins-python"))]
    Python,
    #[cfg(any(feature = "plugins-r"))]
    R,
    #[cfg(any(feature = "plugins-link"))]
    Link,
}

/// # Description of a plugin
///
/// As far as possible using existing properties defined in schema.org
/// [`SoftwareApplication`](https://schema.org/SoftwareApplication) but extensions
/// added where necessary.
#[derive(Debug, Default, Clone, JsonSchema, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plugin {
    // Property names use the Rust convention of snake_case but are renamed
    // to schema.org camelCase on serialization.

    // Properties that are read from the plugin's manifest file
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

    // Properties set / derived at runtime (should all be optional)
    /// If the plugin is installed, the installation type
    #[serde(skip_serializing_if = "Option::is_none")]
    installation: Option<PluginInstallation>,

    /// The last time that the plugin manifest was updated.
    /// Used to determine if a refresh is necessary.
    #[serde(skip_serializing_if = "Option::is_none")]
    refreshed: Option<DateTime<Utc>>,

    /// The next version of the plugin, if any.
    /// If the plugin is installed and there is a newer version of
    /// the plugin then this property should be set at the
    /// time of refresh.
    #[serde(skip_serializing_if = "Option::is_none")]
    next: Option<Box<Plugin>>,

    /// The current alias for this plugin, if any
    #[serde(skip_serializing_if = "Option::is_none")]
    alias: Option<String>,
}

/// Get the JSON Schema for a plugin
pub fn schema() -> String {
    let schema = schema_for!(Plugin);
    serde_json::to_string_pretty(&schema).unwrap()
}

impl Plugin {
    /// The name of the plugin file within the plugin directory
    const FILE_NAME: &'static str = "codemeta.json";

    /// Create a Markdown document describing a plugin
    pub fn display(&self, format: &str) -> Result<String> {
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
                hb.render_template(template, self)?
            }
            _ => serde_json::to_string_pretty(self)?,
        };
        Ok(content)
    }

    /// Split a plugin spec into `owner`, `name`, and `version`
    pub fn spec_to_parts(spec: &str) -> (&str, &str, &str) {
        let (owner, name) = if spec.contains('/') {
            let parts: Vec<&str> = spec.split('/').collect();
            (parts[0].trim(), parts[1].trim())
        } else {
            ("stencila", spec)
        };

        let (name, version) = if name.contains('@') {
            let parts: Vec<&str> = name.split('@').collect();
            (parts[0].trim(), parts[1].trim())
        } else {
            (name, "latest")
        };

        (owner, name, version)
    }

    /// Merge locally configured aliases into global aliases possibly extending
    /// and overriding them
    fn merge_aliases(
        global: &HashMap<String, String>,
        local: &HashMap<String, String>,
    ) -> HashMap<String, String> {
        let mut aliases = global.clone();
        aliases.extend(local.iter().map(|(k, v)| (k.clone(), v.clone())));
        aliases
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

    /// Resolve a plugin name to a preferred alias
    pub fn name_to_alias(name: &str, aliases: &HashMap<String, String>) -> String {
        for (alias, value) in aliases.iter() {
            if name == value {
                return alias.into();
            }
        }
        name.into()
    }

    /// Get the path of the plugin's directory
    pub fn dir(name: &str) -> Result<PathBuf> {
        Ok(dirs::plugins(false)?.join(name))
    }

    /// Get the path of the plugin's manifest file
    pub fn file(name: &str) -> Result<PathBuf> {
        Ok(Plugin::dir(name)?.join(Plugin::FILE_NAME))
    }

    /// Load a plugin from its JSON manifest
    pub fn load(json: &str) -> Result<Plugin> {
        let plugin: Plugin = match serde_json::from_str(json) {
            Ok(plugin) => plugin,
            Err(error) => bail!("Error parsing plugin manifest: {}", error),
        };
        Ok(plugin)
    }

    /// Load a plugin from its JSON manifest generated by running it with
    /// the manifest subcommand
    #[tracing::instrument]
    pub fn load_from_command(command: &mut Command) -> Result<Plugin> {
        let output = match command.stdout(Stdio::piped()).output() {
            Ok(output) => output,
            Err(error) => bail!("When attempting to run command: {}", error),
        };

        if !output.status.success() {
            bail!(
                "When attempting to get manifest: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
        }

        let json = std::str::from_utf8(&output.stdout)?;
        let plugin: Plugin = serde_json::from_str(json)?;
        Ok(plugin)
    }

    /// Read the plugin from its file
    ///
    /// If the plugin directory is a symlink then set the installation
    /// method as `Link`.
    pub fn read(name: &str) -> Result<Plugin> {
        let json = match fs::read_to_string(Plugin::file(name)?) {
            Ok(json) => json,
            Err(_) => bail!("Plugin '{}' is not installed", name),
        };

        let mut plugin = Plugin::load(&json)?;

        if fs::symlink_metadata(Plugin::dir(name)?)?
            .file_type()
            .is_symlink()
        {
            plugin.installation = Some(PluginInstallation::Link);
        }

        Ok(plugin)
    }

    /// Write the plugin to its directory
    pub fn write(name: &str, plugin: &Plugin) -> Result<()> {
        let json = serde_json::to_string_pretty(plugin)?;

        let dir = Plugin::dir(name)?;
        fs::create_dir_all(dir)?;

        let file = Plugin::file(name)?;
        fs::write(file, json)?;

        Ok(())
    }

    /// Remove the plugin's directory
    pub fn remove(name: &str) -> Result<()> {
        let dir = Plugin::dir(name)?;
        if dir.exists() || fs::symlink_metadata(&dir).is_ok() {
            if dir.is_file() {
                fs::remove_file(dir)?
            } else {
                // Note that if `dir` is a symlink to a directory that
                // only the directory will be removed.
                fs::remove_dir_all(dir)?
            }
        }

        Ok(())
    }

    /// Replace a plugin
    ///
    /// Removes any existing plugin directory and then recreates it
    /// with the new plugin file
    pub fn replace(name: &str, plugin: &Plugin) -> Result<()> {
        Plugin::remove(name)?;
        Plugin::write(name, plugin)
    }

    /// Install a plugin
    pub async fn install(
        spec: &str,
        installs: &[PluginInstallation],
        aliases: &HashMap<String, String>,
        plugins: &mut Plugins,
        current_version: Option<String>,
    ) -> Result<()> {
        // Parse the spec to get it's parts.
        // If there is no matching plugin then we'll attempt to use these parts
        // to install using each of the installation methods.
        let (owner, name, version) = Plugin::spec_to_parts(&spec);

        // Attempt to convert the parsed name using aliases
        let aliases = Plugin::merge_aliases(&plugins.aliases, aliases);
        let name = Plugin::alias_to_name(name, &aliases);
        let name = name.as_str();

        // Attempt to get the matching plugin so we can use it's `installUrl` property
        // if possible.
        let plugin = plugins.plugins.get(name);

        // If the plugin hasn't been refreshed for a while then do that
        let plugin = if let Some(plugin) = plugin {
            if Utc::now()
                > plugin
                    .refreshed
                    .unwrap_or_else(|| Utc.ymd(1900, 1, 1).and_hms(0, 0, 0))
                    + Duration::from_std(humantime::parse_duration("1 day")?)?
            {
                // Load the plugin's latest manifest from its URL
                let url = match plugins.registry.get::<str>(&name) {
                    None => bail!("No plugin registered with alias or name '{}'", name),
                    Some(url) => url,
                };
                let json = reqwest::get(url).await?.text().await?;
                let latest = Plugin::load(&json)?;
                Some(latest)
            } else {
                Some(plugin.clone())
            }
        } else {
            None
        };

        for install in installs {
            let result = match install {
                PluginInstallation::Docker => {
                    Plugin::install_docker(&plugin, owner, name, version).await
                }
                PluginInstallation::Binary => Plugin::install_binary(
                    &plugin,
                    owner,
                    name,
                    version,
                    current_version.clone(),
                    false,
                    true,
                ),
                PluginInstallation::Javascript => Plugin::install_js(&plugin, owner, name, version),
                PluginInstallation::Python => Plugin::install_py(&plugin, name, version),
                PluginInstallation::R => Plugin::install_r(&plugin, name, version),
                PluginInstallation::Link => Plugin::install_link(spec),
            };
            match result {
                // Success, so add the plugin to the in-memory store
                Ok(plugin) => {
                    plugins.add(plugin)?;
                    return Ok(());
                }
                // Error, keep trying other install methods, or if there is
                // only one method, return that error.
                Err(error) => {
                    if installs.len() == 1 {
                        return Err(error);
                    }
                }
            }
        }

        bail!(
            "Unable to install plugin '{}', tried installation methods {:?}",
            spec,
            installs
        )
    }

    /// Install a list of plugins
    pub async fn install_list(
        specs: Vec<String>,
        installs: &[PluginInstallation],
        aliases: &HashMap<String, String>,
        plugins: &mut Plugins,
    ) -> Result<()> {
        for spec in specs {
            match Plugin::install(&spec, installs, aliases, plugins, None).await {
                Ok(_) => tracing::info!("Added plugin {}", spec),
                Err(error) => bail!(error),
            }
        }
        Ok(())
    }

    /// Parse a plugin's NPM install URL (if any)
    #[cfg(any(feature = "plugins-javascript"))]
    pub fn parse_js_install_url(self: &Plugin) -> Option<(String, String)> {
        static REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"^https?://www.npmjs.com/package/@([a-z]+)/([a-z]+)")
                .expect("Unable to create regex")
        });
        for url in &self.install_url {
            if let Some(captures) = REGEX.captures(&url) {
                return Some((
                    captures.get(1).map_or("", |matc| matc.as_str()).into(),
                    captures.get(2).map_or("", |matc| matc.as_str()).into(),
                ));
            }
        }
        None
    }

    /// Add a plugin as a NPM package
    ///
    /// Installs the package within the `plugins/node_modules` directory so that it
    /// is not necessary to run as root (the NPM `--global` flag requires sudo).
    #[cfg(any(feature = "plugins-javascript"))]
    pub fn install_js(
        plugin: &Option<Plugin>,
        owner: &str,
        name: &str,
        version: &str,
    ) -> Result<Plugin> {
        // If this is a known, registered plugin then check that it can be installed
        // as a NPM package and use declared repo owner and name.
        // Otherwise, use the repo owner and name parsed from the spec.
        let (owner, name) = if let Some(plugin) = plugin {
            if let Some((owner, name)) = plugin.parse_js_install_url() {
                (owner, name)
            } else {
                return Err(anyhow!(
                    "Install as NPM package is not supported by plugin '{}'",
                    plugin.name
                ));
            }
        } else {
            (owner.into(), name.into())
        };

        let npm_prefix = plugins(false)?;
        let node_modules = npm_prefix.join("node_modules");
        fs::create_dir_all(&node_modules)?;

        // Ensure we have a package.json in npm_prefix to avoid warnings
        let package_json = npm_prefix.join("package.json");
        if !package_json.exists() {
            let json = r#"{
  "description": "Stencila plugins installed as NPM packages",
  "repository": "https://github.com/stencila/stencila",
  "license": "Apache-2.0"
}"#;
            fs::write(package_json, json)?;
        }

        tracing::debug!(
            "Installing NPM package '{}/{}@{}' to '{}'",
            owner,
            name,
            version,
            node_modules.display()
        );

        match Command::new("npm")
            .arg(format!("--prefix={}", npm_prefix.display()))
            .arg("install")
            .arg(format!("@{}/{}@{}", owner, name, version))
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(child) => {
                let output = child.wait_with_output()?;
                if output.status.success() {
                    tracing::info!("NPM package '{}/{}@{}' installed", owner, name, version);

                    let bin = node_modules.join(".bin").join(&name).display().to_string();
                    tracing::debug!("Obtaining manifest from {}", bin);

                    let mut plugin =
                        Plugin::load_from_command(Command::new("node").arg(bin).arg("manifest"))?;
                    plugin.installation = Some(PluginInstallation::Javascript);
                    Plugin::replace(&name, &plugin)?;
                    Ok(plugin)
                } else {
                    bail!(
                        "When installing NPM package '{}': {} {}",
                        name,
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    )
                }
            }
            Err(error) => {
                bail!(
                    "When attempting to run `npm` (is it installed and on your PATH?): {}",
                    error.to_string()
                )
            }
        }
    }

    /// Parse a plugin's PyPI install URL (if any)
    #[cfg(any(feature = "plugins-python"))]
    pub fn parse_py_install_url(self: &Plugin) -> Option<String> {
        static REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"^https?://pypi.org/project/([\w\-\.]+)").expect("Unable to create regex")
        });
        for url in &self.install_url {
            if let Some(captures) = REGEX.captures(&url) {
                return Some(captures.get(1).map_or("", |matc| matc.as_str()).into());
            }
        }
        None
    }

    /// Add a plugin as a Python package from PyPI
    ///
    /// Installs the package globally for the user.
    #[cfg(any(feature = "plugins-python"))]
    pub fn install_py(plugin: &Option<Plugin>, name: &str, version: &str) -> Result<Plugin> {
        // If this is a known, registered plugin then check that it can be installed
        // as a Python package and use declared package name.
        // Otherwise, use the name parsed from the spec.
        let name = if let Some(plugin) = plugin {
            if let Some(name) = plugin.parse_py_install_url() {
                name
            } else {
                return Err(anyhow!(
                    "Install as Python package is not supported by plugin '{}'",
                    plugin.name
                ));
            }
        } else {
            name.into()
        };

        tracing::debug!("Installing PyPi package '{}@{}'", name, version);

        match Command::new("python3")
            .arg("-mpip")
            .arg("install")
            .arg(if version != "latest" {
                format!("{}=={}", name, version)
            } else {
                name.to_string()
            })
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(child) => {
                let output = child.wait_with_output()?;
                if output.status.success() {
                    tracing::info!("PyPi package '{}@{}' installed", name, version);

                    tracing::debug!("Obtaining manifest from Python module {}", name);
                    let mut plugin = Plugin::load_from_command(
                        Command::new("python3")
                            .arg(format!("-m{}", name))
                            .arg("manifest"),
                    )?;
                    plugin.installation = Some(PluginInstallation::Python);
                    Plugin::replace(&name, &plugin)?;
                    Ok(plugin)
                } else {
                    bail!(
                        "When installing CRAN package '{}': {} {}",
                        name,
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    )
                }
            }
            Err(error) => {
                bail!(
                    "When attempting to run `python3` (is it installed and on your PATH?): {}",
                    error.to_string()
                )
            }
        }
    }

    /// Parse a plugin's CRAN install URL (if any)
    #[cfg(any(feature = "plugins-r"))]
    pub fn parse_r_install_url(self: &Plugin) -> Option<String> {
        static REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"^https://cran.r-project.org/web/packages/([\w\-\.]+)")
                .expect("Unable to create regex")
        });
        for url in &self.install_url {
            if let Some(captures) = REGEX.captures(&url) {
                return Some(captures.get(1).map_or("", |matc| matc.as_str()).into());
            }
        }
        None
    }

    /// Add a plugin as a R package from CRAN
    #[cfg(any(feature = "plugins-r"))]
    pub fn install_r(plugin: &Option<Plugin>, name: &str, version: &str) -> Result<Plugin> {
        // If this is a known, registered plugin then check that it can be installed
        // as a R package and use declared package name. Otherwise, use the name parsed from the spec.
        let name = if let Some(plugin) = plugin {
            if let Some(name) = plugin.parse_r_install_url() {
                name
            } else {
                return Err(anyhow!(
                    "Install as R package is not supported by plugin '{}'",
                    plugin.name
                ));
            }
        } else {
            name.into()
        };

        tracing::debug!("Installing CRAN package '{}'", name);
        if version != "latest" {
            tracing::warn!("Installing specific versions of CRAN package is not currently support; will install latest version");
        }

        match Command::new("Rscript")
            .arg("-e")
            .arg(format!(
                "tryCatch(install.packages('{}'), warning = function(e) {{ cat(e$message); quit(status=1) }})",
                name
            ))
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(child) => {
                let output = child.wait_with_output()?;
                if output.status.success() {
                    tracing::info!("CRAN package '{}' installed", name);

                    tracing::debug!("Obtaining manifest from R package {}", name);
                    let mut plugin = Plugin::load_from_command(
                        Command::new("Rscript")
                            .arg("-e")
                            .arg(format!("{}::manifest()", name)),
                    )?;
                    plugin.installation = Some(PluginInstallation::R);
                    Plugin::replace(&name, &plugin)?;
                    Ok(plugin)
                } else {
                    bail!(
                        "When installing CRAN package '{}': {} {}",
                        name,
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    )
                }
            }
            Err(error) => {
                bail!("When attempting to run `Rscript` (is it installed and on your PATH?): {}", error.to_string())
            }
        }
    }

    /// Parse a plugin's binary install URL (if any)
    ///
    /// Currently this will parse a GitHub URL. In the future is may also
    /// parse other URLs used for distributing URLs.
    #[cfg(any(feature = "plugins-binary"))]
    pub fn parse_binary_install_url(self: &Plugin) -> Option<(String, String)> {
        static REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"^https?://github.com/([a-z]+)/([a-z]+)/releases")
                .expect("Unable to create regex")
        });
        for url in &self.install_url {
            if let Some(captures) = REGEX.captures(&url) {
                return Some((
                    captures.get(1).map_or("", |matc| matc.as_str()).into(),
                    captures.get(2).map_or("", |matc| matc.as_str()).into(),
                ));
            }
        }
        None
    }

    /// Add a plugin as a downloaded binary
    #[cfg(any(feature = "plugins-binary"))]
    pub fn install_binary(
        plugin: &Option<Plugin>,
        owner: &str,
        name: &str,
        version: &str,
        current_version: Option<String>,
        confirm: bool,
        verbose: bool,
    ) -> Result<Plugin> {
        // If this is a known, registered plugin then check that it can be installed
        // as a binary and use declared repo owner and name.
        // Otherwise, use the repo owner and name parsed from the spec.
        let (owner, name) = if let Some(plugin) = plugin {
            if let Some((owner, name)) = plugin.parse_binary_install_url() {
                (owner, name)
            } else {
                return Err(anyhow!(
                    "Binary install is not supported by plugin '{}'",
                    plugin.name
                ));
            }
        } else {
            (owner.into(), name.into())
        };

        // Remove the plugin directory if this is not an upgrade
        // (we don't want it remove if the user aborts download)
        if current_version.is_none() {
            Plugin::remove(&name)?
        }

        // (Re)create the directory where the binary will be downloaded to
        let install_dir = dirs::plugins(false)?.join(&name);
        fs::create_dir_all(&install_dir)?;
        let install_path = install_dir.join(&name);

        let mut builder = self_update::backends::github::Update::configure();
        builder
            .repo_owner(&owner)
            .repo_name(&name)
            .bin_name(&name)
            .current_version(&current_version.unwrap_or_else(|| "0.0.0".into()))
            .bin_install_path(&install_path)
            .no_confirm(!confirm)
            .show_output(verbose)
            .show_download_progress(verbose);
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
                            bail!(
                                "Could not find repository or corresponding release in repository"
                            )
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

        // Get plugin JSON manifest and write it to disk
        let mut plugin =
            Plugin::load_from_command(&mut Command::new(&install_path).arg("manifest"))?;
        plugin.installation = Some(PluginInstallation::Binary);
        Plugin::write(&name, &plugin)?;
        Ok(plugin)
    }

    /// Parse a plugin's Docker install URL (if any)
    ///
    /// Currently this will parse a Docker Hub URL. In the future is may also
    /// parse container registry URLs e.g. gcr.io
    #[cfg(any(feature = "plugins-docker"))]
    pub fn parse_docker_install_url(self: &Plugin) -> Option<(String, String)> {
        for url in &self.install_url {
            static REGEX: Lazy<Regex> = Lazy::new(|| {
                Regex::new(r"^https?://hub.docker.com/r/([a-z]+)/([a-z]+)")
                    .expect("Unable to create regex")
            });
            if let Some(captures) = REGEX.captures(&url) {
                return Some((
                    captures.get(1).map_or("", |matc| matc.as_str()).into(),
                    captures.get(2).map_or("", |matc| matc.as_str()).into(),
                ));
            }
        }
        None
    }

    /// Add a plugin as a pulled Docker image
    ///
    /// For this to succeed must be able to connect to the local
    /// Docker server and be able to pull an image with corresponding
    /// name.
    #[cfg(any(feature = "plugins-docker"))]
    pub async fn install_docker(
        plugin: &Option<Plugin>,
        owner: &str,
        name: &str,
        version: &str,
    ) -> Result<Plugin> {
        // If this is a known, registered plugin then check that it can be installed
        // as a Docker image and use the declared image owner and name.
        // Otherwise, use the owner and name parsed from the spec.
        let (owner, name) = if let Some(plugin) = plugin {
            if let Some((owner, name)) = plugin.parse_docker_install_url() {
                (owner, name)
            } else {
                return Err(anyhow!(
                    "Docker install is not supported by plugin '{}'",
                    plugin.name
                ));
            }
        } else {
            (owner.into(), name.into())
        };

        let docker = bollard::Docker::connect_with_local_defaults()?;

        // Pull the image (by creating an image from it)
        let image = format!("{}/{}:{}", owner, name, version);
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

        // Load and write the plugin file
        let mut plugin = Plugin::load(json)?;
        plugin.installation = Some(PluginInstallation::Docker);
        Plugin::replace(&name, &plugin)?;
        Ok(plugin)
    }

    /// Add a plugin as a soft link to a directory on the current machine
    ///
    /// # Arguments
    ///
    /// - `path`: Local file system path to the directory
    #[cfg(any(feature = "plugins-link"))]
    pub fn install_link(path: &str) -> Result<Plugin> {
        // Make the path absolute (for symlink to work)
        let path = fs::canonicalize(&path)?;

        // Check that the path is a directory
        if !path.is_dir() {
            bail!("Path must be a directory")
        }

        // Check that the directory has a plugin file
        let plugin_file = path.join(Plugin::FILE_NAME);
        if !plugin_file.is_file() {
            bail!("Directory must contain a '{}' file", Plugin::FILE_NAME)
        }

        // Check that the plugin's file can be loaded
        let json = fs::read_to_string(plugin_file)?;
        let mut plugin = Plugin::load(&json)?;
        plugin.installation = Some(PluginInstallation::Link);
        let name = plugin.name.as_str();

        // Remove the plugin directory
        Plugin::remove(name)?;

        // Create the soft link
        let link = Plugin::dir(name)?;
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        std::os::unix::fs::symlink(path, link)?;
        #[cfg(target_os = "windows")]
        std::os::windows::fs::symlink_dir(path, link)?;

        Ok(plugin)
    }

    /// Upgrade a plugin
    pub async fn upgrade(
        spec: &str,
        installs: &[PluginInstallation],
        aliases: &HashMap<String, String>,
        plugins: &mut Plugins,
    ) -> Result<()> {
        let aliases = Plugin::merge_aliases(&plugins.aliases, aliases);
        let name_or_spec = Plugin::alias_to_name(spec, &aliases);
        let (_owner, name, _version) = Plugin::spec_to_parts(&name_or_spec);

        let plugin = match plugins.plugins.get(name) {
            None => {
                tracing::info!(
                    "Plugin is not installed or registered, will attempt to install using spec: {}",
                    spec
                );
                return Plugin::install(spec, installs, &aliases, plugins, None).await;
            }
            Some(plugin) => plugin.clone(),
        };

        let (installs, current_version) = match plugin.installation {
            Some(install) => {
                tracing::debug!(
                    "Plugin will be upgraded from {} ({})",
                    plugin.software_version,
                    install
                );
                (vec![install], Some(plugin.software_version))
            }
            None => {
                tracing::debug!("Plugin is not yet installed");
                (Vec::from(installs), None)
            }
        };

        Plugin::install(spec, &installs, &aliases, plugins, current_version).await
    }

    /// Upgrade a list of plugins
    pub async fn upgrade_list(
        list: Vec<String>,
        installs: &[PluginInstallation],
        aliases: &HashMap<String, String>,
        plugins: &mut Plugins,
    ) -> Result<()> {
        let list = if list.is_empty() {
            plugins
                .plugins
                .iter()
                .filter(|(.., plugin)| plugin.installation.is_some())
                .map(|(name, ..)| name.clone())
                .collect()
        } else {
            list
        };
        for spec in list {
            match Plugin::upgrade(&spec, installs, aliases, plugins).await {
                Ok(_) => tracing::info!("Upgraded plugin {}", spec),
                Err(error) => bail!(error),
            }
        }
        Ok(())
    }

    /// Upgrade all installed plugins
    ///
    /// This is equivalent to calling `upgrade_list` with an empty list but
    /// requires fewer arguments. Intended for auto upgrades primarily.
    pub async fn upgrade_all(plugins: &mut Plugins) -> Result<()> {
        Plugin::upgrade_list(Vec::new(), &Vec::new(), &HashMap::new(), plugins).await
    }

    /// Uninstall a plugin
    ///
    /// Removes the plugin directory and marks it as not installed in the plugins
    /// in-memory store.
    pub fn uninstall(
        alias: &str,
        aliases: &HashMap<String, String>,
        plugins: &mut Plugins,
    ) -> Result<()> {
        let aliases = Plugin::merge_aliases(&plugins.aliases, aliases);
        let name = Plugin::alias_to_name(alias, &aliases);

        Plugin::remove(&name)?;
        plugins.uninstall(&name)?;

        Ok(())
    }

    /// Remove a list of plugins
    pub fn uninstall_list(
        list: Vec<String>,
        aliases: &HashMap<String, String>,
        plugins: &mut Plugins,
    ) -> Result<()> {
        for alias in list {
            match Plugin::uninstall(&alias, aliases, plugins) {
                Ok(_) => tracing::info!("Removed plugin {}", alias),
                Err(error) => bail!(error),
            }
        }
        Ok(())
    }

    /// Refresh a plugin
    pub async fn refresh(
        alias: &str,
        aliases: &HashMap<String, String>,
        plugins: &mut Plugins,
    ) -> Result<()> {
        let aliases = Plugin::merge_aliases(&plugins.aliases, aliases);
        let name = Plugin::alias_to_name(&alias, &aliases);

        let plugin = plugins.plugins.get(&name);

        // If the plugin is linked then there is nothing more to do
        // (we don't want to write anything into the directory)
        if let Some(plugin) = plugin {
            if let Some(PluginInstallation::Link) = plugin.installation {
                return Ok(());
            }
        }

        // Load the plugin's latest manifest from its URL
        let url = match plugins.registry.get(&name) {
            None => bail!("No plugin registered with alias or name '{}'", alias),
            Some(url) => url,
        };
        let json = reqwest::get(url).await?.text().await?;
        let latest = Plugin::load(&json)?;

        let mut plugin = if let Some(plugin) = plugin {
            // This plugin is previously known. if it is installed and
            // the latest version is greater than the current then, indicate
            // it can be upgraded using `next`, otherwise just use the latest version.
            if plugin.installation.is_some()
                && Version::parse(&latest.software_version)
                    > Version::parse(&plugin.software_version)
            {
                let mut plugin = plugin.clone();
                plugin.next = Some(Box::new(latest));
                plugin
            } else {
                latest
            }
        } else {
            // This plugin is previously "unknown" locally so just use the latest
            latest
        };

        // Write the plugin to disk and update in memory
        plugin.refreshed = Some(Utc::now());
        Plugin::write(&name, &plugin)?;
        plugins.plugins.insert(name, plugin);

        tracing::info!("Refreshed plugin {}", alias);
        Ok(())
    }

    /// Refresh a list of plugins
    pub async fn refresh_list(
        list: Vec<String>,
        aliases: &HashMap<String, String>,
        plugins: &mut Plugins,
    ) -> Result<()> {
        let list = if list.is_empty() {
            plugins.registry.keys().cloned().collect::<Vec<String>>()
        } else {
            list
        };

        for alias in list {
            if let Err(error) = Plugin::refresh(&alias, aliases, plugins).await {
                tracing::error!("When refreshing plugin {}: {}", alias, error)
            }
        }
        Ok(())
    }
}

/// An in-memory store for an implementation of a method
#[derive(Debug)]
struct MethodImplem {
    /// The name of the plugin which provides the implementation
    plugin: String,

    /// The plugin's JSON Schema which describes the method
    schema: Box<serde_json::Value>,

    /// The plugin's JSON Schema compiled
    #[allow(dead_code)]
    compiled_schema: JSONSchema<'static>,
}

/// An in-memory store of plugins and the methods that they implement
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Plugins {
    /// The global aliases for plugin names
    /// Can be overridden by local config.
    pub aliases: HashMap<String, String>,

    /// The global registry of plugins that maps their
    /// name to their manifest file
    pub registry: HashMap<String, String>,

    /// The plugins manifests
    pub plugins: HashMap<String, Plugin>,

    /// The methods that are implemented by installed plugins
    #[serde(skip)]
    methods: HashMap<String, Vec<MethodImplem>>,
}

/// In some cases it is necessary to clone the plugins store, e.g. when doing
/// a background update in a separate thread. In these cases just clone the plugins
/// info, not the derived methods.
impl Clone for Plugins {
    fn clone(&self) -> Self {
        Plugins {
            aliases: self.aliases.clone(),
            registry: self.registry.clone(),
            plugins: self.plugins.clone(),
            methods: HashMap::new(),
        }
    }
}

impl Plugins {
    /// Create an empty plugins store (mainly for testing)
    pub fn empty() -> Self {
        Plugins {
            aliases: HashMap::new(),
            registry: HashMap::new(),
            plugins: HashMap::new(),
            methods: HashMap::new(),
        }
    }

    /// Load the registry, aliases, and any plugin manifests
    pub fn load() -> Result<Self> {
        let mut plugins: Plugins = serde_json::from_str(include_str!("../../plugins.json"))?;

        // Create an entry for all plugins in the registry
        for (name, _url) in plugins.registry.clone() {
            let plugin = Plugin {
                name,
                ..Default::default()
            };
            plugins.add(plugin)?
        }

        // Add / update using any manifests that are stored locally in plugins directory
        let dir = dirs::plugins(true)?;
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let name = path.display().to_string();
                // Check this directory actually has a plugin file
                if Plugin::file(&name)?.exists() {
                    let plugin = Plugin::read(&name)?;
                    plugins.add(plugin)?
                }
            }
        }

        Ok(plugins)
    }

    /// Add / update a plugin
    ///
    /// Compiles the JSON Schema for each item in its `featureList`.
    pub fn add(&mut self, plugin: Plugin) -> Result<()> {
        let name = plugin.name.as_str();

        for feature in &plugin.feature_list {
            let title = match feature.get("title") {
                None => bail!("JSON Schema is missing 'title' property"),
                Some(serde_json::Value::String(value)) => value.clone(),
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
                    self.methods
                        .entry(title)
                        .or_insert_with(Vec::new)
                        .push(MethodImplem {
                            plugin: name.into(),
                            schema: schema_box_droppable,
                            compiled_schema,
                        });
                }
                Err(error) => {
                    tracing::warn!(
                        "Error compiling schema for method '{}' of plugin '{}'; will ignore, please let the plugin maintainer know: {}",
                        title, name, error
                    )
                }
            };
        }

        self.plugins.insert(name.into(), plugin);

        Ok(())
    }

    /// Uninstall a plugin
    ///
    /// Does not remove the plugin, simply marks it as not installed.
    pub fn uninstall(&mut self, name: &str) -> Result<()> {
        if let Some(mut plugin) = self.plugins.get_mut(name) {
            plugin.installation = None
        } else {
            tracing::warn!("Plugin with name '{}' is not known", name)
        }
        Ok(())
    }

    /// List all the installed plugins
    ///
    /// Populates the alias of the plugin based on the passed aliases map
    pub fn list_plugins(&self, aliases: &HashMap<String, String>) -> Vec<Plugin> {
        let aliases = Plugin::merge_aliases(&self.aliases, aliases);

        let mut plugins = self
            .plugins
            .iter()
            .map(|(name, plugin)| Plugin {
                alias: Some(Plugin::name_to_alias(&name, &aliases)),
                ..plugin.clone()
            })
            .collect::<Vec<Plugin>>();

        plugins.sort_by(|a, b| a.alias.cmp(&b.alias));
        plugins
    }

    /// Display an individual plugin
    pub fn display_plugin(
        &self,
        alias: &str,
        format: &str,
        aliases: &HashMap<String, String>,
    ) -> Result<String> {
        let aliases = Plugin::merge_aliases(&self.aliases, aliases);
        let name = Plugin::alias_to_name(alias, &aliases);

        match self.plugins.get(&name) {
            None => bail!("Plugin with name or alias '{}' is not loaded", alias),
            Some(plugin) => plugin.display(format),
        }
    }

    /// Create a Markdown table of all the registered and/or installed plugins
    pub fn display_plugins(&self, aliases: &HashMap<String, String>) -> Result<String> {
        let plugins = self.list_plugins(aliases);

        if plugins.is_empty() {
            return Ok("No plugins registered or installed.".to_string());
        }

        let head = r#"
| ----- | ------ | --------- | --------- | ----------- | ---------- |
|       | Plugin | Installed | Latest    | Description | Refreshed  |
| :---- | :----- | --------: | --------: | :---------- | ---------: |
    "#
        .trim();
        let body = plugins
            .iter()
            .map(|plugin| {
                let Plugin {
                    alias,
                    name,
                    software_version,
                    installation,
                    description,
                    next,
                    refreshed,
                    ..
                } = plugin.clone();
                let latest = match next {
                    // If not installed, then this is the latest known version
                    None => {
                        if installation.is_none() {
                            software_version.clone()
                        } else {
                            String::new()
                        }
                    }
                    // Use the version in `next`.
                    Some(next) => next.software_version,
                };
                let installation = match installation {
                    None => String::new(),
                    Some(value) => format!("{:>8} *{:>6}*", software_version, value),
                };
                let refreshed = match refreshed {
                    None => String::new(),
                    Some(refreshed) => {
                        use std::convert::TryFrom;
                        let duration = Utc::now() - refreshed;
                        format_duration(std::time::Duration::new(
                            u64::try_from(duration.num_seconds())
                                .expect("Unable to convert i64 to u64"),
                            0,
                        ))
                        .to_string()
                            + " ago"
                    }
                };
                format!(
                    "| **{}** | {} | {} | {} | {} | {} |",
                    alias.unwrap_or_else(|| name.clone()),
                    name,
                    installation,
                    latest,
                    description,
                    refreshed
                )
            })
            .collect::<Vec<String>>()
            .join("\n");
        let foot = "|-";
        Ok(format!("{}\n{}\n{}\n", head, body, foot))
    }

    /// Create a Markdown document describing a method and the plugins that implement it
    pub fn display_method(&self, name: &str, format: &str) -> Result<String> {
        let plugins = match self.methods.get(name) {
            None => bail!("No implementations for method `{}`", name),
            Some(implems) => implems
                .iter()
                .map(|method_implem| {
                    serde_json::json!({
                        "name": name,
                        "schema": method_implem.schema
                    })
                })
                .collect::<serde_json::Value>(),
        };

        let method = &serde_json::json!({ "name": name, "plugins": plugins });

        let content = match format {
            #[cfg(any(feature = "template-handlebars"))]
            "md" => {
                let template = r#"
# {{name}}

{{#each plugins}}
## {{name}}

{{#with schema }}
{{description}}

{{#each properties}}
- **{{@key}}**: *{{type}}* : {{description}}{{/each}}
{{/with}}
{{/each}}
    "#
                .trim();
                use handlebars::Handlebars;
                let hb = Handlebars::new();
                hb.render_template(template, &method)?
            }
            _ => serde_json::to_string_pretty(&method)?,
        };
        Ok(content)
    }

    /// Create a Markdown table of all the registered aliases
    pub fn display_aliases(&self, aliases: &HashMap<String, String>) -> Result<String> {
        let aliases = Plugin::merge_aliases(&self.aliases, aliases);
        if aliases.is_empty() {
            return Ok("No aliases registered".into());
        }

        let head = r#"
| -------- | -------- |
| Alias    | Plugin   |
| :------- | :------- |
    "#
        .trim();
        let body = aliases
            .iter()
            .map(|(alias, plugin)| format!("| {} | {} |", alias, plugin))
            .collect::<Vec<String>>()
            .join("\n");
        let foot = "|-";
        Ok(format!("{}\n{}\n{}\n", head, body, foot))
    }

    /// Create a Markdown table of all the registered methods
    pub fn display_methods(&self) -> Result<String> {
        if self.methods.is_empty() {
            return Ok("No methods registered.".into());
        }

        let head = r#"
| -------- | -------- |
| Method   | Plugins  |
| :------- | :------- |
    "#
        .trim();
        let body = self
            .methods
            .iter()
            .map(|method| {
                let (name, implems) = method;
                let plugins = implems
                    .iter()
                    .map(|plugin| plugin.plugin.clone())
                    .collect::<Vec<String>>()
                    .join(", ");
                format!("| {} | {} |", name, plugins)
            })
            .collect::<Vec<String>>()
            .join("\n");
        let foot = "|-";
        Ok(format!("{}\n{}\n{}\n", head, body, foot))
    }

    /// Delegate a method call to a particular plugin
    ///
    /// Note that this function does not do any validation of parameters against
    /// the plugin schema before sending a JSON-RPC request.
    pub async fn delegate_to(
        &self,
        _plugin: &str,
        _method: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        // TODO Check if the plugin has an existing client and create one if necessary
        Ok(params.clone())
    }

    /// Delegate a method call to any of the plugins
    pub async fn delegate(
        &self,
        _method: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        // TODO find a plugin with matching schema
        Ok(params.clone())
    }
}
#[cfg(feature = "config")]
pub mod config {
    use super::*;
    use defaults::Defaults;
    use validator::Validate;

    /// # Plugins
    ///
    /// Configuration settings for plugin installation and management
    #[derive(Debug, Defaults, PartialEq, Clone, JsonSchema, Deserialize, Serialize, Validate)]
    #[serde(default)]
    pub struct PluginsConfig {
        /// The order of preference of plugin installation method.
        #[def = "PluginInstallation::iter().collect()"]
        pub installations: Vec<PluginInstallation>,

        /// The local plugin aliases that extends and/or override those in the
        /// global aliases at https://github.com/stencila/stencila/blob/master/plugins.json
        pub aliases: HashMap<String, String>,
    }
}

#[cfg(feature = "cli")]
pub mod cli {
    use super::*;
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Manage plugins",
        setting = structopt::clap::AppSettings::ColoredHelp,
        setting = structopt::clap::AppSettings::VersionlessSubcommands
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
        #[structopt(
            about = "List installed plugins",
            setting = structopt::clap::AppSettings::ColoredHelp
        )]
        List,

        Show(Show),
        Install(Install),
        Link(Link),
        Upgrade(Upgrade),
        Uninstall(Uninstall),
        Unlink(Unlink),
        Refresh(Refresh),

        #[structopt(
            about = "List registered plugin aliases",
            setting = structopt::clap::AppSettings::ColoredHelp
        )]
        Aliases,

        #[structopt(
            about = "Get the JSON Schema for plugins",
            setting = structopt::clap::AppSettings::ColoredHelp
        )]
        Schema,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Show the details of an installed plugin",
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Show {
        /// The name of the plugin to show
        #[structopt()]
        pub plugin: String,

        /// The format to show the plugin in
        #[structopt(short, long, default_value = "md")]
        pub format: String,
    }

    #[derive(Debug, Default, StructOpt)]
    #[structopt(
        about = "Install one or more plugins",
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Install {
        /// Install plugins as Docker images
        #[structopt(short, long)]
        pub docker: bool,

        /// Install plugins as binaries
        #[structopt(short, long)]
        pub binary: bool,

        /// Install plugins as Javascript packages
        #[structopt(short, long)]
        pub javascript: bool,

        /// Install plugins as Python packages
        #[structopt(short, long)]
        pub python: bool,

        /// Install plugins as R packages
        #[structopt(short, long)]
        pub r: bool,

        /// Install plugins as soft links
        #[structopt(short, long)]
        pub link: bool,

        /// The names or aliases of plugins to add
        #[structopt(required = true, multiple = true)]
        pub plugins: Vec<String>,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Link to a local plugins",
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Link {
        /// The path of a plugin directory
        #[structopt()]
        pub path: String,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Upgrade one of more plugins",
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Upgrade {
        /// The names or aliases of plugins to upgrade
        /// (omit to upgrade all plugins)
        #[structopt(multiple = true)]
        pub plugins: Vec<String>,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Uninstall one or more plugins",
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Uninstall {
        /// The names or aliases of plugins to uninstall
        #[structopt(required = true, multiple = true)]
        pub plugins: Vec<String>,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Unlink a local plugins",
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Unlink {
        /// The name of the plugin to unlink
        #[structopt()]
        pub plugin: String,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Refresh details of one or more plugins",
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Refresh {
        /// The names or aliases of plugins to refresh (leave blank for all)
        #[structopt(required = false, multiple = true)]
        pub plugins: Vec<String>,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "List methods and the plugins that implement them",
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Methods {
        /// The name of the method to display
        #[structopt()]
        pub method: Option<String>,

        /// The format to show the method using
        #[structopt(short, long, default_value = "md")]
        pub format: String,
    }

    impl Methods {
        pub async fn run(&self, plugins: &mut Plugins) -> Result<()> {
            let Methods { method, format } = self;

            let content = match method {
                None => plugins.display_methods()?,
                Some(method) => plugins.display_method(&method, &format)?,
            };
            if format == "json" {
                println!("{}", content)
            } else {
                let skin = termimad::MadSkin::default();
                println!("{}", skin.term_text(content.as_str()))
            }
            Ok(())
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Delegate a method call to any, or a particular, plugin",
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Delegate {
        /// The method to call
        #[structopt()]
        pub method: String,

        /// The plugin to delegate to
        #[structopt()]
        pub plugin: Option<String>,

        /// Method parameters (after `--`) as strings (e.g. `format=json`) or JSON (e.g. `node:='{"type":...}'`)
        #[structopt(raw(true))]
        params: Vec<String>,
    }

    impl Delegate {
        pub async fn run(&self, plugins: &mut Plugins) -> Result<()> {
            let Delegate {
                method,
                plugin,
                params,
            } = self;
            let params = crate::util::params::parse(params);
            let result = match plugin {
                Some(plugin) => plugins.delegate_to(&plugin, &method, &params).await?,
                None => plugins.delegate(&method, &params).await?,
            };
            println!("{}", serde_json::to_string_pretty(&result)?);

            Ok(())
        }
    }

    pub async fn run(
        args: Args,
        config: &config::PluginsConfig,
        plugins: &mut Plugins,
    ) -> Result<()> {
        let Args { action } = args;
        let config::PluginsConfig {
            aliases,
            installations,
        } = config;

        let skin = termimad::MadSkin::default();
        match action {
            Action::List => {
                let md = plugins.display_plugins(aliases)?;
                println!("{}", skin.term_text(md.as_str()));
                Ok(())
            }
            Action::Show(action) => {
                let Show { plugin, format } = action;

                let content = plugins.display_plugin(&plugin, &format, aliases)?;
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
                    javascript,
                    python,
                    r,
                    link,
                    plugins: list,
                } = action;

                let mut installs = vec![];
                if docker {
                    installs.push(PluginInstallation::Docker)
                }
                if binary {
                    installs.push(PluginInstallation::Binary)
                }
                if javascript {
                    installs.push(PluginInstallation::Javascript)
                }
                if python {
                    installs.push(PluginInstallation::Python)
                }
                if r {
                    installs.push(PluginInstallation::R)
                }
                if link {
                    installs.push(PluginInstallation::Link)
                }

                let installs = if installs.is_empty() {
                    &installations
                } else {
                    &installs
                };

                Plugin::install_list(list, installs, aliases, plugins).await
            }
            Action::Link(action) => {
                let Link { path } = action;

                Plugin::install(&path, &[PluginInstallation::Link], aliases, plugins, None).await
            }
            Action::Upgrade(action) => {
                let Upgrade { plugins: list } = action;

                Plugin::upgrade_list(list, &installations, aliases, plugins).await
            }
            Action::Uninstall(action) => {
                let Uninstall { plugins: list } = action;

                Plugin::uninstall_list(list, aliases, plugins)
            }
            Action::Unlink(action) => {
                let Unlink { plugin } = action;

                Plugin::uninstall(&plugin, aliases, plugins)
            }
            Action::Refresh(action) => {
                let Refresh { plugins: list } = action;

                Plugin::refresh_list(list, aliases, plugins).await
            }
            Action::Aliases => {
                let md = plugins.display_aliases(aliases)?;
                println!("{}", skin.term_text(md.as_str()));
                Ok(())
            }
            Action::Schema => {
                println!("{}", schema());
                Ok(())
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

        let config = config::PluginsConfig {
            ..Default::default()
        };
        let mut plugins = Plugins::empty();

        run(
            Args {
                action: Action::List,
            },
            &config,
            &mut plugins,
        )
        .await?;

        run(
            Args {
                action: Action::Show(Show {
                    plugin: "foo".to_string(),
                    format: "md".to_string(),
                }),
            },
            &config,
            &mut plugins,
        )
        .await
        .expect_err("Expected an error!");

        run(
            Args {
                action: Action::Install(Install {
                    plugins: vec![],
                    ..Default::default()
                }),
            },
            &config,
            &mut plugins,
        )
        .await?;

        run(
            Args {
                action: Action::Link(Link {
                    path: "../foo".to_string(),
                }),
            },
            &config,
            &mut plugins,
        )
        .await
        .expect_err("Expected an error!");

        run(
            Args {
                action: Action::Upgrade(Upgrade { plugins: vec![] }),
            },
            &config,
            &mut plugins,
        )
        .await?;

        run(
            Args {
                action: Action::Uninstall(Uninstall { plugins: vec![] }),
            },
            &config,
            &mut plugins,
        )
        .await?;

        Ok(())
    }
}
