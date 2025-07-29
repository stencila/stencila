use std::path::{Path, PathBuf};

use cli_utils::color_print::cstr;
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
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// The path of the document to synchronize
    doc: PathBuf,

    /// The files to synchronize with
    files: Vec<PathBuf>,

    #[command(flatten)]
    decode_options: DecodeOptions,

    #[command(flatten)]
    encode_options: EncodeOptions,

    #[command(flatten)]
    strip_options: StripOptions,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Sync a Markdown document with HTML (bidirectional)</dim>
  <b>stencila sync</b> <g>document.md</g> <g>preview.html</g>

  <dim># Sync with multiple formats</dim>
  <b>stencila sync</b> <g>source.md</g> <g>output.html</g> <g>output.pdf</g>

  <dim># Sync only incoming changes from HTML</dim>
  <b>stencila sync</b> <g>document.md</g> <g>edited.html:in</g>

  <dim># Sync only outgoing changes to PDF</dim>
  <b>stencila sync</b> <g>document.md</g> <g>output.pdf:out</g>

  <dim># Mixed sync directions</dim>
  <b>stencila sync</b> <g>main.md</g> <g>preview.html:out</g> <g>edits.docx:in</g>

  <dim># Sync with custom encoding options</dim>
  <b>stencila sync</b> <g>doc.md</g> <g>output.html</g> <c>--standalone</c>

<bold><b>Sync Directions</b></bold>
  • <c>:in</c> - Only accept incoming changes from the file
  • <c>:out</c> - Only push outgoing changes to the file
  • <c>:io</c> - Bidirectional sync (default)

<bold><b>Note</b></bold>
  The sync command runs continuously, watching for changes.
  Press Ctrl+C to stop synchronization.
"
);

impl Cli {
    pub async fn run(self) -> Result<()> {
        let (main, direction) = resolve_path_direction(&self.doc);
        let doc = Document::synced(&main, direction).await?;

        for file in self.files.iter() {
            let (path, direction) = resolve_path_direction(file);

            let decode_options = Some(
                self.decode_options
                    .build(Some(main.as_ref()), self.strip_options.clone()),
            );
            let encode_options = Some(self.encode_options.build(
                Some(main.as_ref()),
                Some(&path),
                Format::Json,
                self.strip_options.clone(),
            ));

            doc.sync_file(&path, direction, decode_options, encode_options)
                .await?;
        }

        // Sleep forever (or until Ctrl+C is pressed)
        use tokio::time::{Duration, sleep};
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
