use once_cell::sync::Lazy;
use provider::{
    eyre::{bail, eyre, Result},
    stencila_schema::Node,
    ProviderTrait,
};
use std::collections::BTreeMap;
use std::sync::Arc;

pub use provider::{Provider, ProviderDetection};

// The following high level functions hide the implementation
// detail of having a static list of providers. They are intended as the
// only public interface for this crate.

pub async fn detect(node: &Node) -> Result<Vec<ProviderDetection>> {
    PROVIDERS.detect(node).await
}

pub async fn enrich(node: Node) -> Result<Node> {
    PROVIDERS.enrich(node).await
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
    async fn detect(&self, node: &Node) -> Result<Vec<ProviderDetection>> {
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
    async fn enrich(&self, mut node: Node) -> Result<Node> {
        for (name, ..) in &self.inner {
            if let Some(future) = dispatch_builtins!(name, enrich, node.clone()) {
                node = future.await?;
            }
        }
        Ok(node)
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
    }

    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            match self {
                Command::List(action) => action.run().await,
                Command::Show(action) => action.run().await,
                Command::Detect(action) => action.run().await,
                Command::Enrich(action) => action.run().await,
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
                let node = enrich(detection.node).await?;
                nodes.push(node);
            }

            result::value(nodes)
        }
    }
}
