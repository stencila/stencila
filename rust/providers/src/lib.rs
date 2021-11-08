use once_cell::sync::Lazy;
use provider::{
    eyre::{bail, Result},
    Provider,
};
use std::collections::BTreeMap;
use std::sync::Arc;

// The following high level functions hide the implementation
// detail of having a static list of providers. They are intended as the
// only public interface for this crate.

/// The set of registered providers in the current process
static PROVIDERS: Lazy<Arc<Providers>> = Lazy::new(|| Arc::new(Providers::new()));

/// A set of registered providers, either built-in, or provided by plugins
struct Providers {
    inner: BTreeMap<String, Provider>,
}

/// A macro to dispatch methods to builtin providers
///
/// This avoids having to do a search over the providers's specs for matching `languages`.
macro_rules! dispatch_builtins {
    ($var:expr, $method:ident $(,$arg:expr)*) => {
        match $var.as_str() {
            #[cfg(feature = "elife")]
            "elife" => Some(provider_elife::ElifeProvider::$method($($arg),*)),
            #[cfg(feature = "github")]
            "github" => Some(provider_github::GithubProvider::$method($($arg),*)),
            _ => None
        }
    };
}

impl Providers {
    /// Create a set of providers
    pub fn new() -> Self {
        let inner = vec![
            #[cfg(feature = "elife")]
            ("elife", provider_elife::ElifeProvider::spec()),
            #[cfg(feature = "github")]
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

    /// Get a provider by label
    fn get(&self, label: &str) -> Result<Provider> {
        match self.inner.get(label) {
            Some(provider) => Ok(provider.clone()),
            None => bail!("No provider with label `{}`", label),
        }
    }
}

impl Default for Providers {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "cli")]
pub mod commands {
    use super::*;
    use cli_utils::{async_trait::async_trait, result, Result, Run};
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Manage providers",
        setting = structopt::clap::AppSettings::ColoredHelp,
        setting = structopt::clap::AppSettings::VersionlessSubcommands
    )]
    pub struct Command {
        #[structopt(subcommand)]
        pub action: Action,
    }

    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::DeriveDisplayOrder
    )]
    pub enum Action {
        List(List),
        Show(Show),
    }

    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            let Self { action } = self;
            match action {
                Action::List(action) => action.run().await,
                Action::Show(action) => action.run().await,
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
        /// The label of the provider
        ///
        /// To get the list of provider labels use `stencila providers list`.
        label: String,
    }
    #[async_trait]
    impl Run for Show {
        async fn run(&self) -> Result {
            let provider = PROVIDERS.get(&self.label)?;
            result::value(provider)
        }
    }
}
