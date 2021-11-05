use formats::FORMATS;
use once_cell::sync::Lazy;
use parser::{
    eyre::{bail, Result},
    Parser, ParserTrait,
};
use std::sync::Arc;

// Re-exports for convenience elsewhere
pub use parser::ParseOptions;

// The following high level functions hide the implementation
// detail of having a static list of parsers. They are intended as the
// only public interface for this crate.

/// The set of registered parsers in the current process
static PARSERS: Lazy<Arc<Parsers>> = Lazy::new(|| Arc::new(Parsers::new()));

/// A set of registered parsers, either built-in, or provided by plugins
struct Parsers {
    inner: Vec<Parser>,
}

/// A macro to dispatch methods to builtin parsers
///
/// This avoids having to do a search over the parsers's specs for matching `languages`.
macro_rules! dispatch_builtins {
    ($var:expr, $method:ident $(,$arg:expr)*) => {
        match $var.as_str() {
            #[cfg(feature = "calc")]
            "calc" => Some(parser_calc::CalcParser::$method($($arg),*)),
            #[cfg(feature = "js")]
            "js" => Some(parser_js::JsParser::$method($($arg),*)),
            #[cfg(feature = "py")]
            "py" => Some(parser_py::PyParser::$method($($arg),*)),
            #[cfg(feature = "r")]
            "r" => Some(parser_r::RParser::$method($($arg),*)),
            #[cfg(feature = "ts")]
            "ts" => Some(parser_ts::TsParser::$method($($arg),*)),
            _ => None
        }
    };
}

impl Parsers {
    pub fn new() -> Self {
        let inner = vec![
            #[cfg(feature = "calc")]
            parser_calc::CalcParser::spec(),
            #[cfg(feature = "js")]
            parser_js::JsParser::spec(),
            #[cfg(feature = "py")]
            parser_py::PyParser::spec(),
            #[cfg(feature = "r")]
            parser_r::RParser::spec(),
            #[cfg(feature = "ts")]
            parser_ts::TsParser::spec(),
        ];

        Self { inner }
    }

    fn list(&self) -> Vec<String> {
        self.inner
            .iter()
            .map(|spec| spec.language.clone())
            .collect()
    }

    fn get(&self, label: &str) -> Result<Parser> {
        let index = label.parse::<usize>()?;
        match self.inner.get(index) {
            Some(parser) => Ok(parser.clone()),
            None => bail!("No parser with label `{}`", label),
        }
    }
}

impl Default for Parsers {
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
        about = "Manage parsers",
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

    /// List the parsers that are available
    ///
    /// The list of available parsers includes those that are built into the Stencila
    /// binary as well as any parsers provided by plugins.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct List {}
    #[async_trait]
    impl Run for List {
        async fn run(&self) -> Result {
            let list = PARSERS.list();
            result::value(list)
        }
    }

    /// Show the specifications of a parser
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Show {
        /// The label of the parser
        ///
        /// To get the list of parser labels use `stencila parsers list`.
        label: String,
    }
    #[async_trait]
    impl Run for Show {
        async fn run(&self) -> Result {
            let parser = PARSERS.get(&self.label)?;
            result::value(parser)
        }
    }
}
