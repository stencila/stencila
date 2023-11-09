use std::path::PathBuf;

use yansi::Color;

use common::{
    chrono::{Local, SecondsFormat, TimeZone},
    clap::{self, Args, Parser, Subcommand},
    eyre::{eyre, Result},
    tokio, tracing,
};
use document::{Document, DocumentType, SyncDirection};
use format::Format;
use node_strip::StripScope;
use server::{serve, ServeOptions};

use crate::{
    display,
    logging::{LoggingFormat, LoggingLevel},
};

/// CLI subcommands and global options
#[derive(Debug, Parser)]
#[command(name = "stencila", author, version, about, long_about)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,

    /// The minimum log level to output
    #[arg(long, default_value = "info", global = true)]
    pub log_level: LoggingLevel,

    /// A filter for log entries
    ///
    /// Allows more fine-grained control over which log entries are shown.
    /// To additionally see lower level entries for a specific crates use
    /// syntax such as `tokio_postgres=debug`.
    #[arg(long, default_value = "", global = true)]
    pub log_filter: String,

    /// The log format to use
    ///
    /// When `auto`, uses `simple` for terminals and `json`
    /// for non-TTY devices.
    #[arg(long, default_value = "auto", global = true)]
    pub log_format: LoggingFormat,

    /// The details to include in error reports
    ///
    /// A comma separated list including `location`, `span`, or `env`.
    #[arg(long, default_value = "auto", global = true)]
    pub error_details: String,

    /// Output a link to more easily report an issue
    #[arg(long, global = true)]
    pub error_link: bool,
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

        #[command(flatten)]
        strip_options: StripOptions,
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

        /// What to do if there are losses when either encoding or decoding between any of the files
        #[arg(long, short, default_value_t = codecs::LossesResponse::Warn)]
        losses: codecs::LossesResponse,

        #[command(flatten)]
        decode_options: DecodeOptions,

        #[command(flatten)]
        encode_options: EncodeOptions,

        #[command(flatten)]
        strip_options: StripOptions,
    },

    /// Display the history of commits to the document
    Log {
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

    /// Serve
    Serve(ServeOptions),
}

/// Command line arguments for stripping nodes
///
/// It is necessary to have this as a separate `struct` (rather than adding
/// these fields to both `DecodeOptions` and `EncodeOptions`) to avoid duplication
/// when DecodeOptions` and `EncodeOptions` are both flattened into `Sync` and `Convert`
/// commands.
#[derive(Debug, Clone, Args)]
struct StripOptions {
    /// Scopes defining which properties of nodes should be stripped
    #[arg(long)]
    strip_scopes: Vec<StripScope>,

    /// A list of node types to strip
    #[arg(long)]
    strip_types: Vec<String>,

    /// A list of node properties to strip
    #[arg(long, default_value = "id")]
    strip_props: Vec<String>,
}

/// Command line arguments for decoding nodes from other formats
#[derive(Debug, Args)]
struct DecodeOptions {}

impl DecodeOptions {
    /// Build a set of [`codecs::DecodeOptions`] from command line arguments
    fn build(
        &self,
        format_or_codec: Option<String>,
        strip_options: StripOptions,
        losses: codecs::LossesResponse,
    ) -> codecs::DecodeOptions {
        let (format, codec) = codecs::format_or_codec(format_or_codec);

        codecs::DecodeOptions {
            codec,
            format,
            strip_scopes: strip_options.strip_scopes,
            strip_types: strip_options.strip_types,
            strip_props: strip_options.strip_props,
            losses,
        }
    }
}

/// Command line arguments for encoding nodes to other formats
#[derive(Debug, Args)]
struct EncodeOptions {
    /// Encode as a standalone document
    #[arg(long, conflicts_with = "not_standalone")]
    standalone: bool,

    /// Do not encode as a standalone document when writing to file
    #[arg(long, conflicts_with = "standalone")]
    not_standalone: bool,

    /// Use compact form of encoding if possible
    ///
    /// Use this flag to enable compact forms of encoding (i.e. no indentation)
    /// which are supported by some formats (e.g. JSON, HTML).
    #[arg(long, short)]
    compact: bool,
}

impl EncodeOptions {
    /// Build a set of [`codecs::EncodeOptions`] from command line arguments
    fn build(
        &self,
        format_or_codec: Option<String>,
        strip_options: StripOptions,
        losses: codecs::LossesResponse,
    ) -> codecs::EncodeOptions {
        let (format, codec) = codecs::format_or_codec(format_or_codec);
        let standalone = self
            .standalone
            .then_some(true)
            .or(self.not_standalone.then_some(false));

        codecs::EncodeOptions {
            codec,
            format,
            compact: self.compact,
            standalone,
            strip_scopes: strip_options.strip_scopes,
            strip_types: strip_options.strip_types,
            strip_props: strip_options.strip_props,
            losses,
        }
    }
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

        let mut wait = false;
        match self.command {
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
                strip_options,
                ..
            } => {
                let doc = Document::open(&doc).await?;

                let options = codecs::DecodeOptions {
                    codec,
                    format,
                    strip_scopes: strip_options.strip_scopes,
                    strip_types: strip_options.strip_types,
                    strip_props: strip_options.strip_props,
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
                strip_options,
            } => {
                let doc = Document::open(&doc).await?;

                let options = codecs::EncodeOptions {
                    codec,
                    format,
                    standalone: options.not_standalone.then_some(false),
                    compact: options.compact,
                    strip_scopes: strip_options.strip_scopes,
                    strip_types: strip_options.strip_types,
                    strip_props: strip_options.strip_props,
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
                losses,
                decode_options,
                encode_options,
                strip_options,
            } => {
                let doc = Document::open(&doc).await?;

                for (index, file) in files.iter().enumerate() {
                    let file = file.to_string_lossy();
                    let (file, direction) = if file.ends_with(":in") {
                        (file.trim_end_matches(":in"), SyncDirection::In)
                    } else if file.ends_with(":out") {
                        (file.trim_end_matches(":out"), SyncDirection::Out)
                    } else if file.ends_with(":io") {
                        (file.trim_end_matches(":io"), SyncDirection::InOut)
                    } else {
                        (file.as_ref(), SyncDirection::InOut)
                    };
                    let file = PathBuf::from(file);

                    let format_or_codec = formats.get(index).cloned();

                    let decode_options = Some(decode_options.build(
                        format_or_codec.clone(),
                        strip_options.clone(),
                        losses.clone(),
                    ));
                    let encode_options = Some(encode_options.build(
                        format_or_codec,
                        strip_options.clone(),
                        losses.clone(),
                    ));

                    if file.ends_with("-") {
                        let (change_sender, mut change_receiver) =
                            common::tokio::sync::mpsc::channel(32);
                        tokio::spawn(async move {
                            while let Some(change) = change_receiver.recv().await {
                                tracing::info!("Change {change:?}");
                            }
                        });
                        doc.sync_string(None, change_sender, decode_options, encode_options)
                            .await?;
                    } else {
                        doc.sync_file(&file, direction, decode_options, encode_options)
                            .await?;
                    }
                }
                wait = true;
            }

            Command::Log { doc } => {
                let doc = Document::open(&doc).await?;
                let log = doc.log().await?;

                for entry in log {
                    let date = Local
                        .timestamp_opt(entry.timestamp, 0)
                        .single()
                        .ok_or_else(|| eyre!("invalid timestamp"))?
                        .to_rfc3339_opts(SecondsFormat::Secs, true);
                    let date = Color::Blue.paint(date);

                    let author = Color::Green.paint(entry.author);
                    let hash = Color::White.style().dimmed().paint(entry.hash);
                    let message = entry.message;

                    println!(
                        "{date} {author}
{hash}

{message}
"
                    )
                }
            }

            Command::Inspect { doc } => {
                let json = Document::inspect(&doc).await?;
                display::highlighted(&json, Format::Json)?;
            }

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
                let encode_options = encode_options.build(to, strip_options, output_losses);

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

            Command::Serve(options) => serve(options).await?,
        }

        if wait {
            use tokio::time::{sleep, Duration};
            sleep(Duration::from_secs(u64::MAX)).await;
        }

        Ok(())
    }
}
