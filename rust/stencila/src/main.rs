use std::path::PathBuf;

use common::{
    clap::{self, Args, Parser, Subcommand},
    eyre::Result,
    tokio, tracing,
};
use document::{Document, DocumentType, SyncDirection};
use format::Format;

mod display;
mod errors;
mod logging;

use crate::logging::{LoggingFormat, LoggingLevel};

/// Main entry function
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    errors::setup(&cli.error_details, cli.error_link)?;
    logging::setup(cli.log_level, &cli.log_filter, cli.log_format)?;
    run(cli).await
}

/// CLI subcommands and global options
#[derive(Debug, Parser)]
#[command(name = "stencila", author, version, about, long_about)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// The minimum log level to output
    #[arg(long, default_value = "info", global = true)]
    log_level: LoggingLevel,

    /// A filter for log entries
    ///
    /// Allows more fine-grained control over which log entries are shown.
    /// To additionally see lower level entries for a specific crates use
    /// syntax such as `tokio_postgres=debug`.
    #[arg(long, default_value = "", global = true)]
    log_filter: String,

    /// The log format to use
    ///
    /// When `auto`, uses `simple` for terminals and `json`
    /// for non-TTY devices.
    #[arg(long, default_value = "auto", global = true)]
    log_format: LoggingFormat,

    /// The details to include in error reports
    ///
    /// A comma separated list including `location`, `span`, or `env`.
    #[arg(long, default_value = "auto", global = true)]
    error_details: String,

    /// Output a link to more easily report an issue
    #[arg(long, global = true)]
    error_link: bool,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Create a new document
    New {
        /// The type of document to create
        #[arg(default_value_t = DocumentType::Article)]
        r#type: DocumentType,

        /// The path of the document to create
        path: Option<PathBuf>,

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

    /// Import a file in another format into a new or existing document
    Import {
        /// The path of the document to create or import to
        doc: PathBuf,

        /// The source file to import from
        source: PathBuf,

        /// The format of the source file
        #[arg(long, short, alias = "from")]
        format: Option<Format>,

        /// The codec to use to decode the source
        #[arg(long)]
        codec: Option<String>,

        /// The type of document to import
        ///
        /// Defaults to determining the type based on the `format`, or for
        /// formats such as JSON and YAML, the value of the root `type` property.
        #[arg(long, short)]
        r#type: Option<DocumentType>,

        /// What to do if there are losses when decoding
        #[arg(long, short, default_value_t = codecs::LossesResponse::Warn)]
        losses: codecs::LossesResponse,

        #[command(flatten)]
        options: DecodeOptions,
    },

    /// Export a document to a file in another format
    Export {
        /// The path of the document to export from
        doc: PathBuf,

        /// The destination file to export to
        dest: Option<PathBuf>,

        /// The format of the destination file
        #[arg(long, short, alias = "to")]
        format: Option<Format>,

        /// The codec to use to encode to the destination
        #[arg(long)]
        codec: Option<String>,

        /// What to do if there are losses when encoding
        #[arg(long, short, default_value_t = codecs::LossesResponse::Warn)]
        losses: codecs::LossesResponse,

        #[command(flatten)]
        options: EncodeOptions,
    },

    /// Synchronize a document with one of more other files in other formats
    Sync {
        /// The path of the document to synchronize
        doc: PathBuf,

        /// The files to synchronize with
        files: Vec<PathBuf>,

        /// The formats of the files (or the name of codecs to use)
        ///
        /// This option can be provided separately for each file.
        #[arg(long = "format", short)]
        formats: Vec<String>,

        /// The synchronization directions to use for each file
        ///
        /// This option can be provided separately for each file.
        #[arg(long = "dir", short)]
        directions: Vec<SyncDirection>,

        /// What to do if there are losses when either encoding or decoding between any of the files
        #[arg(long, short, default_value_t = codecs::LossesResponse::Warn)]
        losses: codecs::LossesResponse,

        #[command(flatten)]
        decode_options: DecodeOptions,

        #[command(flatten)]
        encode_options: EncodeOptions,
    },

    /// Display the history of commits to the document
    History {
        /// The path of the document to display the history for
        doc: PathBuf,
    },

    /// Inspect a document as JSON
    ///
    /// This command is mostly intended for debugging issues with loading a
    /// document from file storage.
    Inspect {
        /// The path of the document to inspect
        doc: PathBuf,
    },

    /// Convert a document between formats
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

        /// What to do if there are losses when either decoding from the input, or encoding to the output
        #[arg(long, short, default_value_t = codecs::LossesResponse::Warn)]
        losses: codecs::LossesResponse,

        #[command(flatten)]
        decode_options: DecodeOptions,

        #[command(flatten)]
        encode_options: EncodeOptions,
    },

    /// Get available format conversion codecs
    #[command(alias = "codec")]
    Codecs {
        /// The name of the codec to show details for
        name: Option<String>,
    },
}

/// Command line arguments for decoding nodes from other formats
#[derive(Debug, Args)]
struct DecodeOptions {}

impl DecodeOptions {
    /// Build a set of [`codecs::DecodeOptions`] from command line arguments
    fn build(
        &self,
        format_or_codec: Option<String>,
        losses: codecs::LossesResponse,
    ) -> codecs::DecodeOptions {
        let (format, codec) = infer_format_or_codec(format_or_codec);

        codecs::DecodeOptions {
            codec,
            format,
            losses,
        }
    }
}

/// Command line arguments for encoding nodes to other formats
#[derive(Debug, Args)]
struct EncodeOptions {
    /// Use compact form of encoding if possible
    ///
    /// Use this flag to enable compact forms of encoding (i.e. no indentation)
    /// which are supported by some formats (e.g. JSON, HTML).
    #[arg(long, short)]
    compact: bool,

    /// Do not strip the id property of nodes when encoding
    #[arg(long)]
    no_strip_id: bool,

    /// Strip the code of executable nodes when encoding
    #[arg(long)]
    strip_code: bool,

    /// Strip derived properties of executable nodes when encoding
    #[arg(long)]
    strip_execution: bool,

    /// Strip the outputs of executable nodes when encoding
    #[arg(long)]
    strip_outputs: bool,
}

impl EncodeOptions {
    /// Build a set of [`codecs::EncodeOptions`] from command line arguments
    fn build(
        &self,
        format_or_codec: Option<String>,
        losses: codecs::LossesResponse,
    ) -> codecs::EncodeOptions {
        let (format, codec) = infer_format_or_codec(format_or_codec);

        codecs::EncodeOptions {
            codec,
            format,
            compact: self.compact,
            strip_id: !self.no_strip_id,
            strip_code: self.strip_code,
            strip_execution: self.strip_execution,
            strip_outputs: self.strip_outputs,
            losses,
        }
    }
}

/// If the string matches the name of a format then assume it is a format, otherwise assume it is a codec name
fn infer_format_or_codec(format_or_codec: Option<String>) -> (Option<Format>, Option<String>) {
    match format_or_codec {
        Some(format_or_codec) => match Format::from_name(&format_or_codec.to_lowercase()) {
            Ok(format) => (Some(format), None),
            Err(..) => (None, Some(format_or_codec)),
        },
        None => (None, None),
    }
}

/// Run the CLI command
///
/// This function mainly exists to have a top level, instrumented function
/// to call after error reporting and logging have been setup. This is
/// useful because then CLI arguments are captured in span traces.
#[tracing::instrument(skip(cli))]
async fn run(cli: Cli) -> Result<()> {
    tracing::trace!("Running CLI command");

    let mut wait = false;
    match cli.command {
        Command::New {
            r#type,
            path,
            overwrite,
            source,
            format,
            codec,
        } => {
            Document::new(
                r#type,
                path.as_deref(),
                overwrite,
                source.as_deref(),
                format,
                codec,
            )
            .await?;
        }

        Command::Import {
            doc,
            source,
            format,
            codec,
            r#type,
            losses,
            ..
        } => {
            let doc = Document::open(&doc).await?;

            let options = codecs::DecodeOptions {
                codec,
                format,
                losses,
            };

            doc.import(&source, Some(options), r#type).await?;
        }

        Command::Export {
            doc,
            dest,
            format,
            codec,
            losses,
            options,
        } => {
            let doc = Document::open(&doc).await?;

            let options = codecs::EncodeOptions {
                codec,
                format,
                compact: options.compact,
                strip_id: !options.no_strip_id,
                strip_code: options.strip_code,
                strip_execution: options.strip_execution,
                strip_outputs: options.strip_outputs,
                losses,
            };

            let content = doc.export(dest.as_deref(), Some(options)).await?;
            if !content.is_empty() {
                let format = format.unwrap_or(Format::Json);
                display::highlighted(&content, format)?;
            }
        }

        Command::Sync {
            doc,
            files,
            formats,
            directions,
            losses,
            decode_options,
            encode_options,
        } => {
            let doc = Document::open(&doc).await?;

            for (index, file) in files.iter().enumerate() {
                let format_or_codec = formats.get(index).cloned();
                let direction = directions.get(index).copied();

                let decode_options = Some(decode_options.build(format_or_codec.clone(), losses));
                let encode_options = Some(encode_options.build(format_or_codec, losses));

                doc.sync_file(file, direction, decode_options, encode_options)
                    .await?;
            }
            wait = true;
        }

        Command::History { doc } => {
            let doc = Document::open(&doc).await?;
            doc.history().await?;
        }

        Command::Inspect { doc } => {
            let json = Document::inspect(&doc).await?;
            println!("{}", json);
        }

        Command::Convert {
            input,
            output,
            from,
            to,
            losses,
            decode_options,
            encode_options,
        } => {
            let decode_options = decode_options.build(from, losses);
            let encode_options = encode_options.build(to, losses);

            let content = codecs::convert(
                input.as_deref(),
                output.as_deref(),
                Some(decode_options),
                Some(encode_options.clone()),
            )
            .await?;
            if !content.is_empty() {
                let format = encode_options.format.unwrap_or(Format::Json);
                display::highlighted(&content, format)?;
            }
        }

        Command::Codecs { name } => match name {
            Some(name) => println!("{:#?}", codecs::spec(&name)?),
            None => println!("{:#?}", codecs::specs()),
        },
    }

    if wait {
        use tokio::time::{sleep, Duration};
        sleep(Duration::from_secs(u64::MAX)).await;
    }

    Ok(())
}
