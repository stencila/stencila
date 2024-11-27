use std::path::{Path, PathBuf};

use common::{
    clap::{self, Parser},
    eyre::Result,
    tokio,
};
use document::{Document, SyncDirection};
use format::Format;

use crate::options::{DecodeOptions, EncodeOptions, StripOptions};

/// Synchronize a document between formats
///
/// The direction of synchronization can be specified by appending
/// the to the file path:
///
/// - `:in` only sync incoming changes from the file
/// - `:out` only sync outgoing changes to the file
/// - `:io` sync incoming and outgoing changes (default)
#[derive(Debug, Parser)]
pub struct Cli {
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
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let (main, direction) = resolve_path_direction(&self.doc);
        let doc = Document::synced(&main, direction).await?;

        for (index, file) in self.files.iter().enumerate() {
            let (path, direction) = resolve_path_direction(file);

            let format_or_codec = self.formats.get(index).cloned();

            let decode_options = Some(self.decode_options.build(
                format_or_codec.clone(),
                self.strip_options.clone(),
                self.losses.clone(),
                Vec::new(),
            ));
            let encode_options = Some(self.encode_options.build(
                Some(main.as_ref()),
                Some(&path),
                format_or_codec,
                Format::Json,
                self.strip_options.clone(),
                self.losses.clone(),
                Vec::new(),
            ));

            doc.sync_file(&path, direction, decode_options, encode_options)
                .await?;
        }

        // Sleep forever (or until Ctrl+C is pressed)
        use tokio::time::{sleep, Duration};
        sleep(Duration::from_secs(u64::MAX)).await;

        Ok(())
    }
}

fn resolve_path_direction(path: &Path) -> (PathBuf, SyncDirection) {
    let path = path.to_string_lossy();

    let (path, direction) = if path.ends_with(":in") {
        (path.trim_end_matches(":in"), SyncDirection::In)
    } else if path.ends_with(":out") {
        (path.trim_end_matches(":out"), SyncDirection::Out)
    } else if path.ends_with(":io") {
        (path.trim_end_matches(":io"), SyncDirection::InOut)
    } else {
        (path.as_ref(), SyncDirection::InOut)
    };

    (PathBuf::from(path), direction)
}
