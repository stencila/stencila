use std::path::PathBuf;

use cli_utils::color_print::cstr;
use common::{
    clap::{self, Parser},
    eyre::{bail, OptionExt, Result},
    tokio,
};
use document::{Document, SyncDirection};
use server::{get_access_token, ServeOptions};

/// Preview a document
///
/// Opens a preview of a document in the browser. If the path supplied
/// is a folder then the first file with name `index.*`, `main.*`, or `readme.*`
/// will be opened.
///
/// When `--sync=in` (the default) the preview will update when
/// the document is changed and saved to disk.
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// The path to the document or parent folder
    ///
    /// Defaults to the current folder.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Which direction(s) to sync the document
    #[arg(long, default_value = "in")]
    sync: SyncDirection,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Preview a specific document</dim>
  <b>stencila preview</b> <g>document.md</g>

  <dim># Preview from current directory (finds index/main/readme)</dim>
  <b>stencila preview</b>

  <dim># Preview a document in a specific folder</dim>
  <b>stencila preview</b> <g>docs/</g>

  <dim># Preview without syncing (static preview)</dim>
  <b>stencila preview</b> <g>report.pdf</g> <c>--sync</c> <g>none</g>"
);

impl Cli {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            sync: SyncDirection::In,
        }
    }

    pub async fn run(self) -> Result<()> {
        // Resolve the path to a document file
        let Some(file) = Document::resolve_file(&self.path)? else {
            bail!(
                "Unable to resolve which document file to preview from path {}",
                self.path.display()
            )
        };

        let file = file.canonicalize()?;

        // Serve the parent directory of the file
        let dir = file
            .parent()
            .ok_or_eyre("File has no parent")?
            .to_path_buf();

        // Get (or generate) an access token so it can be included in the URL
        let access_token = get_access_token();

        // Serve the directory
        let options = ServeOptions {
            dir: dir.clone(),
            sync: Some(self.sync),
            access_token: Some(access_token.clone()),
            ..Default::default()
        };
        let serve = tokio::spawn(async move { server::serve(options).await });

        // Open the browser to the login page with redirect to the document path
        let path = file.strip_prefix(&dir)?.to_string_lossy();
        let url = format!("http://127.0.0.1:9000/~login?access_token={access_token}&next={path}");
        webbrowser::open(&url)?;

        // Await the serve task
        serve.await??;

        Ok(())
    }
}
