use std::path::PathBuf;

use clap::Parser;
use eyre::{OptionExt, Result, bail};

use stencila_cli_utils::{color_print::cstr, message};
use stencila_document::{Document, SyncDirection};
use stencila_server::{ServeOptions, get_server_token};

/// Open a document in the browser
///
/// Opens a document in the browser. If the path supplied is a folder then
/// the first file with name `index.*`, `main.*`, or `readme.*` will be opened.
///
/// By default, opens both a local preview server and any tracked remote URLs
/// (e.g., Google Docs, Microsoft 365). Use `--no-local` or `--no-remotes` to
/// open only one or the other.
///
/// When `--sync=in` (the default) the local preview will update when
/// the document is changed and saved to disk.
#[derive(Debug, Parser)]
#[command(alias = "preview", after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// The path to the document or parent folder
    ///
    /// Defaults to the current folder.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Which direction(s) to sync the document
    #[arg(long, default_value = "in")]
    sync: SyncDirection,

    /// Do not open the local preview server
    #[arg(long, conflicts_with = "no_remotes")]
    no_local: bool,

    /// Do not open tracked remote URLs
    #[arg(long, conflicts_with = "no_local")]
    no_remotes: bool,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Open a specific document</dim>
  <b>stencila open</b> <g>document.md</g>

  <dim># Open from current directory (finds index/main/readme)</dim>
  <b>stencila open</b>

  <dim># Open a document in a specific folder</dim>
  <b>stencila open</b> <g>report/main.smd</g>

  <dim># Open only tracked remotes (skip local preview)</dim>
  <b>stencila open</b> <g>document.md</g> <c>--no-local</c>

  <dim># Open only local preview (skip remotes)</dim>
  <b>stencila open</b> <g>document.md</g> <c>--no-remotes</c>
"
);

impl Cli {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            sync: SyncDirection::In,
            no_local: false,
            no_remotes: false,
        }
    }

    pub async fn run(self) -> Result<()> {
        // Resolve the path to a document file
        let Some(file) = Document::resolve_file(&self.path)? else {
            bail!(
                "Unable to resolve which document file to open from path {}",
                self.path.display()
            )
        };

        let file = file.canonicalize()?;

        // Open the document to get tracked remotes
        let doc = Document::open(&file, None).await?;
        let remotes = doc.remotes().await?;

        // Open remote URLs in browser if not disabled
        if !self.no_remotes && !remotes.is_empty() {
            message(
                &format!("Opening {} tracked remote(s) in browser", remotes.len()),
                Some("🌐"),
            );
            for remote_url in &remotes {
                webbrowser::open(remote_url.as_str())?;
                message(&format!("Opened {remote_url}"), Some("✅"));
            }
        }

        // Open local preview server if not disabled
        if !self.no_local {
            // Serve the parent directory of the file
            let dir = file
                .parent()
                .ok_or_eyre("File has no parent")?
                .to_path_buf();

            // Get (or generate) a server token so it can be included in the URL
            let server_token = get_server_token();

            message("Opening local preview server", Some("🖥️ "));

            // Serve the directory
            let options = ServeOptions {
                dir: dir.clone(),
                sync: Some(self.sync),
                server_token: Some(server_token.clone()),
                ..Default::default()
            };
            let serve = tokio::spawn(async move { stencila_server::serve(options).await });

            // Open the browser to the login page with redirect to the document path
            let path = file.strip_prefix(&dir)?.to_string_lossy();
            let url = format!("http://127.0.0.1:9000/~login?sst={server_token}&next={path}");
            webbrowser::open(&url)?;

            // Await the serve task
            serve.await??;
        }

        Ok(())
    }
}
