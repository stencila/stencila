use std::path::PathBuf;

use codecs::{DecodeOptions, EncodeOptions};
use common::{
    clap::{self, Parser, Subcommand},
    eyre::Result,
    tokio, tracing,
};
use document::{Document, DocumentType, SyncDirection};
use format::Format;

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
        #[arg(long, short)]
        format: Option<Format>,

        /// The type of document to import
        ///
        /// Defaults to determining the type based on the `format`, or for
        /// formats such as JSON and YAML, the value of the root `type` property.
        #[arg(long, short)]
        r#type: Option<DocumentType>,
    },

    /// Export a document to a file in another format
    Export {
        /// The path of the document to export from
        doc: PathBuf,

        /// The destination file to export to
        dest: Option<PathBuf>,

        /// The format of the destination file
        #[arg(long, short)]
        format: Option<Format>,
    },

    /// Synchronize a document with one of more other files in other formats
    Sync {
        /// The path of the document to synchronize
        doc: PathBuf,

        /// The files to synchronize with
        files: Vec<PathBuf>,

        /// The formats of the files
        /// 
        /// This option can be provided separately for each file.
        #[arg(long = "format", short)]
        formats: Vec<Format>,

        /// The synchronization directions to use for each file
        /// 
        /// This option can be provided separately for each file.
        #[arg(long = "dir", short)]
        directions: Vec<SyncDirection>,
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

        /// The format to encode from
        ///
        /// Defaults to inferring the format from the file name extension
        /// of the `input`.
        #[arg(long, short)]
        from: Option<Format>,

        /// The format to encode to
        ///
        /// Defaults to inferring the format from the file name extension
        /// of the `output`. If no `output` is supplied, defaults to JSON.
        #[arg(long, short)]
        to: Option<Format>,

        /// Use compact form of encoding if possible
        ///
        /// Use this flag to enable compact forms of encoding (i.e. no indentation)
        /// which are supported by some formats (e.g. JSON, HTML).
        #[arg(long, short)]
        compact: bool,
    },
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
        } => {
            Document::new(
                r#type,
                path.as_deref(),
                overwrite,
                source.as_deref(),
                format,
            )
            .await?;
        }

        Command::Import {
            doc,
            source,
            format,
            r#type,
        } => {
            let doc = Document::open(&doc).await?;
            doc.import(&source, format, r#type).await?;
        }

        Command::Export { doc, dest, format } => {
            let doc = Document::open(&doc).await?;
            let content = doc.export(dest.as_deref(), format).await?;
            if !content.is_empty() {
                println!("{}", content)
            }
        }

        Command::Sync {
            doc,
            files,
            formats,
            directions,
        } => {
            let doc = Document::open(&doc).await?;
            for (index, file) in files.iter().enumerate() {
                let format = formats.get(index).copied();
                let direction = directions.get(index).copied();
                doc.sync_file(file, format, direction).await?;
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
            compact,
        } => {
            let decode_options = DecodeOptions { format: from };

            let encode_options = EncodeOptions {
                format: to,
                compact,
            };

            let content = codecs::convert(
                input.as_deref(),
                output.as_deref(),
                Some(decode_options),
                Some(encode_options),
            )
            .await?;
            if !content.is_empty() {
                println!("{}", content)
            }
        }
    }

    if wait {
        use tokio::time::{sleep, Duration};
        sleep(Duration::from_secs(u64::MAX)).await;
    }

    Ok(())
}
