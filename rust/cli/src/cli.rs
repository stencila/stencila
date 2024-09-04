use std::path::{Path, PathBuf};

use app::DirType;
use cli_utils::{Code, ToStdout};
use codecs::LossesResponse;
use common::{
    clap::{self, Args, Parser, Subcommand},
    eyre::Result,
    tracing,
};
use document::Document;
use format::Format;
use node_execute::ExecuteOptions;
use node_strip::StripScope;
use server::{serve, ServeOptions};

use crate::{
    logging::{LoggingFormat, LoggingLevel},
    preview, sync, uninstall, upgrade,
};

/// CLI subcommands and global options
#[derive(Debug, Parser)]
#[command(name = "stencila", author, version, about, long_about, styles = Cli::styles())]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Display debug level logging and detailed error reports
    ///
    /// Equivalent to using `--log-level=debug`, `--log-format=pretty`, and `--error-details=all`
    #[arg(
        long,
        global = true,
        conflicts_with = "trace",
        conflicts_with = "log_level",
        conflicts_with = "log_format",
        conflicts_with = "error_details"
    )]
    pub debug: bool,

    /// Display trace level logging and detailed error reports
    ///
    /// Equivalent to using `--log-level=trace`, `--log-format=pretty`, and `--error-details=all`
    #[arg(
        long,
        global = true,
        conflicts_with = "debug",
        conflicts_with = "log_level",
        conflicts_with = "log_format",
        conflicts_with = "error_details"
    )]
    pub trace: bool,

    /// The minimum log level to output
    #[arg(long, default_value = "info", global = true)]
    pub log_level: LoggingLevel,

    /// A filter for log entries
    ///
    /// Allows more fine-grained control over which log entries are shown.
    /// To additionally see lower level entries for a specific crates use
    /// syntax such as `tokio=debug`.
    #[arg(
        long,
        default_value = "globset=warn,hyper=info,hyper_util=info,ignore=warn,mio=info,notify=warn,ort=error,reqwest=info,sled=info,tokio=info,tungstenite=info",
        global = true
    )]
    pub log_filter: String,

    /// The log format to use
    ///
    /// When `auto`, uses `simple` for terminals and `json`
    /// for non-TTY devices.
    #[arg(long, default_value = "auto", global = true)]
    pub log_format: LoggingFormat,

    /// The details to include in error reports
    ///
    /// `auto`, `all`, or a comma separated list including `location`, `span`, or `env`.
    #[arg(long, default_value = "auto", global = true)]
    pub error_details: String,

    /// Output a link to more easily report an issue
    #[arg(long, global = true)]
    pub error_link: bool,
}

impl Cli {
    pub fn styles() -> clap::builder::Styles {
        use clap::builder::styling::*;
        Styles::styled()
            .header(AnsiColor::Blue.on_default().bold())
            .usage(AnsiColor::Cyan.on_default())
            .literal(AnsiColor::Cyan.on_default())
            .valid(AnsiColor::Green.on_default())
            .invalid(AnsiColor::Yellow.on_default())
            .error(AnsiColor::Red.on_default().bold())
            .placeholder(AnsiColor::Green.on_default())
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create a new document
    New {
        /// The path of the document to create
        path: PathBuf,

        /// The source file to import from
        #[arg(long, short)]
        source: Option<PathBuf>,

        /// The format of the source file
        #[arg(long, short)]
        format: Option<Format>,

        /// The codec to use to decode the source
        #[arg(long)]
        codec: Option<String>,

        /// Overwrite the document if it already exists
        #[arg(long, short)]
        overwrite: bool,
    },

    /*
    /// Import a file in another format into a new or existing document
    Import {
        /// The path of the document to create or import to
        doc: PathBuf,

        /// The source file to import from
        source: PathBuf,

        /// The format of the source file
        ///
        /// Defaults to inferring the format from the file name extension
        /// of the source file.
        #[arg(long, short, alias = "format")]
        from: Option<String>,

        /// What to do if there are losses when decoding
        #[arg(long, short, default_value_t = codecs::LossesResponse::Warn)]
        losses: codecs::LossesResponse,

        #[command(flatten)]
        options: DecodeOptions,

        #[command(flatten)]
        strip_options: StripOptions,
    },

    /// Export a document to a file in another format
    Export {
        /// The path of the document to export from
        doc: PathBuf,

        /// The destination file to export to
        dest: Option<PathBuf>,

        /// The format of the destination file
        ///
        /// Defaults to inferring the format from the file name extension
        /// of the destination file.
        #[arg(long, short, alias = "format")]
        to: Option<String>,

        /// What to do if there are losses when encoding
        #[arg(long, short, default_value_t = codecs::LossesResponse::Warn)]
        losses: codecs::LossesResponse,

        #[command(flatten)]
        options: EncodeOptions,

        #[command(flatten)]
        strip_options: StripOptions,
    },
    */
    Sync(sync::Cli),

    /// Convert a document to another format
    Convert {
        /// The path of the input file
        ///
        /// If not supplied the input content is read from `stdin`.
        input: Option<PathBuf>,

        /// The path of the output file
        ///
        /// If not supplied the output content is written to `stdout`.
        output: Option<PathBuf>,

        /// The format to encode from (or codec to use)
        ///
        /// Defaults to inferring the format from the file name extension
        /// of the `input`.
        #[arg(long, short)]
        from: Option<String>,

        /// The format to encode to (or codec to use)
        ///
        /// Defaults to inferring the format from the file name extension
        /// of the `output`. If no `output` is supplied, defaults to JSON.
        #[arg(long, short)]
        to: Option<String>,

        /// What to do if there are losses when decoding from the input
        ///
        /// Possible values are "ignore", "trace", "debug", "info", "warn", "error", or "abort", or
        /// a filename to write the losses to (only `json` or `yaml` file extensions are supported).
        #[arg(long, short, default_value_t = codecs::LossesResponse::Warn)]
        input_losses: codecs::LossesResponse,

        /// What to do if there are losses when encoding to the output
        ///
        /// See help for `--input-losses` for details.
        #[arg(long, short, default_value_t = codecs::LossesResponse::Warn)]
        output_losses: codecs::LossesResponse,

        #[command(flatten)]
        decode_options: DecodeOptions,

        #[command(flatten)]
        encode_options: EncodeOptions,

        #[command(flatten)]
        strip_options: StripOptions,
    },

    /// Compile a document
    Compile {
        /// The path of the file to compile
        ///
        /// If not supplied the input content is read from `stdin`.
        input: PathBuf,

        /// The path of the file to write the compiled document to
        ///
        /// If not supplied the output content is written to `stdout`.
        output: Option<PathBuf>,

        /// The format to encode to (or codec to use)
        ///
        /// Defaults to inferring the format from the file name extension
        /// of the `output`. If no `output` is supplied, defaults to JSON.
        #[arg(long, short)]
        to: Option<String>,

        #[command(flatten)]
        encode_options: EncodeOptions,

        #[command(flatten)]
        strip_options: StripOptions,
    },

    /// Execute a document
    #[command(alias = "exec")]
    Execute {
        /// The path of the file to execute
        ///
        /// If not supplied the input content is read from `stdin`.
        input: PathBuf,

        /// The path of the file to write the executed document to
        ///
        /// If not supplied the output content is written to `stdout`.
        output: Option<PathBuf>,

        /// The format to encode to (or codec to use)
        ///
        /// Defaults to inferring the format from the file name extension
        /// of the `output`. If no `output` is supplied, defaults to JSON.
        #[arg(long, short)]
        to: Option<String>,

        #[clap(flatten)]
        execute_options: ExecuteOptions,

        #[command(flatten)]
        encode_options: EncodeOptions,

        #[command(flatten)]
        strip_options: StripOptions,
    },

    /// Render a document
    ///
    /// Equivalent to the `execute` command with the `--render` flag.
    #[command()]
    Render {
        /// The path of the file to render
        ///
        /// If not supplied the input content is read from `stdin`.
        input: PathBuf,

        /// The path of the file to write the rendered document to
        ///
        /// If not supplied the output content is written to `stdout`.
        output: Option<PathBuf>,

        /// The format to encode to (or codec to use)
        ///
        /// Defaults to inferring the format from the file name extension
        /// of the `output`. If no `output` is supplied, defaults to Markdown.
        #[arg(long, short)]
        to: Option<String>,

        #[clap(flatten)]
        execute_options: ExecuteOptions,

        #[command(flatten)]
        encode_options: EncodeOptions,

        #[command(flatten)]
        strip_options: StripOptions,
    },

    Preview(preview::Cli),
    Publish(publish::cli::Cli),

    Serve(ServeOptions),

    /// Run the Language Server Protocol server
    Lsp,

    Prompts(prompts::cli::Cli),
    Models(models::cli::Cli),
    Kernels(kernels::cli::Cli),
    Codecs(codecs::cli::Cli),
    Plugins(plugins::cli::Cli),
    Secrets(secrets::cli::Cli),

    Config(ConfigOptions),

    Upgrade(upgrade::Cli),
    Uninstall(uninstall::Cli),
}

/// Command line arguments for stripping nodes
///
/// It is necessary to have this as a separate `struct` (rather than adding
/// these fields to both `DecodeOptions` and `EncodeOptions`) to avoid duplication
/// when DecodeOptions` and `EncodeOptions` are both flattened into `Sync` and `Convert`
/// commands.
#[derive(Debug, Clone, Args)]
pub struct StripOptions {
    /// Scopes defining which properties of nodes should be stripped
    #[arg(long)]
    strip_scopes: Vec<StripScope>,

    /// A list of node types to strip
    #[arg(long)]
    strip_types: Vec<String>,

    /// A list of node properties to strip
    #[arg(long)]
    strip_props: Vec<String>,
}

/// Command line arguments for decoding nodes from other formats
#[derive(Debug, Args)]
pub struct DecodeOptions {}

impl DecodeOptions {
    /// Build a set of [`codecs::DecodeOptions`] from command line arguments
    pub(crate) fn build(
        &self,
        format_or_codec: Option<String>,
        strip_options: StripOptions,
        losses: codecs::LossesResponse,
    ) -> codecs::DecodeOptions {
        let codec = format_or_codec
            .as_ref()
            .and_then(|name| codecs::codec_maybe(name));
        let format = format_or_codec.map(|name| Format::from_name(&name));

        codecs::DecodeOptions {
            codec,
            format,
            strip_scopes: strip_options.strip_scopes,
            strip_types: strip_options.strip_types,
            strip_props: strip_options.strip_props,
            losses,
            ..Default::default()
        }
    }
}

/// Command line arguments for encoding nodes to other formats
#[derive(Debug, Args)]
pub struct EncodeOptions {
    /// Encode as a standalone document
    #[arg(long, conflicts_with = "not_standalone")]
    standalone: bool,

    /// Do not encode as a standalone document when writing to file
    #[arg(long, conflicts_with = "standalone")]
    not_standalone: bool,

    /// For executable nodes, only encode outputs, not source properties
    #[arg(long, short)]
    render: bool,

    /// Use compact form of encoding if possible
    ///
    /// Use this flag to produce the compact forms of encoding (e.g. no indentation)
    /// which are supported by some formats (e.g. JSON, HTML).
    #[arg(long, short, conflicts_with = "pretty")]
    compact: bool,

    /// Use a "pretty" form of encoding if possible
    ///
    /// Use this flag to produce pretty forms of encoding (e.g. indentation)
    /// which are supported by some formats (e.g. JSON, HTML).
    #[arg(long, short, conflicts_with = "compact")]
    pretty: bool,
}

impl EncodeOptions {
    /// Build a set of [`codecs::EncodeOptions`] from command line arguments
    pub(crate) fn build(
        &self,
        output: Option<&Path>,
        format_or_codec: Option<String>,
        default_format: Format,
        strip_options: StripOptions,
        losses: codecs::LossesResponse,
    ) -> codecs::EncodeOptions {
        let codec = format_or_codec
            .as_ref()
            .and_then(|name| codecs::codec_maybe(name));

        let format = format_or_codec
            .map_or_else(
                || output.map(Format::from_path),
                |name| Some(Format::from_name(&name)),
            )
            .or(Some(default_format));

        let compact = self
            .compact
            .then_some(true)
            .or(self.pretty.then_some(false));

        let standalone = self
            .standalone
            .then_some(true)
            .or(self.not_standalone.then_some(false));

        let render = self.render.then_some(true);

        codecs::EncodeOptions {
            codec,
            format,
            compact,
            standalone,
            render,
            strip_scopes: strip_options.strip_scopes,
            strip_types: strip_options.strip_types,
            strip_props: strip_options.strip_props,
            losses,
            ..Default::default()
        }
    }
}

#[derive(Debug, Args)]
pub struct ConfigOptions {
    #[arg(long, default_value = "config")]
    dir: DirType,

    #[arg(long)]
    ensure: bool,
}

impl Cli {
    /// Run the CLI command
    ///
    /// This function mainly exists to have a top level, instrumented function
    /// to call after error reporting and logging have been setup. This is
    /// useful because then CLI arguments are captured in span traces.
    #[tracing::instrument(skip(self))]
    pub async fn run(self) -> Result<()> {
        tracing::trace!("Running CLI command");

        match self.command {
            Command::New { .. } => {
                Document::new()?;
            }

            /*
            Command::Import {
                doc,
                source,
                from,
                losses,
                strip_options,
                ..
            } => {
                let doc = Document::open(&doc).await?;

                let options = DecodeOptions {}.build(from, strip_options, losses);

                doc.import(&source, Some(options)).await?;
            }

            Command::Export {
                doc,
                dest,
                to,
                losses,
                options,
                strip_options,
            } => {
                let doc = Document::open(&doc).await?;

                let options =
                    options.build(dest.as_deref(), to, Format::Json, strip_options, losses);

                let content = doc.export(dest.as_deref(), Some(options.clone())).await?;
                if !content.is_empty() {
                    Code::new(options.format.unwrap_or_default(), &content).to_stdout();
                }
            }
            */
            Command::Sync(sync) => sync.run().await?,

            Command::Convert {
                input,
                output,
                from,
                to,
                input_losses,
                output_losses,
                decode_options,
                encode_options,
                strip_options,
            } => {
                let decode_options =
                    decode_options.build(from, strip_options.clone(), input_losses);
                let encode_options = encode_options.build(
                    output.as_deref(),
                    to,
                    Format::Json,
                    strip_options,
                    output_losses,
                );

                let content = codecs::convert(
                    input.as_deref(),
                    output.as_deref(),
                    Some(decode_options),
                    Some(encode_options.clone()),
                )
                .await?;

                if !content.is_empty() {
                    Code::new(encode_options.format.unwrap_or_default(), &content).to_stdout();
                }
            }

            Command::Compile {
                input,
                output,
                to,
                encode_options,
                strip_options,
            } => {
                let doc = Document::open(&input).await?;
                doc.compile(true).await?;

                let encode_options = encode_options.build(
                    output.as_deref(),
                    to,
                    Format::Json,
                    strip_options,
                    LossesResponse::Debug,
                );

                let content = doc
                    .export(output.as_deref(), Some(encode_options.clone()))
                    .await?;

                if !content.is_empty() {
                    Code::new(encode_options.format.unwrap_or_default(), &content).to_stdout();
                }
            }

            Command::Execute {
                input,
                output,
                to,
                execute_options,
                encode_options,
                strip_options,
            } => {
                let doc = Document::open(&input).await?;
                doc.compile(true).await?;
                doc.execute(execute_options, true).await?;

                let encode_options = encode_options.build(
                    output.as_deref(),
                    to,
                    Format::Json,
                    strip_options,
                    LossesResponse::Debug,
                );

                let content = doc
                    .export(output.as_deref(), Some(encode_options.clone()))
                    .await?;

                if !content.is_empty() {
                    Code::new(encode_options.format.unwrap_or_default(), &content).to_stdout();
                }
            }

            Command::Render {
                input,
                output,
                to,
                execute_options,
                encode_options,
                strip_options,
            } => {
                let doc = Document::open(&input).await?;
                doc.compile(true).await?;
                doc.execute(execute_options, true).await?;

                let mut encode_options = encode_options.build(
                    output.as_deref(),
                    to,
                    Format::Markdown,
                    strip_options,
                    LossesResponse::Debug,
                );
                encode_options.render = Some(true);

                let content = doc
                    .export(output.as_deref(), Some(encode_options.clone()))
                    .await?;

                if !content.is_empty() {
                    Code::new(encode_options.format.unwrap_or_default(), &content).to_stdout();
                }
            }

            Command::Preview(preview) => preview.run().await?,
            Command::Publish(publish) => publish.run().await?,

            Command::Serve(options) => serve(options).await?,

            Command::Lsp => lsp::run().await,

            Command::Prompts(prompts) => prompts.run().await?,
            Command::Models(models) => models.run().await?,
            Command::Kernels(kernels) => kernels.run().await?,
            Command::Codecs(codecs) => codecs.run().await?,
            Command::Plugins(plugins) => plugins.run().await?,
            Command::Secrets(secrets) => secrets.run().await?,

            Command::Config(options) => {
                // TODO: Make options.dir an option, and if it not there, show all folders.
                let dir = app::get_app_dir(options.dir, options.ensure)?;
                println!("{}", dir.display());
            }

            Command::Upgrade(upgrade) => upgrade.run().await?,
            Command::Uninstall(uninstall) => uninstall.run()?,
        }

        Ok(())
    }
}
