use codec::{
    eyre::{bail, Result},
    stencila_schema::Node,
    Codec, CodecTrait,
};
use codec_format::FormatCodec;
use formats::FORMATS;
use once_cell::sync::Lazy;
use std::{collections::BTreeMap, path::Path, sync::Arc};

// Re-exports for use in other crates that call the following functions
pub use codec::{DecodeOptions, EncodeOptions};

// The following high level functions hide the implementation
// detail of having a static list of codecs. They are intended as the
// only public interface for this crate.

/// Decode a document node from a string
pub async fn from_str(content: &str, format: &str, options: Option<DecodeOptions>) -> Result<Node> {
    CODECS.from_str(content, format, options).await
}

/// Decode a document node from a file system path
pub async fn from_path<T: AsRef<Path>>(
    path: &T,
    format: &str,
    options: Option<DecodeOptions>,
) -> Result<Node>
where
    T: Send + Sync,
{
    CODECS.from_path(path, format, options).await
}

/// Encode a document node to a string
pub async fn to_string(
    node: &Node,
    format: &str,
    options: Option<EncodeOptions>,
) -> Result<String> {
    CODECS.to_string(node, format, options).await
}

/// Encode a document node to a file system path
pub async fn to_path<T: AsRef<Path>>(
    node: &Node,
    path: &T,
    format: &str,
    options: Option<EncodeOptions>,
) -> Result<()>
where
    T: Send + Sync,
{
    CODECS.to_path(node, path, format, options).await
}

/// The set of registered codecs in the current process
static CODECS: Lazy<Arc<Codecs>> = Lazy::new(|| Arc::new(Codecs::new()));

/// A set of registered codecs, either built-in, or provided by plugins
struct Codecs {
    inner: BTreeMap<String, Codec>,
}

/// A macro to dispatch methods to builtin codecs
///
/// This avoids having to do a search over the codecs's specs for matching
/// `formats`.
macro_rules! dispatch_builtins {
    ($var:expr, $method:ident $(,$arg:expr)*) => {
        match $var.as_str() {
            #[cfg(feature = "date")]
            "date" => Some(codec_date::DateCodec::$method($($arg),*)),
            #[cfg(feature = "docx")]
            "docx" => Some(codec_docx::DocxCodec::$method($($arg),*)),
            #[cfg(feature = "html")]
            "html" => Some(codec_html::HtmlCodec::$method($($arg),*)),
            #[cfg(feature = "ipynb")]
            "ipynb" => Some(codec_ipynb::IpynbCodec::$method($($arg),*)),
            #[cfg(feature = "json")]
            "json" => Some(codec_json::JsonCodec::$method($($arg),*)),
            #[cfg(feature = "json5")]
            "json5" => Some(codec_json5::Json5Codec::$method($($arg),*)),
            #[cfg(feature = "latex")]
            "latex" => Some(codec_latex::LatexCodec::$method($($arg),*)),
            #[cfg(feature = "pandoc")]
            "pandoc" => Some(codec_pandoc::PandocCodec::$method($($arg),*)),
            #[cfg(feature = "person")]
            "person" => Some(codec_person::PersonCodec::$method($($arg),*)),
            #[cfg(feature = "md")]
            "md" => Some(codec_md::MdCodec::$method($($arg),*)),
            #[cfg(feature = "rmd")]
            "rmd" => Some(codec_rmd::RmdCodec::$method($($arg),*)),
            #[cfg(feature = "rpng")]
            "rpng" => Some(codec_rpng::RpngCodec::$method($($arg),*)),
            #[cfg(feature = "toml")]
            "toml" => Some(codec_toml::TomlCodec::$method($($arg),*)),
            #[cfg(feature = "txt")]
            "txt" => Some(codec_txt::TxtCodec::$method($($arg),*)),
            #[cfg(feature = "yaml")]
            "yaml" => Some(codec_yaml::YamlCodec::$method($($arg),*)),

            _ => None
        }
    };
}

impl Codecs {
    /// Create a new codec registry
    pub fn new() -> Self {
        let inner = vec![
            #[cfg(feature = "date")]
            ("date", codec_date::DateCodec::spec()),
            #[cfg(feature = "docx")]
            ("docx", codec_docx::DocxCodec::spec()),
            #[cfg(feature = "html")]
            ("html", codec_html::HtmlCodec::spec()),
            #[cfg(feature = "ipynb")]
            ("ipynb", codec_ipynb::IpynbCodec::spec()),
            #[cfg(feature = "json")]
            ("json", codec_json::JsonCodec::spec()),
            #[cfg(feature = "json5")]
            ("json5", codec_json5::Json5Codec::spec()),
            #[cfg(feature = "latex")]
            ("latex", codec_latex::LatexCodec::spec()),
            #[cfg(feature = "pandoc")]
            ("pandoc", codec_pandoc::PandocCodec::spec()),
            #[cfg(feature = "person")]
            ("person", codec_person::PersonCodec::spec()),
            #[cfg(feature = "md")]
            ("md", codec_md::MdCodec::spec()),
            #[cfg(feature = "rmd")]
            ("rmd", codec_rmd::RmdCodec::spec()),
            #[cfg(feature = "rpng")]
            ("rpng", codec_rpng::RpngCodec::spec()),
            #[cfg(feature = "toml")]
            ("toml", codec_toml::TomlCodec::spec()),
            #[cfg(feature = "txt")]
            ("txt", codec_txt::TxtCodec::spec()),
            #[cfg(feature = "yaml")]
            ("yaml", codec_yaml::YamlCodec::spec()),
        ]
        .into_iter()
        .map(|(label, codec)| (label.to_string(), codec))
        .collect();

        Self { inner }
    }

    /// List all the codecs
    fn list(&self) -> Vec<String> {
        self.inner
            .keys()
            .into_iter()
            .cloned()
            .collect::<Vec<String>>()
    }

    /// Get the codec with the given id
    fn get(&self, label: &str) -> Result<Codec> {
        match self.inner.get(label) {
            Some(codec) => Ok(codec.clone()),
            None => bail!("No codec with label `{}`", label),
        }
    }

    /// Decode a document node from a string
    #[allow(clippy::needless_update)]
    async fn from_str(
        &self,
        content: &str,
        format: &str,
        options: Option<DecodeOptions>,
    ) -> Result<Node> {
        let format = FORMATS.match_name(format);
        let options = Some(DecodeOptions {
            format: Some(format.name.clone()),
            ..options.unwrap_or_default()
        });

        if let Some(future) =
            dispatch_builtins!(format.name, from_str_async, content, options.clone())
        {
            return future.await;
        }

        if let Ok(node) = FormatCodec::from_str_async(content, options).await {
            return Ok(node);
        }

        bail!(
            "Unable to decode node from string with format `{}`: no matching codec found",
            format.name
        )
    }

    /// Decode a document node from a file system path
    #[allow(clippy::needless_update)]
    async fn from_path<T: AsRef<Path>>(
        &self,
        path: &T,
        format: &str,
        options: Option<DecodeOptions>,
    ) -> Result<Node>
    where
        T: Send + Sync,
    {
        let format = FORMATS.match_name(format);
        let options = Some(DecodeOptions {
            format: Some(format.name.clone()),
            ..options.unwrap_or_default()
        });

        if let Some(future) = dispatch_builtins!(format.name, from_path, path, options) {
            return future.await;
        }

        bail!(
            "Unable to decode node from path with format `{}`: no matching codec found",
            format.name
        )
    }

    /// Encode a document node to a string
    #[allow(unused_variables)]
    async fn to_string(
        &self,
        node: &Node,
        format: &str,
        options: Option<EncodeOptions>,
    ) -> Result<String> {
        let format = FORMATS.match_name(format);
        let options = Some(EncodeOptions {
            format: Some(format.name.clone()),
            ..options.unwrap_or_default()
        });

        if let Some(future) = dispatch_builtins!(format.name, to_string_async, node, options) {
            return future.await;
        }

        bail!(
            "Unable to encode node to string of format `{}`: no matching codec found",
            format.name
        )
    }

    /// Encode a document node to a file system path
    async fn to_path<T: AsRef<Path>>(
        &self,
        node: &Node,
        path: &T,
        format: &str,
        options: Option<EncodeOptions>,
    ) -> Result<()>
    where
        T: Send + Sync,
    {
        let format = FORMATS.match_name(format);
        let options = Some(EncodeOptions {
            format: Some(format.name.clone()),
            ..options.unwrap_or_default()
        });

        if let Some(future) = dispatch_builtins!(format.name, to_path, node, path, options) {
            return future.await;
        }

        bail!(
            "Unable to encode node to path of format `{}`: no matching codec found",
            format.name
        )
    }
}

impl Default for Codecs {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "cli")]
pub mod commands {
    use super::*;
    use cli_utils::{result, Result, Run};
    use codec::async_trait::async_trait;
    use structopt::StructOpt;

    /// Manage codecs
    ///
    /// In Stencila, a "codec" is responsible for converting documents
    /// to ("encoding") and from ("decoding") a format (e.g. Markdown).
    /// This command alls you to list the available codecs and see their
    /// specifications (e.g. which formats they support).
    #[derive(Debug, StructOpt)]
    #[structopt(
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

    /// List the codecs available
    ///
    /// The list of available codecs includes those that are built into the Stencila
    /// binary (e.g. `html`) as well as any codecs provided by plugins.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct List {}

    #[async_trait]
    impl Run for List {
        async fn run(&self) -> Result {
            let list = CODECS.list();
            result::value(list)
        }
    }

    /// Show the specifications of a codec
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Show {
        /// The label of the codec
        ///
        /// To get the list of codec labels use `stencila codecs list`.
        label: String,
    }

    #[async_trait]
    impl Run for Show {
        async fn run(&self) -> Result {
            let codec = CODECS.get(&self.label)?;
            result::value(codec)
        }
    }
}
