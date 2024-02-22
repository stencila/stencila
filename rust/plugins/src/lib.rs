use std::{
    collections::HashMap,
    env,
    fs::create_dir_all,
    net::TcpListener,
    path::{Path, PathBuf},
    process::Stdio,
    str::FromStr,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use app::{get_app_dir, DirType};
use semver::{Version, VersionReq};

use common::{
    eyre::{bail, eyre, OptionExt, Report, Result},
    itertools::Itertools,
    rand::{distributions::Alphanumeric, thread_rng, Rng},
    reqwest::{self, header, Client, Url},
    serde::{self, Deserialize, Deserializer, Serialize, Serializer},
    serde_json::{self, Value},
    serde_with::{DeserializeFromStr, SerializeDisplay},
    strum::{Display, EnumString},
    tokio::{
        self,
        io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
        process::{Child, ChildStdin, ChildStdout, Command},
    },
    toml,
    which::which,
};

mod check;
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

    /// The name of the message transport protocols that the plugin supports
    #[serde(
        alias = "transport",
        default,
        deserialize_with = "Plugin::deserialize_transports"
    )]
    transports: Vec<PluginTransport>,

    /// The command to run the plugin
    command: String,
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

    /// Deserialize the supported transports for a plugin
    fn deserialize_transports<'de, D>(deserializer: D) -> Result<Vec<PluginTransport>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged, crate = "common::serde")]
        enum OneOrMany {
            One(PluginTransport),
            Many(Vec<PluginTransport>),
        }

        Ok(match OneOrMany::deserialize(deserializer)? {
            OneOrMany::One(one) => vec![one],
            OneOrMany::Many(many) => many,
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

    /// Start an instance of a plugin
    async fn start(&self, transport: Option<PluginTransport>) -> Result<PluginInstance> {
        PluginInstance::start(self, transport).await
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

    /// Get the version of the runtime
    fn version(&self) -> Result<Version> {
        let path = self.path()?;

        let output = std::process::Command::new(path).arg("--version").output()?;
        let output = String::from_utf8(output.stdout)?;

        let version = match &self {
            PluginRuntime::Python => output.strip_prefix("Python "),
            PluginRuntime::Node => output.strip_prefix('v'),
        }
        .ok_or_else(|| eyre!("Unable to extract version from output"))?
        .trim();

        let version = Version::parse(version)
            .map_err(|error| eyre!("Unable to parse version `{version}`: {error}"))?;

        Ok(version)
    }

    /// Install a plugin
    async fn install(&self, url: &Url, dir: &Path) -> Result<()> {
        match self {
            PluginRuntime::Node => Self::install_node(url, dir).await,
            PluginRuntime::Python => Self::install_python(url, dir).await,
        }
    }

    /// Install a Node.js plugin
    async fn install_node(url: &Url, dir: &Path) -> Result<()> {
        let mut child = Command::new("npm")
            .arg(format!("--prefix={}", dir.to_string_lossy()))
            .arg("install")
            .arg(url.to_string())
            .spawn()?;

        let status = child.wait().await?;
        if !status.success() {
            bail!("Install of Node.js plugin failed")
        }

        Ok(())
    }

    /// Install a Python plugin
    async fn install_python(url: &Url, dir: &Path) -> Result<()> {
        bail!("TODO")
    }
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

/// The message transport protocols that a plugin supports
#[derive(
    Debug, Display, Clone, EnumString, DeserializeFromStr, SerializeDisplay, PartialEq, Eq,
)]
#[strum(
    ascii_case_insensitive,
    serialize_all = "lowercase",
    crate = "common::strum"
)]
#[serde_with(crate = "common::serde_with")]
pub enum PluginTransport {
    Stdio,
    Http,
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

/// A running instance of a plugin
pub struct PluginInstance {
    /// The plugin child process
    child: Child,

    /// The transport used to exchange JSON-RPC messages with the plugin
    transport: PluginTransport,

    /// The client and URL to use if the transport is HTTP
    http_client: Option<(Client, Url)>,

    /// The stdin & stdout streams to use if the transport is stdio
    stdio_streams: Option<(BufWriter<ChildStdin>, BufReader<ChildStdout>)>,
}

impl PluginInstance {
    /// Start a plugin instance
    async fn start(plugin: &Plugin, transport: Option<PluginTransport>) -> Result<Self> {
        let mut args = plugin.command.split(' ').collect_vec();
        let program = args.remove(0);
        let dir = Plugin::plugin_dir(&plugin.name, false)?;

        let mut command = Command::new(program);
        command
            .args(args)
            .current_dir(dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let transport = transport
            .or_else(|| {
                if plugin.transports.contains(&PluginTransport::Stdio) {
                    Some(PluginTransport::Stdio)
                } else {
                    plugin.transports.get(0).cloned()
                }
            })
            .ok_or_else(|| eyre!("Plugin does not declare any transports"))?;
        command.env("STENCILA_TRANSPORT", transport.to_string());

        let http_client = if transport == PluginTransport::Http {
            let mut rng = thread_rng();
            let mut port: u16;
            loop {
                // Generate a random port number within the IANA recommended range for dynamic
                // or private ports and attempt to bind to it to check if it's available
                port = rng.gen_range(49152..=65535);
                match TcpListener::bind(("127.0.0.1", port)) {
                    Ok(_) => break,     // If binding succeeds, the port is likely available
                    Err(_) => continue, // If binding fails, try another port
                }
            }
            command.env("STENCILA_PORT", port.to_string());

            let token: String = rng
                .sample_iter(&Alphanumeric)
                .take(36)
                .map(char::from)
                .collect();
            command.env("STENCILA_TOKEN", token.clone());

            let mut headers = header::HeaderMap::new();
            let mut auth_value = header::HeaderValue::try_from(format!("Bearer {token}"))?;
            auth_value.set_sensitive(true);
            headers.insert(header::AUTHORIZATION, auth_value);

            let client = reqwest::Client::builder()
                .default_headers(headers)
                .connect_timeout(Duration::from_millis(10000))
                .build()?;

            let url = Url::parse(&format!("http://127.0.0.1:{}", port))?;

            Some((client, url))
        } else {
            None
        };

        let mut child = command.spawn()?;

        let stdio_streams = if transport == PluginTransport::Stdio {
            // Create streams for input, output and errors
            let stdin = child.stdin.take().ok_or_eyre("Child has no stdin handle")?;
            let stdout = child
                .stdout
                .take()
                .ok_or_eyre("Child has no stdout handle")?;

            // Create stream readers and writers
            let stdin_writer = BufWriter::new(stdin);
            let stdout_reader = BufReader::new(stdout);

            Some((stdin_writer, stdout_reader))
        } else {
            None
        };

        // TODO: instead of waiting here for server to start do retires in `call_http`
        if transport == PluginTransport::Http {
            tokio::time::sleep(Duration::from_millis(5000)).await;
        }

        Ok(Self {
            child,
            transport,
            http_client,
            stdio_streams,
        })
    }

    /// Stop the plugin instance
    async fn stop(&mut self) -> Result<()> {
        self.child.kill().await?;

        Ok(())
    }

    /// Call a method of the plugin instance
    async fn call(&mut self, request: JsonRpcRequest) -> Result<Value> {
        let response = match self.transport {
            PluginTransport::Stdio => self.call_stdio(&request).await,
            PluginTransport::Http => self.call_http(&request).await,
        }?;

        if response.id != request.id {
            bail!("Response id does not match request id")
        }

        match response.result {
            JsonRpcResult::Success { result } => Ok(result),
            JsonRpcResult::Error { error } => bail!("{}", error.message),
        }
    }

    /// Call a method of the plugin instance via stdio
    async fn call_stdio(&mut self, request: &JsonRpcRequest) -> Result<JsonRpcResponse> {
        let (writer, reader) = self
            .stdio_streams
            .as_mut()
            .ok_or_else(|| eyre!("Stdio stream uninitialized"))?;

        let request_json = serde_json::to_string(&request)? + "\n";
        writer.write_all(request_json.as_bytes()).await?;
        writer.flush().await?;

        let Some(response_json) = reader.lines().next_line().await? else {
            bail!("No response line")
        };

        Ok(serde_json::from_str(&response_json)?)
    }

    /// Call a method of the plugin instance via HTTP
    async fn call_http(&self, request: &JsonRpcRequest) -> Result<JsonRpcResponse> {
        let (client, url) = self
            .http_client
            .as_ref()
            .ok_or_else(|| eyre!("HTTP client uninitialized"))?;

        let response = client.post(url.clone()).json(&request).send().await?;

        if let Err(error) = response.error_for_status_ref() {
            let message = response.text().await?;
            bail!("{error}: {message}");
        }

        Ok(response.json().await?)
    }

    /// Check the health of the plugin instance
    async fn health(&mut self) -> Result<()> {
        self.call(JsonRpcRequest::new("health", vec![])).await?;

        Ok(())
    }
}

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Serialize)]
#[serde(crate = "common::serde")]
pub struct JsonRpcRequest {
    id: u64,
    jsonrpc: String,
    method: String,
    params: Vec<Value>,
}

impl JsonRpcRequest {
    pub fn new(method: &str, params: Vec<Value>) -> Self {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: REQUEST_COUNTER.fetch_add(1, Ordering::SeqCst),
            method: method.to_string(),
            params,
        }
    }
}

#[allow(unused)]
#[derive(Deserialize)]
#[serde(crate = "common::serde")]
struct JsonRpcResponse {
    jsonrpc: Option<String>,
    id: u64,
    #[serde(flatten)]
    result: JsonRpcResult,
}

#[derive(Deserialize)]
#[serde(crate = "common::serde")]
#[serde(untagged)]
enum JsonRpcResult {
    Success { result: Value },
    Error { error: JsonRpcError },
}

#[allow(unused)]
#[derive(Deserialize)]
#[serde(crate = "common::serde")]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<Value>,
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
transports = ["stdio", "http"]
command = "python ..."
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
transports = ["http"]
command = "python ..."
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
