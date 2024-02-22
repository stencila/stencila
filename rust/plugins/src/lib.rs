use std::{collections::HashMap, env, fs::create_dir_all, path::PathBuf, str::FromStr};

use app::{get_app_dir, DirType};
use semver::{Version, VersionReq};

use common::{
    eyre::{bail, eyre, Report, Result},
    itertools::Itertools,
    reqwest::{self, Url},
    serde::{self, Deserialize, Deserializer, Serialize, Serializer},
    serde_with::{DeserializeFromStr, SerializeDisplay},
    strum::{Display, EnumString},
    toml,
    which::which,
};

pub mod cli;
mod install;
mod list;
mod uninstall;

/// The specification of a plugin
///
/// This specification
#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "common::serde")]
pub struct Plugin {
    /// The name of the plugin, should be unique across all plugins
    name: String,

    /// The version of the plugin
    #[serde(
        deserialize_with = "Plugin::deserialize_version",
        serialize_with = "Plugin::serialize_version"
    )]
    version: Version,

    /// A brief description of the plugin
    description: String,

    /// The home page for the plugin
    #[serde(
        deserialize_with = "Plugin::deserialize_url",
        serialize_with = "Plugin::serialize_url"
    )]
    home: Url,

    /// The installation URL for the plugin
    #[serde(
        deserialize_with = "Plugin::deserialize_url",
        serialize_with = "Plugin::serialize_url"
    )]
    install: Url,

    /// The name of the language runtimes that the plugin supports
    ///
    /// If empty, assumed to not require any runtime, i.e. installed as
    /// an standalone executable binary.
    #[serde(
        alias = "runtime",
        deserialize_with = "Plugin::deserialize_runtimes",
        serialize_with = "Plugin::serialize_runtimes"
    )]
    runtimes: Vec<(PluginRuntime, VersionReq)>,

    /// The name of the operating system platforms that the plugin supports
    ///
    /// If empty, assumed to work on all platforms.
    #[serde(
        alias = "platform",
        default,
        deserialize_with = "Plugin::deserialize_platforms"
    )]
    platforms: Vec<PluginPlatform>,
}

impl Plugin {
    /// Deserialize the version string for a plugin
    fn deserialize_version<'de, D>(deserializer: D) -> Result<Version, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let string = String::deserialize(deserializer)?;
        Version::parse(&string)
            .map_err(|error| Error::custom(format!("invalid plugin version: {error}")))
    }

    /// Serialize the version of a plugin
    fn serialize_version<S>(version: &Version, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&version.to_string())
    }

    /// Deserialize a URL for a plugin
    fn deserialize_url<'de, D>(deserializer: D) -> Result<Url, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let string = String::deserialize(deserializer)?;
        Url::parse(&string).map_err(|error| Error::custom(format!("invalid plugin URL: {error}")))
    }

    /// Serialize a URL for a plugin
    fn serialize_url<S>(url: &Url, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&url.to_string())
    }

    /// Deserialize the supported runtimes specifications for a plugin
    ///
    /// Supports the following TOML syntaxes (and the equivalents for JSON and YAML):
    ///
    ///   runtime = "python"
    ///   runtimes = "Python >=3"
    ///   runtimes = ["node 20.1"]
    fn deserialize_runtimes<'de, D>(
        deserializer: D,
    ) -> Result<Vec<(PluginRuntime, VersionReq)>, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        #[derive(Deserialize)]
        #[serde(untagged, crate = "common::serde")]
        enum OneOrMany {
            One(String),
            Many(Vec<String>),
        }

        let mut runtimes = Vec::new();
        for string in match Option::<OneOrMany>::deserialize(deserializer)? {
            None => vec![],
            Some(OneOrMany::One(one)) => vec![one],
            Some(OneOrMany::Many(many)) => many,
        } {
            let index = string
                .find(|c: char| !c.is_alphanumeric())
                .unwrap_or(string.len());
            let (first, second) = string.split_at(index);

            let runtime = PluginRuntime::from_str(first)
                .map_err(|error| Error::custom(format!("invalid runtime time: {error}")))?;

            let version_req = if !second.trim().is_empty() {
                VersionReq::parse(second).map_err(|error| {
                    Error::custom(format!(
                        "invalid runtime version requirement `{second}`: {error}"
                    ))
                })?
            } else {
                VersionReq::STAR
            };

            runtimes.push((runtime, version_req));
        }

        Ok(runtimes)
    }

    /// Serialize the supported runtimes specifications for a plugin
    fn serialize_runtimes<S>(
        runtimes: &Vec<(PluginRuntime, VersionReq)>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        runtimes
            .iter()
            .map(|(runtime, version_req)| format!("{runtime} {version_req}"))
            .collect_vec()
            .serialize(serializer)
    }

    /// Deserialize the supported platforms for a plugin
    fn deserialize_platforms<'de, D>(deserializer: D) -> Result<Vec<PluginPlatform>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged, crate = "common::serde")]
        enum OneOrMany {
            One(PluginPlatform),
            Many(Vec<PluginPlatform>),
        }

        Ok(match Option::<OneOrMany>::deserialize(deserializer)? {
            None => vec![],
            Some(OneOrMany::One(one)) => vec![one],
            Some(OneOrMany::Many(many)) => many,
        })
    }

    /// Fetch the latest registry list of plugins from the Stencila repo
    pub async fn fetch_registry() -> Result<HashMap<String, String>> {
        // TODO: change URL to point to `main` before PR is merged
        const PLUGINS_TOML_URL: &str =
            "https://raw.githubusercontent.com/stencila/stencila/feature/plugins/plugins.toml";

        let response = reqwest::get(PLUGINS_TOML_URL).await?;
        if let Err(error) = response.error_for_status_ref() {
            let message = response.text().await?;
            bail!("{error}: {message}");
        }

        let toml = response.text().await?;
        Ok(toml::from_str(&toml)?)
    }

    /// Fetch the manifest for a plugin using its URL in the registry
    pub async fn fetch_manifest(name: &str, url: &str) -> Result<Self> {
        {
            let response = reqwest::get(url).await?;
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
        .map_err(|error| eyre!("Error fetching manifest for plugin `{name}`: {error}"))
    }

    /// Get the directory for a plugin
    pub fn plugin_dir(name: &str, ensure: bool) -> Result<PathBuf> {
        let dir = get_app_dir(DirType::Plugins, false)?.join(name);

        if ensure {
            create_dir_all(&dir)?;
        }

        Ok(dir)
    }

    /// Read the plugin from its installed manifest
    pub fn read_manifest(name: &str) -> Result<Self> {
        let manifest = Plugin::plugin_dir(name, false)?.join("manifest.toml");
        let manifest = std::fs::read_to_string(manifest)?;
        Ok(toml::from_str(&manifest)?)
    }

    /// The availability of the plugin on the current machine
    pub fn availability(&self) -> PluginStatus {
        // Check if already installed and if so if up-to-date
        if let Ok(installed) = Plugin::read_manifest(&self.name) {
            return if installed.version == self.version {
                PluginStatus::InstalledLatest(self.version.clone())
            } else {
                PluginStatus::InstalledOutdated(installed.version, self.version.clone())
            };
        };

        // Check if available on the current platform
        if !self.platforms.is_empty() {
            let Ok(current_platform) = PluginPlatform::current() else {
                return PluginStatus::UnavailablePlatform;
            };
            if !self.platforms.contains(&current_platform) {
                return PluginStatus::UnavailablePlatform;
            }
        }

        // Check if runtime is available
        for (runtime, ..) in &self.runtimes {
            if runtime.is_available() {
                return PluginStatus::Installable;
            }
        }

        PluginStatus::UnavailableRuntime
    }
}

/// A runtime that a plugin supports
#[derive(Debug, Display, EnumString, PartialEq, Eq)]
#[strum(
    ascii_case_insensitive,
    serialize_all = "lowercase",
    crate = "common::strum"
)]
pub enum PluginRuntime {
    Python,
    Node,
}

impl PluginRuntime {
    /// Get the path of the runtime executable
    fn path(&self) -> Result<PathBuf> {
        let name = if cfg!(windows) {
            format!("{self}.exe")
        } else {
            self.to_string()
        };

        Ok(which(name)?)
    }

    /// Is the runtime available of the current machine
    fn is_available(&self) -> bool {
        self.path().is_ok()
    }

    /*
    /// Get the version of the runtime
    fn version(&self) -> Result<String> {
        let path = self.path()?;

        let child = Command

        let version = match &self {
            PluginRuntime::Python => {
                output.splitn(2, ' ').nth(1)
            },
            PluginRuntime::Node => {

            }
        }
    }*/
}

/// An operating system platform that a plugin supports
#[derive(Debug, Display, EnumString, DeserializeFromStr, SerializeDisplay, PartialEq, Eq)]
#[strum(
    ascii_case_insensitive,
    serialize_all = "lowercase",
    crate = "common::strum"
)]
#[serde_with(crate = "common::serde_with")]
pub enum PluginPlatform {
    Linux,
    MacOS,
    Windows,
}

impl PluginPlatform {
    /// Get the current operating system platform
    fn current() -> Result<Self> {
        Ok(match env::consts::OS {
            "linux" => Self::Linux,
            "macos" => Self::MacOS,
            "windows" => Self::Windows,
            _ => bail!("Unhandled operating system `{}`", env::consts::OS),
        })
    }
}

/// The status of a plugin on the current machine
///
/// Install-ability determined based on the `runtimes` and `platforms` properties
/// of the plugin and the runtimes and platform of the current machine.
#[derive(Debug, Display, Clone, PartialEq, Eq)]
#[strum(serialize_all = "lowercase")]
pub enum PluginStatus {
    /// Latest version installed
    InstalledLatest(Version),

    /// An outdated version is installed
    InstalledOutdated(Version, Version),

    /// Available on this machine but requires installation
    Installable,

    /// Required runtime not available
    #[strum(to_string = "requires runtime installation")]
    UnavailableRuntime,

    /// Not available on this operating system platform
    #[strum(to_string = "unavailable on this operating system")]
    UnavailablePlatform,
}

#[cfg(test)]
mod tests {
    use super::*;

    use common::toml;
    use common_dev::pretty_assertions::assert_eq;

    #[test]
    fn deserialization() -> Result<()> {
        let plugin: Plugin = toml::from_str(
            r#"
name = "super-plugin"
version = "0.1.0"
description = "Does super stuff"
home = "https://examples.org/stencila-super-plugin"
install = "https://pypi.org/project/stencila-super-plugin"
runtime = "python >=3,<4"
platforms = ["Linux", "MacOS"]
"#,
        )?;
        assert_eq!(plugin.name, "super-plugin");
        assert_eq!(plugin.description, "Does super stuff");
        assert_eq!(
            plugin.runtimes,
            vec![(PluginRuntime::Python, VersionReq::parse(">=3,<4")?)]
        );
        assert_eq!(
            plugin.platforms,
            vec![PluginPlatform::Linux, PluginPlatform::MacOS]
        );

        let plugin: Plugin = toml::from_str(
            r#"
name = "windows-only"
version = "1.0.0-alpha.23"
description = "Only available under Python on Windows"
home = "https://examples.org/windows-only"
install = "https://github.com/example/windows-only"
runtimes = ["Python", "python >=3"]
platform = "windows"
"#,
        )?;
        assert_eq!(
            plugin.runtimes,
            vec![
                (PluginRuntime::Python, VersionReq::parse("*")?),
                (PluginRuntime::Python, VersionReq::parse(">=3")?)
            ]
        );
        assert_eq!(plugin.platforms, vec![PluginPlatform::Windows]);

        Ok(())
    }
}
