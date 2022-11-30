use std::{env, path::Path};

use common::{
    async_trait::async_trait,
    eyre::Result,
    once_cell::sync::Lazy,
    regex::Regex,
    serde::{Deserialize, Serialize},
    serde_json,
    strum::{AsRefStr, EnumString, EnumVariantNames},
    tokio::sync::mpsc,
    tracing,
};
use http_utils::http::{Request, Response, StatusCode};
use node_address::Address;
use node_pointer::{walk, Visitor};
use stencila_schema::{InlineContent, Node};

// Export and re-export for the convenience of crates that implement a provider
pub use ::common;
pub use ::http_utils;
pub use ::stencila_schema;

pub const PULL: &str = "pull";
pub const PUSH: &str = "push";
pub const PULL_PUSH: &str = "pull/push";
pub const ACTIONS: &[&str] = &[PULL, PUSH, PULL_PUSH];

/// A specification for providers
///
/// All providers, including those implemented in plugins, should provide this
/// specification. Rust implementations return a `Provider` instance from the
/// `spec` function of `ProviderTrait`. Plugins provide a JSON or YAML serialization
/// as part of their manifest.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase", crate = "common::serde")]
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

    /// Pull content from a remote [`Node`] (e.g. an `Article` or `SoftwareSourceCode` repository) to a local path
    async fn pull(_node: &Node, _path: &Path, _options: Option<PullOptions>) -> Result<()> {
        Ok(())
    }

    /// Push content from a local path to a remote [`Node`] (e.g. an `Article` or `SoftwareSourceCode` repository)
    async fn push(node: &Node, _path: &Path, _options: Option<PushOptions>) -> Result<Node> {
        tracing::error!("Push not implemented for provider");
        Ok(node.clone())
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

    /// Schedule pull and/or pull to/from a remove [`Node`] and a local path
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
        match env::var(token) {
            Ok(value) => Some(value),
            Err(..) => {
                tracing::debug!("Token string `{}` appears to be an environment variable name but no such variable found", token);
                None
            }
        }
    } else {
        Some(token.to_string())
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct EnrichOptions {
    pub token: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct PullOptions {
    /// The token required to access the resource (or the `ALL_CAPS` name of the
    /// environment variable containing the token)
    pub token: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct PushOptions {
    /// The token required to access the resource (or the `ALL_CAPS` name of the
    /// environment variable containing the token)
    pub token: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct SyncOptions {
    /// The synchronization mode
    pub mode: Option<WatchMode>,

    /// The token required to access the resource (or the `ALL_CAPS` name of the
    /// environment variable containing the token)
    pub token: Option<String>,
}

#[derive(Debug, Clone, AsRefStr, EnumString, EnumVariantNames, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", crate = "common::serde")]
#[strum(crate = "common::strum")]
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
#[serde(crate = "common::serde")]
pub struct ParseItem {
    /// The start position in the string that the node was parsed from
    pub begin: usize,

    /// The end position in the string that the node was parsed from
    pub end: usize,

    /// The parsed [`Node`] usually with some properties populated
    pub node: Node,
}

#[derive(Debug, Serialize)]
#[serde(crate = "common::serde")]
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

type DetectorParse = dyn Fn(&str) -> Vec<ParseItem>;

pub struct Detector {
    /// The name of the provider that this detector is for
    provider: String,

    /// The function used to attempt to parse a string into a node
    parse: Box<DetectorParse>,

    /// The list of detected nodes and their location
    detections: Vec<DetectItem>,
}

impl Detector {
    fn new(provider: String, parse: Box<DetectorParse>) -> Self {
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
