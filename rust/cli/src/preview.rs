use std::path::PathBuf;

use common::{
    clap::{self, Parser},
    eyre::{OptionExt, Result},
    tokio,
};
use document::SyncDirection;
use server::{get_access_token, ServeOptions};

/// Preview a document or site
///
/// Opens a preview of a document or site in the browser.
/// When `--sync=in` (the default) the preview will update when
/// the document is saved to disk.
#[derive(Debug, Parser)]
pub struct Cli {
    /// The path to the document file or site directory to preview
    ///
    /// Defaults to the current directory.
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Which direction(s) to sync the document
    #[arg(long, default_value = "in")]
    sync: SyncDirection,
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let path = self.path.canonicalize()?;

        // Serve the directory of the path
        let dir = if path.is_file() {
            path.parent().ok_or_eyre("File has no parent")?
        } else {
            &path
        }
        .to_path_buf();

        // Get (or generate) an access token so it can be included
        // in the URL
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
        let path = path.strip_prefix(&dir)?.to_string_lossy();
        let url = format!("http://127.0.0.1:9000/~login?access_token={access_token}&next={path}");
        webbrowser::open(&url)?;

        // Await the serve task
        serve.await??;

        Ok(())
    }
}
