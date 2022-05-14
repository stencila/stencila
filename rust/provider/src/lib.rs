use std::{env, path::Path};

use async_trait::async_trait;
use chrono::Utc;
use eyre::Result;
use http_utils::http::{Request, Response, StatusCode};
use node_address::Address;
use node_pointer::{walk, Visitor};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use stencila_schema::{InlineContent, Node};
use strum::{AsRefStr, EnumString, EnumVariantNames};
use tokio::{
    sync::mpsc,
    time::{timeout, Duration},
};

// Export and re-export for the convenience of crates that implement a provider
pub use ::async_trait;
pub use ::chrono;
pub use ::codecs;
pub use ::eyre;
pub use ::futures;
pub use ::http_utils;
pub use ::once_cell;
pub use ::regex;
pub use ::serde;
pub use ::serde_json;
pub use ::stencila_schema;
pub use ::strum;
pub use ::tokio;
pub use ::tracing;

pub const IMPORT: &str = "import";
pub const EXPORT: &str = "export";
pub const IMPORT_EXPORT: &str = "import/export";
pub const ACTIONS: &[&str] = &[IMPORT, EXPORT, IMPORT_EXPORT];

/// A specification for providers
///
/// All providers, including those implemented in plugins, should provide this
/// specification. Rust implementations return a `Provider` instance from the
/// `spec` function of `ProviderTrait`. Plugins provide a JSON or YAML serialization
/// as part of their manifest.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Provider {
    /// The name of the provider
    pub name: String,
}

impl Provider {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            name: name.as_ref().to_string(),
        }
    }
}

/// A trait for providers
///
/// This trait can be used by Rust implementations of providers, allowing them to
/// be compiled into the Stencila binaries.
#[async_trait]
pub trait ProviderTrait {
    /// Get the [`Provider`] specification
    fn spec() -> Provider;

    /// Parse a string into a node
    fn parse(_string: &str) -> Vec<ParseItem> {
        Vec::new()
    }

    /// Detect nodes within a root node that the provider may be able to identify and enrich.
    ///
    /// Returns a vector of [`Detection`].
    async fn detect(root: &Node) -> Result<Vec<DetectItem>> {
        let name = Self::spec().name;
        let parse = Box::new(|string: &str| Self::parse(string));

        let mut detector = Detector::new(name, parse);
        walk(root, &mut detector);
        Ok(detector.detections)
    }

    /// Does the provider recognize a node?
    fn recognize(_node: &Node) -> bool {
        false
    }

    /// Identify a node
    ///
    /// The node is supplied to the provider, with one or more properties populated.
    /// The provider then attempts to identify the node based on those properties,
    /// and if it was able to do so, returns a copy of the node with one or more identifying
    /// properties populated (e.g. the `GithubProvider` might populate the `codeRepository` property
    /// of a `SofwareSourceCode` node).
    async fn identify(node: Node) -> Result<Node> {
        Ok(node)
    }

    /// Enrich a node
    ///
    /// If the provider had previously identified the node, then the relevant identifiers
    /// will be used to fetch enrichment data, otherwise `identify` will be called.
    /// Then, the provider will return a opy of the node with properties that are missing.
    async fn enrich(node: Node, _options: Option<EnrichOptions>) -> Result<Node> {
        Ok(node)
    }

    /// Import content from a remote [`Node`] (e.g. an `Article` or `SoftwareSourceCode` repository) to a local path
    async fn import(_node: &Node, _path: &Path, _options: Option<ImportOptions>) -> Result<()> {
        Ok(())
    }

    /// Export content from a local path to a remote [`Node`] (e.g. an `Article` or `SoftwareSourceCode` repository)
    async fn export(_node: &Node, _path: &Path, _options: Option<ExportOptions>) -> Result<()> {
        Ok(())
    }

    /// Synchronize changes between a remote [`Node`] (e.g. a `SoftwareSourceCode` repository) and a local
    /// destination path (a file or directory)
    async fn sync(
        _node: &Node,
        _path: &Path,
        _request: &Request<serde_json::Value>,
        _options: Option<SyncOptions>,
    ) -> Result<Response<String>> {
        let message =
            "Provider received a sync request but does not yet implement handling of those"
                .to_string();
        tracing::error!("{}", message);
        let response = Response::builder()
            .status(StatusCode::NOT_IMPLEMENTED)
            .body(message)?;
        Ok(response)
    }

    /// Schedule import and/or export to/from a remove [`Node`] and a local path
    async fn cron(
        _node: &Node,
        _path: &Path,
        _action: &str,
        _schedule: &str,
        _canceller: mpsc::Receiver<()>,
    ) -> Result<()> {
        Ok(())
    }
}

/// Resolve a string to an access token
///
/// If the string looks like an environment variable (is `UPPER_SNAKE_CASE`), and that variable exists
/// then will return the value of that variable. Otherwise will return the input string.
pub fn resolve_token(token: &str) -> Option<String> {
    static ENV_VAR_REGEX: Lazy<Regex> =
        Lazy::new(|| Regex::new("[A-Z_]+").expect("Unable to create regex"));

    if ENV_VAR_REGEX.is_match(token) {
        match env::var(&token) {
            Ok(value) => Some(value),
            Err(..) => {
                tracing::debug!("Token string appears to be an environment variable name but no such variable found");
                None
            }
        }
    } else {
        Some(token.to_string())
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EnrichOptions {
    pub token: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ImportOptions {
    /// The token required to access the resource (or the `ALL_CAPS` name of the
    /// environment variable containing the token)
    pub token: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    /// The token required to access the resource (or the `ALL_CAPS` name of the
    /// environment variable containing the token)
    pub token: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SyncOptions {
    /// The synchronization mode
    pub mode: Option<WatchMode>,

    /// The token required to access the resource (or the `ALL_CAPS` name of the
    /// environment variable containing the token)
    pub token: Option<String>,
}

#[derive(Debug, Clone, AsRefStr, EnumString, EnumVariantNames, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WatchMode {
    /// Synchronize the resource whenever it has been changed
    #[strum(serialize = "changed", serialize = "change")]
    Changed,

    /// Synchronize the resource whenever it has been committed (e.g. using `git commit`)
    #[strum(serialize = "committed", serialize = "commit")]
    Committed,

    /// Synchronize the resource whenever it is tagged (e.g. a git tag is added, or a Google file has a revision made)
    #[strum(serialize = "tagged", serialize = "tag")]
    Tagged,
}

impl Default for WatchMode {
    fn default() -> Self {
        WatchMode::Changed
    }
}

#[derive(Debug, Serialize)]
pub struct ParseItem {
    /// The start position in the string that the node was parsed from
    pub begin: usize,

    /// The end position in the string that the node was parsed from
    pub end: usize,

    /// The parsed [`Node`] usually with some properties populated
    pub node: Node,
}

#[derive(Debug, Serialize)]
pub struct DetectItem {
    /// The name of the provider that detected the node
    pub provider: String,

    /// The percent confidence in the detection (0-100)
    pub confidence: u32,

    /// The [`Address`], within the node tree, that the node detected node begins
    pub begin: Address,

    /// The [`Address`], within the node tree, that the node detected node ends
    pub end: Address,

    /// The detected [`Node`] usually with some properties populated (i.e. those
    /// properties that were used to detect it)
    pub node: Node,
}

pub struct Detector {
    /// The name of the provider that this detector is for
    provider: String,

    /// The function used to attempt to parse a string into a node
    parse: Box<dyn Fn(&str) -> Vec<ParseItem>>,

    /// The list of detected nodes and their location
    detections: Vec<DetectItem>,
}

impl Detector {
    fn new(provider: String, parse: Box<dyn Fn(&str) -> Vec<ParseItem>>) -> Self {
        Self {
            provider,
            parse,
            detections: Vec::new(),
        }
    }

    fn visit_string(&mut self, address: &Address, string: &str) {
        let nodes = (self.parse)(string);
        let mut detections = nodes
            .into_iter()
            .map(|ParseItem { begin, end, node }| DetectItem {
                provider: self.provider.clone(),
                confidence: 100,
                begin: address.add_index(begin),
                end: address.add_index(end),
                node,
            })
            .collect();
        self.detections.append(&mut detections);
    }
}

impl Visitor for Detector {
    fn visit_node(&mut self, address: &Address, node: &Node) -> bool {
        if let Node::String(string) = node {
            self.visit_string(address, string);
            false
        } else {
            true
        }
    }

    fn visit_inline(&mut self, address: &Address, node: &InlineContent) -> bool {
        if let InlineContent::String(string) = node {
            self.visit_string(address, string);
            false
        } else {
            true
        }
    }
}

/// Schedule import and/or export to/from a remove [`Node`] and a local path
pub async fn run_schedule(
    schedule: &str,
    sender: mpsc::Sender<()>,
    mut canceller: mpsc::Receiver<()>,
) -> Result<()> {
    let (schedules, timezone) = cron_utils::parse(schedule)?;
    tracing::info!(
        "Running cron schedule `{}` in timezone `{}`",
        schedules
            .iter()
            .map(|schedule| schedule.to_string())
            .collect::<Vec<String>>()
            .join("; "),
        timezone
    );

    tokio::spawn(async move {
        let interval = Duration::from_secs(1);
        let mut next = cron_utils::next(&schedules, &timezone);
        if let Some(time) = next {
            tracing::debug!("First action scheduled for {}", time);
        }

        loop {
            if let Err(..) = timeout(interval, canceller.recv()).await {
                match next {
                    Some(time) => {
                        if Utc::now() >= time {
                            if let Err(error) = sender.send(()).await {
                                tracing::error!("When sending schedule message: {}", error);
                            }
                            next = cron_utils::next(&schedules, &timezone);
                            if let Some(time) = next {
                                tracing::debug!("Next action scheduled for {}", time);
                            }
                        }
                    }
                    None => {
                        tracing::info!("No more scheduled actions");
                        break;
                    }
                }
            } else {
                tracing::info!("Schedule was cancelled");
                break;
            }
        }
    })
    .await?;

    Ok(())
}
