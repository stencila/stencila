use formats::{match_name, Format};
use once_cell::sync::Lazy;
use parser::{
    eyre::{bail, Result},
    graph_triples::{Pairs, Relation, ResourceInfo},
    ParserTrait,
};
use std::sync::Arc;
use std::{collections::BTreeMap, path::Path};

// Re-exports
pub use parser::Parser;

// The following high level functions hide the implementation
// detail of having a static list of parsers. They are intended as the
// only public interface for this crate.

pub fn parse<P: AsRef<Path>>(path: P, code: &str, language: &str) -> Result<ResourceInfo> {
    PARSERS.parse(path, code, language)
}

/// The set of registered parsers in the current process
static PARSERS: Lazy<Arc<Parsers>> = Lazy::new(|| Arc::new(Parsers::new()));

/// A set of registered parsers, either built-in, or provided by plugins
struct Parsers {
    inner: BTreeMap<String, Parser>,
}

/// A macro to dispatch methods to builtin parsers
///
/// This avoids having to do a search over the parsers's specs for matching `languages`.
macro_rules! dispatch_builtins {
    ($format:expr, $method:ident $(,$arg:expr)*) => {
        match $format {
            #[cfg(feature = "parser-bash")]
            Format::Bash | Format::Shell | Format::Zsh => Some(parser_bash::BashParser::$method($($arg),*)),
            #[cfg(feature = "parser-calc")]
            Format::Calc => Some(parser_calc::CalcParser::$method($($arg),*)),
            #[cfg(feature = "parser-js")]
            Format::JavaScript => Some(parser_js::JsParser::$method($($arg),*)),
            #[cfg(feature = "parser-py")]
            Format::Python => Some(parser_py::PyParser::$method($($arg),*)),
            #[cfg(feature = "parser-r")]
            Format::R => Some(parser_r::RParser::$method($($arg),*)),
            #[cfg(feature = "parser-rust")]
            Format::Rust => Some(parser_rust::RustParser::$method($($arg),*)),
            #[cfg(feature = "parser-ts")]
            Format::TypeScript => Some(parser_ts::TsParser::$method($($arg),*)),
            // Fallback to empty result
            _ => Option::<Result<ResourceInfo>>::None
        }
    };
}

impl Parsers {
    /// Create a set of parsers
    ///
    /// Note that these strings are labels for the parser which
    /// aim to be consistent with the parser name, are convenient
    /// to use to `stencila parsers show`, and don't need to be
    /// consistent with format names or aliases.
    pub fn new() -> Self {
        let inner = vec![
            #[cfg(feature = "parser-bash")]
            ("bash", parser_bash::BashParser::spec()),
            #[cfg(feature = "parser-calc")]
            ("calc", parser_calc::CalcParser::spec()),
            #[cfg(feature = "parser-js")]
            ("js", parser_js::JsParser::spec()),
            #[cfg(feature = "parser-py")]
            ("py", parser_py::PyParser::spec()),
            #[cfg(feature = "parser-r")]
            ("r", parser_r::RParser::spec()),
            #[cfg(feature = "parser-rust")]
            ("rust", parser_rust::RustParser::spec()),
            #[cfg(feature = "parser-ts")]
            ("ts", parser_ts::TsParser::spec()),
        ]
        .into_iter()
        .map(|(label, parser): (&str, Parser)| (label.to_string(), parser))
        .collect();

        Self { inner }
    }

    /// List the available parsers
    fn list(&self) -> Vec<String> {
        self.inner
            .keys()
            .into_iter()
            .cloned()
            .collect::<Vec<String>>()
    }

    /// Get a parser by label
    fn get(&self, label: &str) -> Result<Parser> {
        match self.inner.get(label) {
            Some(parser) => Ok(parser.clone()),
            None => bail!("No parser with label `{}`", label),
        }
    }

    /// Parse code in a particular language
    fn parse<P: AsRef<Path>>(&self, path: P, code: &str, language: &str) -> Result<ResourceInfo> {
        let path = path.as_ref();
        let format = match_name(language);

        let resource_info = if let Some(result) = dispatch_builtins!(format, parse, path, code) {
            result?
        } else {
            bail!(
                "Unable to parse code in language `{}`: no matching parser found",
                language
            )
        };

        // Normalize pairs by removing any `Uses` of locally assigned variables
        let mut normalized: Pairs = Vec::with_capacity(resource_info.relations.len());
        for (relation, object) in resource_info.relations {
            let mut include = true;
            if matches!(relation, Relation::Use(..)) {
                for (other_relation, other_object) in &normalized {
                    if matches!(other_relation, Relation::Assign(..)) && *other_object == object {
                        include = false;
                        break;
                    }
                }
            }
            if !include {
                continue;
            }

            normalized.push((relation, object))
        }

        Ok(ResourceInfo {
            relations: normalized,
            ..resource_info
        })
    }
}

impl Default for Parsers {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "cli")]
pub mod commands {
    use std::{fs, path::PathBuf};

    use super::*;
    use cli_utils::{async_trait::async_trait, result, Result, Run};
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Manage and use parsers",
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
        Parse(Parse),
    }

    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            let Self { action } = self;
            match action {
                Action::List(action) => action.run().await,
                Action::Show(action) => action.run().await,
                Action::Parse(action) => action.run().await,
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

    /// Parse some code using a parser
    ///
    /// The code is parsed into a set of graph `Relation`/`Resource` pairs using the
    /// parser that matches the filename extension (or specified using `--lang`).
    /// Useful for testing Stencila's static code analysis for a particular language.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Parse {
        /// The file (or code) to parse
        #[structopt(multiple = true)]
        code: Vec<String>,

        /// If the argument should be treated as text, rather than a file path
        #[structopt(short, long)]
        text: bool,

        /// The language of the code
        #[structopt(short, long)]
        lang: Option<String>,
    }
    #[async_trait]
    impl Run for Parse {
        async fn run(&self) -> Result {
            let (file, code, lang) = if self.text || self.code.len() > 1 {
                let code = self.code.join(" ");
                (
                    "<text>".to_string(),
                    code,
                    self.lang.clone().unwrap_or_default(),
                )
            } else {
                let file = self.code[0].clone();
                let code = fs::read_to_string(&file)?;
                let ext = PathBuf::from(&file)
                    .extension()
                    .map(|ext| ext.to_string_lossy().to_string())
                    .unwrap_or_default();
                let lang = self.lang.clone().or(Some(ext)).unwrap_or_default();
                (file, code, lang)
            };

            let pairs = PARSERS.parse(file, &code, &lang)?;
            result::value(pairs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parser::graph_triples::{relations, resources};
    use std::path::PathBuf;
    use test_utils::assert_json_eq;

    #[test]
    #[cfg(feature = "parser-calc")]
    fn test_parse() -> Result<()> {
        let path = PathBuf::from("<test>");
        let resource_info = parse(&path, "a = 1\nb = a * a", "calc")?;
        assert!(matches!(resource_info.pure, None));
        assert!(!resource_info.is_pure());
        assert_json_eq!(
            resource_info.relations,
            vec![
                (
                    relations::assigns((0, 0, 0, 1)),
                    resources::symbol(&path, "a", "Number")
                ),
                (
                    relations::assigns((1, 0, 1, 1)),
                    resources::symbol(&path, "b", "Number")
                )
            ]
        );
        Ok(())
    }
}
