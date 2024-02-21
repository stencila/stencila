use std::str::FromStr;

use semver::{Version, VersionReq};

use common::{
    eyre::Result,
    itertools::Itertools,
    reqwest::Url,
    serde::{self, Deserialize, Deserializer, Serialize, Serializer},
    serde_with::{DeserializeFromStr, SerializeDisplay},
    strum::{Display, EnumString},
};

pub mod cli;
mod list;

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
    #[serde(alias = "platform", deserialize_with = "Plugin::deserialize_platforms")]
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

/// The availability of a plugin on the current machine
///
/// Determined based on the `runtimes` and `platforms` properties
/// of the plugin and the runtimes and platform of the current machine.
#[derive(Debug, Display, PartialEq, Eq)]
#[strum(serialize_all = "lowercase")]
pub enum PluginAvailability {
    /// Available on this machine
    Installed,
    /// Available on this machine but requires installation
    Installable,
    /// Not available on this machine (e.g. language runtime not available)
    Unavailable,
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
