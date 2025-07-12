use std::path::{Path, PathBuf};

use common::clap::{self, Args};
use format::Format;
use node_strip::StripScope;

/// Command line arguments for stripping nodes
///
/// It is necessary to have this as a separate `struct` (rather than adding
/// these fields to both `DecodeOptions` and `EncodeOptions`) to avoid duplication
/// when DecodeOptions` and `EncodeOptions` are both flattened into `Sync` and `Convert`
/// commands.
#[derive(Debug, Default, Clone, Args)]
pub struct StripOptions {
    /// Scopes defining which properties of nodes should be stripped
    #[arg(long, help_heading = "Stripping Options")]
    strip_scopes: Vec<StripScope>,

    /// A list of node types to strip
    #[arg(long, help_heading = "Stripping Options")]
    strip_types: Vec<String>,

    /// A list of node properties to strip
    #[arg(long, help_heading = "Stripping Options")]
    strip_props: Vec<String>,
}

/// Command line arguments for decoding nodes from other formats
#[derive(Debug, Args)]
pub struct DecodeOptions {
    /// The format of the input/s
    ///
    /// If not supplied, and inputting from a file, is inferred from the extension.
    /// See `stencila formats list` for available formats.
    #[arg(long, short, help_heading = "Decoding Options")]
    pub from: Option<String>,

    /// Use fine decoding if available for input format
    ///
    /// Use this flag to decode content to the finest level of granularity
    /// supported by the format. This is the default for most formats.
    #[arg(long, conflicts_with = "coarse", help_heading = "Decoding Options")]
    fine: bool,

    /// Use coarse decoding if available for input format
    ///
    /// Use this flag to decode content to the coarsest level of granularity
    /// supported by the format. Useful for decoding formats that are not fully
    /// supported to avoid loss of structure.
    #[arg(long, conflicts_with = "fine", help_heading = "Decoding Options")]
    coarse: bool,

    /// Reconstitute nodes from a cache
    ///
    /// Only useful when reconstituting a document from a file previously
    /// encoded with the `--reproducible` option and where a JSON cache of the document
    /// was encoded at the same times.
    ///
    /// Only supported for some formats (.e.g DOCX, ODT).
    /// At present, the cache must be the path to a JSON file.
    #[arg(long, help_heading = "Decoding Options")]
    cache: Option<PathBuf>,

    /// Action when there are losses decoding from input files
    ///
    /// Possible values are "ignore", "trace", "debug", "info", "warn", "error", or "abort", or
    /// a filename to write the losses to (only `json` or `yaml` file extensions are supported).
    #[arg(long, default_value_t = codecs::LossesResponse::Debug, help_heading = "Decoding Options")]
    input_losses: codecs::LossesResponse,
}

impl DecodeOptions {
    /// Build a set of [`codecs::DecodeOptions`] from command line arguments
    pub(crate) fn build(
        &self,
        input: Option<&Path>,
        strip_options: StripOptions,
    ) -> codecs::DecodeOptions {
        let format = self.from.as_ref().map_or_else(
            || input.map(Format::from_path),
            |name| Some(Format::from_name(name)),
        );

        let coarse = self.coarse.then_some(true).or(self.fine.then_some(false));

        codecs::DecodeOptions {
            format,
            coarse,
            cache: self.cache.clone(),
            strip_scopes: strip_options.strip_scopes,
            strip_types: strip_options.strip_types,
            strip_props: strip_options.strip_props,
            losses: self.input_losses.clone(),
            ..Default::default()
        }
    }
}

/// Command line arguments for encoding nodes to other formats
#[derive(Debug, Args)]
pub struct EncodeOptions {
    /// The format of the output/s
    ///
    /// If not supplied, and outputting to a file, is inferred from the extension.
    /// See `stencila formats list` for available formats.
    #[arg(long, short, help_heading = "Encoding Options")]
    pub to: Option<String>,

    /// The template document to use
    ///
    /// Only supported by some formats (e.g. DOCX).
    #[arg(long, help_heading = "Encoding Options")]
    template: Option<PathBuf>,

    /// Encode executable nodes so that they are reproducible
    ///
    /// Encode links to the source of executable nodes so that edits made
    /// to rendered documents can be merged back to the source document.
    ///
    /// Only supported by some formats, and may be the default for those.
    #[arg(long, alias = "repro", help_heading = "Encoding Options")]
    reproducible: bool,

    /// Highlight the rendered outputs of executable nodes
    ///
    /// Only supported by some formats (e.g. DOCX and ODT).
    /// Defaults to `true` when `--reproducible` flag is used.
    #[arg(long, help_heading = "Encoding Options")]
    highlight: bool,

    /// Do not highlight the rendered outputs of executable nodes.
    #[arg(
        long,
        alias = "not-highlight",
        alias = "dont-highlight",
        conflicts_with = "highlight",
        help_heading = "Encoding Options"
    )]
    no_highlight: bool,

    /// Encode as a standalone document
    #[arg(
        long,
        conflicts_with = "not_standalone",
        help_heading = "Encoding Options"
    )]
    standalone: bool,

    /// Do not encode as a standalone document when writing to file
    #[arg(long, conflicts_with = "standalone", help_heading = "Encoding Options")]
    not_standalone: bool,

    /// Recursively encode the content of `IncludeBlock`s to their source file
    ///
    /// Only supported when encoding to a path.
    #[arg(long, help_heading = "Encoding Options")]
    recursive: bool,

    /// Use a compact form of encoding if available
    ///
    /// Use this flag to produce a compact form of encoding if the format supports it.
    /// For formats such as JSON and HTML, this usually means no indentation.
    /// For Markdown-based formats, this means that embedded Base64 media will NOT
    /// be written to separate files in a media folder (the default behavior).
    #[arg(long, conflicts_with = "pretty", help_heading = "Encoding Options")]
    compact: bool,

    /// Use a "pretty" form of encoding if available
    ///
    /// Use this flag to produce pretty forms of encoding (e.g. indentation)
    /// which are supported by some formats (e.g. JSON, HTML).
    #[arg(long, conflicts_with = "compact", help_heading = "Encoding Options")]
    pretty: bool,

    /// Action when there are losses encoding to output files
    ///
    /// See help for `--input-losses` for details.
    #[arg(long, default_value_t = codecs::LossesResponse::Debug, help_heading = "Encoding Options")]
    output_losses: codecs::LossesResponse,
}

impl EncodeOptions {
    /// Build a set of [`codecs::EncodeOptions`] from command line arguments
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build(
        &self,
        input: Option<&Path>,
        output: Option<&Path>,
        default_format: Format,
        strip_options: StripOptions,
    ) -> codecs::EncodeOptions {
        let format = self
            .to
            .as_ref()
            .map_or_else(
                || output.map(Format::from_path),
                |name| Some(Format::from_name(name)),
            )
            .or(Some(default_format));

        let compact = self
            .compact
            .then_some(true)
            .or(self.pretty.then_some(false));

        let reproducible = self.reproducible.then_some(true);

        let highlight = self
            .highlight
            .then_some(true)
            .or(self.no_highlight.then_some(false));

        let template = self.template.clone();

        let standalone = self
            .standalone
            .then_some(true)
            .or(self.not_standalone.then_some(false));

        let recurse = self.recursive.then_some(true);

        let from_path = input.map(PathBuf::from);

        codecs::EncodeOptions {
            format,
            compact,
            highlight,
            reproducible,
            template,
            standalone,
            recurse,
            from_path,
            strip_scopes: strip_options.strip_scopes,
            strip_types: strip_options.strip_types,
            strip_props: strip_options.strip_props,
            ..Default::default()
        }
    }
}
