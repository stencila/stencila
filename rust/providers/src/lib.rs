use std::{path::Path, sync::Arc};

use provider::{
    common::{
        eyre::{bail, eyre, Result},
        once_cell::sync::Lazy,
        serde_json,
        tokio::sync::mpsc,
    },
    http_utils::http::{Request, Response},
    stencila_schema::Node,
    EnrichOptions, ProviderTrait,
};

pub use provider::{DetectItem, Provider, PullOptions, PushOptions, SyncOptions, WatchMode};

// Re-exports for consumers of this crate
pub use ::provider;

// The following high level functions hide the implementation
// detail of having a static list of providers. They are intended as the
// only public interface for this crate.

pub async fn detect(node: &Node) -> Result<Vec<DetectItem>> {
    PROVIDERS.detect(node).await
}

pub async fn resolve(identifier: &str) -> Result<(String, Node)> {
    let root = Node::String(identifier.to_string());
    let detections = detect(&root).await?;
    match detections.first() {
        Some(detection) => Ok((detection.provider.clone(), detection.node.clone())),
        None => bail!("Unable to resolve identifier `{}`", identifier),
    }
}

pub async fn enrich(node: Node, options: Option<EnrichOptions>) -> Result<Node> {
    PROVIDERS.enrich(node, options).await
}

pub async fn pull(node: &Node, path: &Path, options: Option<PullOptions>) -> Result<()> {
    PROVIDERS.pull(node, path, options).await
}

pub async fn push(node: &Node, path: &Path, options: Option<PushOptions>) -> Result<Node> {
    PROVIDERS.push(node, path, options).await
}

pub async fn sync(
    node: &Node,
    dest: &Path,
    request: &Request<serde_json::Value>,
    options: Option<SyncOptions>,
) -> Result<Response<String>> {
    PROVIDERS.sync(node, dest, request, options).await
}

pub async fn cron(
    node: &Node,
    dest: &Path,
    action: &str,
    schedule: &str,
    canceller: mpsc::Receiver<()>,
) -> Result<()> {
    PROVIDERS
        .cron(node, dest, action, schedule, canceller)
        .await
}

/// The set of registered providers in the current process
static PROVIDERS: Lazy<Arc<Providers>> = Lazy::new(|| Arc::new(Providers::new()));

/// A set of registered providers, either built-in, or provided by plugins
struct Providers {
    inner: Vec<Provider>,
}

/// A macro to dispatch methods to builtin providers
macro_rules! dispatch_builtins {
    ($var:expr, $method:ident $(,$arg:expr)*) => {
        match $var.as_str() {
            #[cfg(feature = "provider-doi")]
            "doi" => provider_doi::DoiProvider::$method($($arg),*),
            #[cfg(feature = "provider-elife")]
            "elife" => provider_elife::ElifeProvider::$method($($arg),*),
            #[cfg(feature = "provider-gdrive")]
            "gdrive" => provider_gdrive::GoogleDriveProvider::$method($($arg),*),
            #[cfg(feature = "provider-github")]
            "github" => provider_github::GithubProvider::$method($($arg),*),
            #[cfg(feature = "provider-gitlab")]
            "gitlab" => provider_gitlab::GitlabProvider::$method($($arg),*),
            #[cfg(feature = "provider-http")]
            "http" => provider_http::HttpProvider::$method($($arg),*),
            _ => bail!("Unable to dispatch to provider `{}`", $var)
        }
    };
}

impl Providers {
    /// Create a set of providers
    ///
    /// Ordering is important because detection is done in order and often
    /// when there are multiple detections for the same location (e.g. a GitHub
    /// url and a generic HTTP/S url) the first is used.
    pub fn new() -> Self {
        let inner = vec![
            #[cfg(feature = "provider-doi")]
            provider_doi::DoiProvider::spec(),
            #[cfg(feature = "provider-elife")]
            provider_elife::ElifeProvider::spec(),
            #[cfg(feature = "provider-gdrive")]
            provider_gdrive::GoogleDriveProvider::spec(),
            #[cfg(feature = "provider-github")]
            provider_github::GithubProvider::spec(),
            #[cfg(feature = "provider-gitlab")]
            provider_gitlab::GitlabProvider::spec(),
            #[cfg(feature = "provider-http")]
            provider_http::HttpProvider::spec(),
        ];
        Self { inner }
    }

    /// List the available providers
    fn list(&self) -> Vec<String> {
        self.inner
            .iter()
            .map(|provider| provider.name.clone())
            .collect()
    }

    /// Get a provider by name
    fn get(&self, name: &str) -> Result<Provider> {
        for provider in &self.inner {
            if provider.name == name {
                return Ok(provider.clone());
            }
        }
        bail!("No provider with name `{}`", name)
    }

    /// Find the provider which recognizes a node
    fn provider_for(&self, node: &Node) -> Result<Provider> {
        for provider in &self.inner {
            if dispatch_builtins!(provider.name, recognize, node) {
                return Ok(provider.clone());
            }
        }
        bail!("None of the registered providers recognize this node")
    }

    /// Detect nodes within a node
    ///
    /// The `detect` method of each provider is called on the node and the result
    /// is a list of detections across all providers.
    async fn detect(&self, node: &Node) -> Result<Vec<DetectItem>> {
        let mut detected = Vec::new();
        for provider in &self.inner {
            let mut result = dispatch_builtins!(provider.name, detect, node).await?;
            detected.append(&mut result);
        }
        Ok(detected)
    }

    /// Enrich a node
    ///
    /// The `enrich` method of each provider is called on the node possibly mutating it with new
    /// and/or different values for fields.
    async fn enrich(&self, mut node: Node, options: Option<EnrichOptions>) -> Result<Node> {
        for provider in &self.inner {
            node = dispatch_builtins!(provider.name, enrich, node, options.clone()).await?;
        }
        Ok(node)
    }

    /// Pull content from a remote [`Node`] to a local path
    async fn pull(&self, node: &Node, path: &Path, options: Option<PullOptions>) -> Result<()> {
        let provider = self.provider_for(node)?;
        dispatch_builtins!(provider.name, pull, node, path, options.clone()).await
    }

    /// Push content from a local path to a remote [`Node`]
    async fn push(&self, node: &Node, path: &Path, options: Option<PushOptions>) -> Result<Node> {
        let provider = self.provider_for(node)?;
        dispatch_builtins!(provider.name, push, node, path, options.clone()).await
    }

    /// Synchronize changes between a remote [`Node`] and a local path
    async fn sync(
        &self,
        node: &Node,
        path: &Path,
        request: &Request<serde_json::Value>,
        options: Option<SyncOptions>,
    ) -> Result<Response<String>> {
        let resolved = match node {
            Node::String(identifier) => resolve(identifier).await?.1,
            _ => node.to_owned(),
        };
        let provider = self.provider_for(&resolved)?;
        dispatch_builtins!(provider.name, sync, &resolved, path, request, options).await
    }

    /// Schedule pull and/or push to/from a remove [`Node`] and a local path
    async fn cron(
        &self,
        node: &Node,
        path: &Path,
        action: &str,
        schedule: &str,
        canceller: mpsc::Receiver<()>,
    ) -> Result<()> {
        let provider = self.provider_for(node)?;
        dispatch_builtins!(provider.name, cron, node, path, action, schedule, canceller).await
    }
}

impl Default for Providers {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "cli")]
pub mod commands {
    use std::path::PathBuf;

    use cli_utils::{
        clap::{self, Parser},
        common::async_trait::async_trait,
        result, Result, Run,
    };

    use super::*;

    /// Manage and use source providers
    #[derive(Parser)]
    pub struct Command {
        #[clap(subcommand)]
        pub action: Action,
    }

    #[derive(Parser)]
    pub enum Action {
        List(List),
        Show(Show),
        Detect(Detect),
        Enrich(Enrich),
        #[clap(alias = "import")]
        Pull(Pull),
        #[clap(alias = "export")]
        Push(Push),
        Cron(Cron),
    }

    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            match &self.action {
                Action::List(action) => action.run().await,
                Action::Show(action) => action.run().await,
                Action::Detect(action) => action.run().await,
                Action::Enrich(action) => action.run().await,
                Action::Pull(action) => action.run().await,
                Action::Push(action) => action.run().await,
                Action::Cron(action) => action.run().await,
            }
        }
    }

    /// List the providers that are available
    ///
    /// The list of available providers includes those that are built into the Stencila
    /// binary as well as those provided by plugins.
    #[derive(Parser)]
    pub struct List {}

    #[async_trait]
    impl Run for List {
        async fn run(&self) -> Result {
            let list = PROVIDERS.list();
            result::value(list)
        }
    }

    /// Show the specifications of a provider
    #[derive(Parser)]
    pub struct Show {
        /// The name of the provider
        ///
        /// To get the list of provider names using `stencila providers list`.
        name: String,
    }

    #[async_trait]
    impl Run for Show {
        async fn run(&self) -> Result {
            let provider = PROVIDERS.get(&self.name)?;
            result::value(provider)
        }
    }

    /// Detect nodes within a file or string
    #[derive(Parser)]
    pub struct Detect {
        /// The path to the file (or the string value if the `--string` flag is used)
        path: PathBuf,

        /// The format of the file; defaults to the file extension
        format: Option<String>,

        /// If the argument should be treated as a string, rather than a file path
        #[clap(short, long)]
        string: bool,
    }

    #[async_trait]
    impl Run for Detect {
        async fn run(&self) -> Result {
            let node = if self.string {
                let string = self
                    .path
                    .to_str()
                    .ok_or_else(|| eyre!("Value is not valid Unicode"))?
                    .into();
                Node::String(string)
            } else {
                codecs::from_path(&self.path, self.format.as_deref(), None).await?
            };

            let detections = detect(&node).await?;
            result::value(detections)
        }
    }

    /// Enrich nodes within a file or string
    #[derive(Parser)]
    pub struct Enrich {
        /// The path to the file (or the string value if the `--string` flag is used)
        path: PathBuf,

        /// The format of the file; defaults to the file extension
        format: Option<String>,

        /// If the argument should be treated as a string, rather than a file path
        #[clap(short, long)]
        string: bool,

        /// The token (or name of environment variable) required to access the resource
        ///
        /// Only necessary if authentication is required for the resource. Defaults to
        /// using the environment variable corresponding to the provider of the resource
        /// e.g. `GITHUB_TOKEN`.
        #[clap(long)]
        token: Option<String>,
    }

    #[async_trait]
    impl Run for Enrich {
        async fn run(&self) -> Result {
            let node = if self.string {
                let string = self
                    .path
                    .to_str()
                    .ok_or_else(|| eyre!("Value is not valid Unicode"))?
                    .into();
                Node::String(string)
            } else {
                codecs::from_path(&self.path, self.format.as_deref(), None).await?
            };

            let detections = detect(&node).await?;

            let mut nodes: Vec<Node> = Vec::with_capacity(detections.len());
            let options = EnrichOptions {
                token: self.token.clone(),
            };
            for detection in detections.into_iter() {
                let node = enrich(detection.node, Some(options.clone())).await?;
                nodes.push(node);
            }

            result::value(nodes)
        }
    }

    /// Pull files or content from a remote source to a local path
    #[derive(Parser)]
    pub struct Pull {
        /// The source identifier e.g. `github:org/name@v1.2.0`
        source: String,

        /// The local path to import file/s to e.g. `data`
        #[clap(default_value = ".")]
        path: PathBuf,

        /// The token (or name of environment variable) required to access the resource
        ///
        /// Only necessary if authentication is required for the resource. Defaults to
        /// using the environment variable corresponding to the provider of the resource
        /// e.g. `GITHUB_TOKEN`.
        #[clap(long)]
        token: Option<String>,
    }

    #[async_trait]
    impl Run for Pull {
        async fn run(&self) -> Result {
            let (.., node) = resolve(&self.source).await?;

            let options = PullOptions {
                token: self.token.clone(),
            };
            pull(&node, &self.path, Some(options)).await?;

            result::nothing()
        }
    }

    /// Push files or content from a local path to a remote source
    #[derive(Parser)]
    pub struct Push {
        /// The source identifier e.g. `github:org/name@v1.2.0`
        source: String,

        /// The local path to export file/s from e.g. `report.md`
        #[clap(default_value = ".")]
        path: PathBuf,

        /// The token (or name of environment variable) required to access the resource
        ///
        /// Only necessary if authentication is required for the resource. Defaults to
        /// using the environment variable corresponding to the provider of the resource
        /// e.g. `GITHUB_TOKEN`.
        #[clap(long)]
        token: Option<String>,
    }

    #[async_trait]
    impl Run for Push {
        async fn run(&self) -> Result {
            let (.., node) = resolve(&self.source).await?;

            let options = PushOptions {
                token: self.token.clone(),
            };
            let node = push(&node, &self.path, Some(options)).await?;

            result::value(node)
        }
    }

    /// Schedule pull and/or push between remote source and a local path
    #[derive(Parser)]
    pub struct Cron {
        /// The action to take at the scheduled time
        #[clap(possible_values=provider::ACTIONS)]
        action: String,

        /// The schedule on which to perform the action
        schedule: String,

        /// The source identifier e.g. `github:org/name`
        source: String,

        /// The local path to synchronize with the source
        #[clap(default_value = ".")]
        path: PathBuf,
    }

    #[async_trait]
    impl Run for Cron {
        async fn run(&self) -> Result {
            let (.., node) = resolve(&self.source).await?;

            let (subscriber, canceller) = mpsc::channel(1);
            events::subscribe_to_interrupt(subscriber).await;

            cron(&node, &self.path, &self.action, &self.schedule, canceller).await?;

            result::nothing()
        }
    }
}
