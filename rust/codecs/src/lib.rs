use codec::{
    eyre::{bail, Result},
    stencila_schema::Node,
    Codec, CodecTrait,
};
use codec_format::FormatCodec;
use formats::{match_name, Format};
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
pub async fn from_path(path: &Path, format: &str, options: Option<DecodeOptions>) -> Result<Node> {
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
pub async fn to_path(
    node: &Node,
    path: &Path,
    format: &str,
    options: Option<EncodeOptions>,
) -> Result<()> {
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
    ($format:expr, $method:ident $(,$arg:expr)*) => {
        match $format {
            #[cfg(feature = "codec-date")]
            Format::Date => Some(codec_date::DateCodec::$method($($arg),*)),
            #[cfg(feature = "codec-docx")]
            Format::Docx => Some(codec_docx::DocxCodec::$method($($arg),*)),
            #[cfg(feature = "codec-html")]
            Format::Html => Some(codec_html::HtmlCodec::$method($($arg),*)),
            #[cfg(feature = "codec-ipynb")]
            Format::Ipynb => Some(codec_ipynb::IpynbCodec::$method($($arg),*)),
            #[cfg(feature = "codec-json")]
            Format::Json => Some(codec_json::JsonCodec::$method($($arg),*)),
            #[cfg(feature = "codec-json5")]
            Format::Json5 => Some(codec_json5::Json5Codec::$method($($arg),*)),
            #[cfg(feature = "codec-latex")]
            Format::LaTeX => Some(codec_latex::LatexCodec::$method($($arg),*)),
            #[cfg(feature = "codec-md")]
            Format::Markdown => Some(codec_md::MdCodec::$method($($arg),*)),
            #[cfg(feature = "codec-pandoc")]
            Format::Pandoc => Some(codec_pandoc::PandocCodec::$method($($arg),*)),
            #[cfg(feature = "codec-pdf")]
            Format::Pdf => Some(codec_pdf::PdfCodec::$method($($arg),*)),
            #[cfg(feature = "codec-person")]
            Format::Person => Some(codec_person::PersonCodec::$method($($arg),*)),
            #[cfg(feature = "codec-png")]
            Format::Png => Some(codec_png::PngCodec::$method($($arg),*)),
            #[cfg(feature = "codec-rmd")]
            Format::RMarkdown => Some(codec_rmd::RmdCodec::$method($($arg),*)),
            #[cfg(feature = "codec-rpng")]
            Format::Rpng => Some(codec_rpng::RpngCodec::$method($($arg),*)),
            #[cfg(feature = "codec-toml")]
            Format::Toml => Some(codec_toml::TomlCodec::$method($($arg),*)),
            #[cfg(feature = "codec-txt")]
            Format::PlainText => Some(codec_txt::TxtCodec::$method($($arg),*)),
            #[cfg(feature = "codec-yaml")]
            Format::Yaml => Some(codec_yaml::YamlCodec::$method($($arg),*)),

            _ => None
        }
    };
}

impl Codecs {
    /// Create a new codec registry
    ///
    /// Note that these strings are labels for the codec which
    /// aim to be consistent with the codec name, are convenient
    /// to use to `stencila codecs show`, and don't need to be
    /// consistent with format names or aliases.
    pub fn new() -> Self {
        let inner = vec![
            #[cfg(feature = "codec-date")]
            ("date", codec_date::DateCodec::spec()),
            #[cfg(feature = "codec-docx")]
            ("docx", codec_docx::DocxCodec::spec()),
            #[cfg(feature = "codec-html")]
            ("html", codec_html::HtmlCodec::spec()),
            #[cfg(feature = "codec-ipynb")]
            ("ipynb", codec_ipynb::IpynbCodec::spec()),
            #[cfg(feature = "codec-json")]
            ("json", codec_json::JsonCodec::spec()),
            #[cfg(feature = "codec-json5")]
            ("json5", codec_json5::Json5Codec::spec()),
            #[cfg(feature = "codec-latex")]
            ("latex", codec_latex::LatexCodec::spec()),
            #[cfg(feature = "codec-md")]
            ("md", codec_md::MdCodec::spec()),
            #[cfg(feature = "codec-pandoc")]
            ("pandoc", codec_pandoc::PandocCodec::spec()),
            #[cfg(feature = "codec-pdf")]
            ("pdf", codec_pdf::PdfCodec::spec()),
            #[cfg(feature = "codec-person")]
            ("person", codec_person::PersonCodec::spec()),
            #[cfg(feature = "codec-png")]
            ("png", codec_png::PngCodec::spec()),
            #[cfg(feature = "codec-rmd")]
            ("rmd", codec_rmd::RmdCodec::spec()),
            #[cfg(feature = "codec-rpng")]
            ("rpng", codec_rpng::RpngCodec::spec()),
            #[cfg(feature = "codec-toml")]
            ("toml", codec_toml::TomlCodec::spec()),
            #[cfg(feature = "codec-txt")]
            ("txt", codec_txt::TxtCodec::spec()),
            #[cfg(feature = "codec-yaml")]
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
        let format = match_name(format);
        let format_spec = format.spec();

        let options = Some(DecodeOptions {
            format: Some(format_spec.extension.clone()),
            ..options.unwrap_or_default()
        });

        if let Some(future) = dispatch_builtins!(format, from_str_async, content, options.clone()) {
            return future.await;
        }

        if let Ok(node) = FormatCodec::from_str_async(content, options).await {
            return Ok(node);
        }

        bail!(
            "Unable to decode from string with format `{}`: no matching codec found",
            format_spec.title
        )
    }

    /// Decode a document node from a file system path
    #[allow(clippy::needless_update)]
    async fn from_path(
        &self,
        path: &Path,
        format: &str,
        options: Option<DecodeOptions>,
    ) -> Result<Node> {
        let format = match_name(format);
        let format_spec = format.spec();

        let options = Some(DecodeOptions {
            format: Some(format_spec.extension.clone()),
            ..options.unwrap_or_default()
        });

        if let Some(future) = dispatch_builtins!(format, from_path, path, options) {
            return future.await;
        }

        bail!(
            "Unable to decode from path with format `{}`: no matching codec found",
            format_spec.title
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
        let format = match_name(format);
        let format_spec = format.spec();

        let options = Some(EncodeOptions {
            format: Some(format_spec.extension.clone()),
            ..options.unwrap_or_default()
        });

        if let Some(future) = dispatch_builtins!(format, to_string_async, node, options) {
            return future.await;
        }

        bail!(
            "Unable to encode to string of format `{}`: no matching codec found",
            format_spec.title
        )
    }

    /// Encode a document node to a file system path
    async fn to_path(
        &self,
        node: &Node,
        path: &Path,
        format: &str,
        options: Option<EncodeOptions>,
    ) -> Result<()> {
        let format = match_name(format);
        let format_spec = format.spec();

        let options = Some(EncodeOptions {
            format: Some(format_spec.extension.clone()),
            ..options.unwrap_or_default()
        });

        if let Some(future) = dispatch_builtins!(format, to_path, node, path, options) {
            return future.await;
        }

        bail!(
            "Unable to encode to path with format `{}`: no matching codec found",
            format_spec.title
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
