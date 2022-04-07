use std::{path::Path, sync::Arc};

use once_cell::sync::Lazy;
use provider::{
    codecs,
    eyre::{bail, eyre, Result},
    stencila_schema::Node,
    strum::VariantNames,
    EnrichOptions, ExportOptions, ImportOptions, ProviderTrait, SyncOptions,
};
use tokio::sync::mpsc;

pub use provider::{DetectItem, Provider};

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

pub async fn import(node: &Node, path: &Path, options: Option<ImportOptions>) -> Result<()> {
    PROVIDERS.import(node, path, options).await
}

pub async fn export(node: &Node, path: &Path, options: Option<ExportOptions>) -> Result<()> {
    PROVIDERS.export(node, path, options).await
}

pub async fn watch(
    node: &Node,
    dest: &Path,
    canceller: mpsc::Receiver<()>,
    options: Option<SyncOptions>,
) -> Result<()> {
    PROVIDERS.sync(node, dest, canceller, options).await
}

pub async fn cron(
    action: &str,
    schedule: &str,
    node: &Node,
    dest: &Path,
    canceller: mpsc::Receiver<()>,
) -> Result<()> {
    PROVIDERS
        .cron(action, schedule, node, dest, canceller)
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

    /// Import content from a remote [`Node`] to a local path
    async fn import(&self, node: &Node, path: &Path, options: Option<ImportOptions>) -> Result<()> {
        let provider = self.provider_for(node)?;
        dispatch_builtins!(provider.name, import, node, path, options.clone()).await
    }

    /// Export content from a local path to a remote [`Node`]
    async fn export(&self, node: &Node, path: &Path, options: Option<ExportOptions>) -> Result<()> {
        let provider = self.provider_for(node)?;
        dispatch_builtins!(provider.name, export, node, path, options.clone()).await
    }

    /// Synchronize changes between a remote [`Node`] and a local path
    async fn sync(
        &self,
        node: &Node,
        path: &Path,
        canceller: mpsc::Receiver<()>,
        options: Option<SyncOptions>,
    ) -> Result<()> {
        let provider = self.provider_for(node)?;
        dispatch_builtins!(provider.name, sync, node, path, canceller, options).await
    }

    /// Schedule import and/or export to/from a remove [`Node`] and a local path
    async fn cron(
        &self,
        action: &str,
        schedule: &str,
        node: &Node,
        path: &Path,
        canceller: mpsc::Receiver<()>,
    ) -> Result<()> {
        let provider = self.provider_for(node)?;
        dispatch_builtins!(provider.name, cron, action, schedule, node, path, canceller).await
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
    use provider::WatchMode;
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
        Export(Export),
        Watch(Watch),
        Cron(Cron),
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
                Command::Export(action) => action.run().await,
                Command::Watch(action) => action.run().await,
                Command::Cron(action) => action.run().await,
            }
        }
    }

    /// List the providers that are available
    ///
    /// The list of available providers includes those that are built into the Stencila
    /// binary as well as those provided by plugins.
    #[derive(Debug, StructOpt)]
    #[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
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
    #[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
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
    #[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
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
    #[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
    pub struct Enrich {
        /// The path to the file (or the string value if the `--string` flag is used)
        path: PathBuf,

        /// The format of the file; defaults to the file extension
        format: Option<String>,

        /// If the argument should be treated as a string, rather than a file path
        #[structopt(short, long)]
        string: bool,

        /// The name of a secret environment variable required to access the resource
        ///
        /// Only necessary if authentication is required for the resource and the name
        /// of the secret is different to the default for the corresponding provider.
        #[structopt(long)]
        secret: Option<String>,
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
                secret_name: self.secret.clone(),
            };
            for detection in detections.into_iter() {
                let node = enrich(detection.node, Some(options.clone())).await?;
                nodes.push(node);
            }

            result::value(nodes)
        }
    }

    /// Import content from a remote source to a local path
    #[derive(Debug, StructOpt)]
    #[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
    pub struct Import {
        /// The source identifier e.g. `github:org/name@v1.2.0`
        source: String,

        /// The local path to import file/s to e.g. `data`
        #[structopt(default_value = ".")]
        path: PathBuf,

        /// The name of a secret environment variable required to access the resource
        ///
        /// Only necessary if authentication is required for the resource and the name
        /// of the secret is different to the default for the corresponding provider.
        #[structopt(long)]
        secret: Option<String>,
    }
    #[async_trait]
    impl Run for Import {
        async fn run(&self) -> Result {
            let (.., node) = resolve(&self.source).await?;

            let options = ImportOptions {
                secret_name: self.secret.clone(),
            };
            import(&node, &self.path, Some(options)).await?;

            result::nothing()
        }
    }

    /// Export content from a local path to a remote source
    #[derive(Debug, StructOpt)]
    #[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
    pub struct Export {
        /// The source identifier e.g. `github:org/name@v1.2.0`
        source: String,

        /// The local path to export file/s from e.g. `report.md`
        #[structopt(default_value = ".")]
        path: PathBuf,

        /// The name of a secret environment variable required to access the resource
        ///
        /// Only necessary if authentication is required for the resource and the name
        /// of the secret is different to the default for the corresponding provider.
        #[structopt(long)]
        secret: Option<String>,
    }
    #[async_trait]
    impl Run for Export {
        async fn run(&self) -> Result {
            let (.., node) = resolve(&self.source).await?;

            let options = ExportOptions {
                secret_name: self.secret.clone(),
            };
            export(&node, &self.path, Some(options)).await?;

            result::nothing()
        }
    }

    /// Watch a resource and synchronize changes between a remote source and a local path
    #[derive(Debug, StructOpt)]
    #[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
    pub struct Watch {
        /// The source identifier e.g. `github:org/name`
        source: String,

        /// The local path to synchronize with the source
        #[structopt(default_value = ".")]
        path: PathBuf,

        /// The synchronization mode
        #[structopt(long, short, possible_values = &WatchMode::VARIANTS)]
        mode: Option<WatchMode>,

        /// The name of a secret environment variable required to access the resource
        ///
        /// Only necessary if authentication is required for the resource and the name
        /// of the secret is different to the default for the corresponding provider.
        #[structopt(long)]
        secret: Option<String>,

        /// The host to listen on for events from the source provider
        ///
        /// This option is usually only used for testing during development
        /// with a tool such as ngrok to forward a public host to localhost.
        /// The value should exclude the protocol e.g. "https://" but may include
        /// a port number.
        #[structopt(long)]
        host: Option<String>,
    }
    #[async_trait]
    impl Run for Watch {
        async fn run(&self) -> Result {
            let (.., node) = resolve(&self.source).await?;

            let (subscriber, canceller) = mpsc::channel(1);
            events::subscribe_to_interrupt(subscriber).await;

            let options = SyncOptions {
                mode: self.mode.clone(),
                secret_name: self.secret.clone(),
                host: self.host.clone(),
            };
            watch(&node, &self.path, canceller, Some(options)).await?;

            result::nothing()
        }
    }

    /// Schedule import and/or export between remote source and a local path
    #[derive(Debug, StructOpt)]
    #[structopt(setting = structopt::clap::AppSettings::ColoredHelp)]
    pub struct Cron {
        /// The action to take at the scheduled time
        #[structopt(possible_values=provider::ACTIONS)]
        action: String,

        /// The schedule on which to perform the action
        schedule: String,

        /// The source identifier e.g. `github:org/name`
        source: String,

        /// The local path to synchronize with the source
        #[structopt(default_value = ".")]
        path: PathBuf,
    }
    #[async_trait]
    impl Run for Cron {
        async fn run(&self) -> Result {
            let (.., node) = resolve(&self.source).await?;

            let (subscriber, canceller) = mpsc::channel(1);
            events::subscribe_to_interrupt(subscriber).await;

            cron(&self.action, &self.schedule, &node, &self.path, canceller).await?;

            result::nothing()
        }
    }
}
