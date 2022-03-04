use once_cell::sync::Lazy;
use provider::{
    codecs,
    eyre::{bail, eyre, Result},
    stencila_schema::Node,
    tracing, EnrichOptions, ImportOptions, ProviderTrait, WatchOptions,
};
use std::sync::Arc;
use std::{collections::BTreeMap, path::Path};

pub use provider::{DetectItem, Provider};

// The following high level functions hide the implementation
// detail of having a static list of providers. They are intended as the
// only public interface for this crate.

pub async fn detect(node: &Node) -> Result<Vec<DetectItem>> {
    PROVIDERS.detect(node).await
}

pub async fn find(node: &Node) -> Result<Node> {
    let detections = detect(node).await?;
    let detection = match detections.len() {
        0 => bail!("No node detected"),
        1 => &detections[0],
        _ => {
            tracing::warn!("More that one node detected; will only use first");
            &detections[0]
        }
    };
    Ok(detection.node.clone())
}

pub async fn enrich(node: Node, options: Option<EnrichOptions>) -> Result<Node> {
    PROVIDERS.enrich(node, options).await
}

pub async fn import(node: &Node, dest: &Path, options: Option<ImportOptions>) -> Result<bool> {
    PROVIDERS.import(node, dest, options).await
}

pub async fn watch(node: &Node, dest: &Path, options: Option<WatchOptions>) -> Result<bool> {
    PROVIDERS.watch(node, dest, options).await
}

/// The set of registered providers in the current process
static PROVIDERS: Lazy<Arc<Providers>> = Lazy::new(|| Arc::new(Providers::new()));

/// A set of registered providers, either built-in, or provided by plugins
struct Providers {
    inner: BTreeMap<String, Provider>,
}

/// A macro to dispatch methods to builtin providers
macro_rules! dispatch_builtins {
    ($var:expr, $method:ident $(,$arg:expr)*) => {
        match $var.as_str() {
            #[cfg(feature = "provider-doi")]
            "doi" => Some(provider_doi::DoiProvider::$method($($arg),*)),
            #[cfg(feature = "provider-elife")]
            "elife" => Some(provider_elife::ElifeProvider::$method($($arg),*)),
            #[cfg(feature = "provider-github")]
            "github" => Some(provider_github::GithubProvider::$method($($arg),*)),
            _ => None
        }
    };
}

impl Providers {
    /// Create a set of providers
    pub fn new() -> Self {
        let inner = vec![
            #[cfg(feature = "provider-doi")]
            ("doi", provider_doi::DoiProvider::spec()),
            #[cfg(feature = "provider-elife")]
            ("elife", provider_elife::ElifeProvider::spec()),
            #[cfg(feature = "provider-github")]
            ("github", provider_github::GithubProvider::spec()),
        ]
        .into_iter()
        .map(|(label, provider): (&str, Provider)| (label.to_string(), provider))
        .collect();

        Self { inner }
    }

    /// List the available providers
    fn list(&self) -> Vec<String> {
        self.inner
            .keys()
            .into_iter()
            .cloned()
            .collect::<Vec<String>>()
    }

    /// Get a provider by name
    fn get(&self, name: &str) -> Result<Provider> {
        match self.inner.get(&name.to_lowercase()) {
            Some(provider) => Ok(provider.clone()),
            None => bail!("No provider with name `{}`", name),
        }
    }

    /// Detect nodes within a node
    ///
    /// The `detect` method of each registered provider is called on the node and the result
    /// is a list of detections across all providers.
    async fn detect(&self, node: &Node) -> Result<Vec<DetectItem>> {
        let mut detected = Vec::new();
        for (name, ..) in &self.inner {
            if let Some(future) = dispatch_builtins!(name, detect, node) {
                let mut result = future.await?;
                detected.append(&mut result);
            }
        }
        Ok(detected)
    }

    /// Enrich a node
    ///
    /// The `enrich` method of each registered provider is called on the node possibly mutating it with new
    /// and/or different values for fields.
    async fn enrich(&self, mut node: Node, options: Option<EnrichOptions>) -> Result<Node> {
        for (name, ..) in &self.inner {
            if let Some(future) = dispatch_builtins!(name, enrich, node.clone(), options.clone()) {
                node = future.await?;
            }
        }
        Ok(node)
    }

    /// Import a node
    ///
    /// The `import` method of each registered provider is called until the first that returns `true` (indicating that
    /// the node was imported).
    async fn import(
        &self,
        node: &Node,
        dest: &Path,
        options: Option<ImportOptions>,
    ) -> Result<bool> {
        for (name, ..) in &self.inner {
            if let Some(future) = dispatch_builtins!(name, import, node, dest, options.clone()) {
                let imported = future.await?;
                if imported {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// Watch a node
    async fn watch(&self, node: &Node, dest: &Path, options: Option<WatchOptions>) -> Result<bool> {
        for (name, ..) in &self.inner {
            if let Some(future) = dispatch_builtins!(name, watch, node, dest, options.clone()) {
                let watched = future.await?;
                if watched {
                    return Ok(true);
                }
            }
        }
        Ok(false)
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

    use super::*;
    use cli_utils::{async_trait::async_trait, result, Result, Run};
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Manage providers",
        setting = structopt::clap::AppSettings::ColoredHelp,
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::VersionlessSubcommands
    )]
    pub enum Command {
        List(List),
        Show(Show),
        Detect(Detect),
        Enrich(Enrich),
        Import(Import),
        Watch(Watch),
    }

    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            match self {
                Command::List(action) => action.run().await,
                Command::Show(action) => action.run().await,
                Command::Detect(action) => action.run().await,
                Command::Enrich(action) => action.run().await,
                Command::Import(action) => action.run().await,
                Command::Watch(action) => action.run().await,
            }
        }
    }

    /// List the providers that are available
    ///
    /// The list of available providers includes those that are built into the Stencila
    /// binary as well as those provided by plugins.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct List {}
    #[async_trait]
    impl Run for List {
        async fn run(&self) -> Result {
            let list = PROVIDERS.list();
            result::value(list)
        }
    }

    /// Show the specifications of a provider
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
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
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Detect {
        /// The path to the file (or the string value if the `--string` flag is used)
        path: PathBuf,

        /// The format of the file; defaults to the file extension
        format: Option<String>,

        /// If the argument should be treated as a string, rather than a file path
        #[structopt(short, long)]
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
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Enrich {
        /// The path to the file (or the string value if the `--string` flag is used)
        path: PathBuf,

        /// The format of the file; defaults to the file extension
        format: Option<String>,

        /// If the argument should be treated as a string, rather than a file path
        #[structopt(short, long)]
        string: bool,
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
            for detection in detections.into_iter() {
                let node = enrich(detection.node, None).await?;
                nodes.push(node);
            }

            result::value(nodes)
        }
    }

    /// Import files from a node
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Import {
        /// The source identifier e.g. github:org/name@v1.2.0
        source: String,

        /// The destination to import the source to
        #[structopt(default_value = ".")]
        destination: PathBuf,
    }
    #[async_trait]
    impl Run for Import {
        async fn run(&self) -> Result {
            let identifier = Node::String(self.source.clone());
            let node = find(&identifier).await?;

            let imported = import(&node, &self.destination, None).await?;
            if !imported {
                tracing::warn!("Unable to import node: {:?}", node);
            }

            result::nothing()
        }
    }

    #[derive(Debug, StructOpt)]
    #[structopt()]
    pub struct Watch {
        /// The source identifier e.g. github:org/name
        source: String,

        /// The destination to synchronize the source to
        #[structopt(default_value = ".")]
        destination: PathBuf,

        /// An access token required to setup the watch (usually to create a Webhook with provider)
        #[structopt(long, short)]
        token: Option<String>,

        /// The URL to listen on
        #[structopt(long, short)]
        url: Option<String>,
    }
    #[async_trait]
    impl Run for Watch {
        async fn run(&self) -> Result {
            let identifier = Node::String(self.source.clone());
            let node = find(&identifier).await?;

            let options = WatchOptions {
                token: self.token.clone(),
                url: self.url.clone(),
            };
            let watching = watch(&node, &self.destination, Some(options)).await?;
            if !watching {
                tracing::warn!("Unable to watch node: {:?}", node);
            }

            result::nothing()
        }
    }
}
