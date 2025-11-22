use std::path::PathBuf;

use clap::Parser;
use eyre::{OptionExt, Result, bail, eyre};
use url::Url;

use stencila_cli_utils::{color_print::cstr, message};
use stencila_document::{Document, SyncDirection};
use stencila_remotes::{RemoteService, get_remotes_for_path};
use stencila_server::{ServeOptions, get_server_token};

/// Open a document in the browser
///
/// Opens a document in the browser. If the path supplied is a folder then
/// the first file with name `index.*`, `main.*`, or `readme.*` will be opened.
///
/// By default, opens both a local preview server and any tracked remote URLs
/// (e.g., Google Docs, Microsoft 365). Use the `target` argument to open only a
/// specific remote (by service shorthand like "gdoc" or "m365", or by full URL),
/// or use "local" to open only the local preview. Alternatively, use `--no-local`
/// or `--no-remotes` to open only one or the other.
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

    /// The target to open
    ///
    /// Can be a full URL (e.g., https://docs.google.com/document/d/...),
    /// a service shorthand (e.g., "gdoc" or "m365"), or "local" to open
    /// only the local preview server. If omitted, opens all tracked remotes
    /// and the local preview server.
    #[arg(conflicts_with = "no_local", conflicts_with = "no_remotes")]
    target: Option<String>,

    /// Which direction(s) to sync the document
    #[arg(long, default_value = "in")]
    sync: SyncDirection,

    /// Do not open the local preview server
    #[arg(long, conflicts_with = "no_remotes", conflicts_with = "target")]
    no_local: bool,

    /// Do not open tracked remote URLs
    #[arg(long, conflicts_with = "no_local", conflicts_with = "target")]
    no_remotes: bool,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Open a specific document (all remotes + local)</dim>
  <b>stencila open</b> <g>document.md</g>

  <dim># Open current directory (finds index/main/readme)</dim>
  <b>stencila open</b>

  <dim># Open only Google Docs remote</dim>
  <b>stencila open</b> <g>document.md</g> <g>gdoc</g>

  <dim># Open only Microsoft 365 remote</dim>
  <b>stencila open</b> <g>document.md</g> <g>m365</g>

  <dim># Open only local preview server</dim>
  <b>stencila open</b> <g>document.md</g> <g>local</g>

  <dim># Open a specific remote URL</dim>
  <b>stencila open</b> <g>document.md</g> <g>https://docs.google.com/document/d/abc123</g>

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
            target: None,
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
        let path_display = self.path.display();

        // Get remotes for the resolved file (not the original path which might be a directory)
        let remote_infos = get_remotes_for_path(&file, None).await?;

        // Parse the target argument to determine what to open
        let (open_local, remote_to_open) = if let Some(ref target_str) = self.target {
            match target_str.as_str() {
                "local" => {
                    // Special keyword to open only local preview
                    (true, None)
                }
                "gdoc" | "gdocs" => {
                    // Find Google Docs remote
                    let remote = remote_infos
                        .iter()
                        .find(|info| RemoteService::GoogleDocs.matches_url(&info.url))
                        .ok_or_else(|| eyre!("No Google Doc configured for `{path_display}`"))?;
                    (false, Some(remote.url.clone()))
                }
                "m365" => {
                    // Find Microsoft 365 remote
                    let remote = remote_infos
                        .iter()
                        .find(|info| RemoteService::Microsoft365.matches_url(&info.url))
                        .ok_or_else(|| {
                            eyre!("No Microsoft 365 doc configured for `{path_display}`",)
                        })?;
                    (false, Some(remote.url.clone()))
                }
                "site" | "sites" => {
                    // Find Stencila Site
                    let remote = remote_infos
                        .iter()
                        .find(|info| RemoteService::StencilaSites.matches_url(&info.url))
                        .ok_or_else(|| eyre!("No Stencila Site configured for `{path_display}`"))?;
                    (false, Some(remote.url.clone()))
                }
                _ => {
                    // Try to parse as full URL
                    let url = Url::parse(target_str).map_err(|_| {
                        eyre!(
                            "Invalid target or service: `{target_str}`. Use `local`, `site`, `gdoc`, `m365` or a full URL.",
                        )
                    })?;

                    // Validate it's from a supported service
                    let _service = RemoteService::from_url(&url)
                        .ok_or_else(|| eyre!("URL {url} is not from a supported remote service"))?;

                    // Check if this URL is configured for the document
                    if !remote_infos.iter().any(|info| info.url == url) {
                        bail!("URL {url} is not configured for `{path_display}`");
                    }

                    (false, Some(url))
                }
            }
        } else {
            // No target argument - use default behavior (respect no_local/no_remotes flags)
            (true, None)
        };

        // Open remote URLs in browser if specified or not disabled
        if let Some(remote_url) = remote_to_open {
            // Convert to browseable URL for Stencila Sites
            let url_to_open = if RemoteService::StencilaSites.matches_url(&remote_url) {
                stencila_codec_site::browseable_url(&remote_url, Some(&file))
                    .unwrap_or_else(|_| remote_url.clone())
            } else {
                remote_url.clone()
            };

            // Open only the specified remote
            message("🌐 Opening {url_to_open} in browser");
            webbrowser::open(url_to_open.as_str())?;
        } else if self.target.is_none() && !self.no_remotes && !remote_infos.is_empty() {
            // No target specified and remotes not disabled - open all remotes
            message!(
                "🌐 Opening {} configured remote(s) in browser",
                remote_infos.len(),
            );
            for info in &remote_infos {
                // Convert to browseable URL for Stencila Sites
                let remote_url = &info.url;
                let url_to_open = if RemoteService::StencilaSites.matches_url(remote_url) {
                    stencila_codec_site::browseable_url(remote_url, Some(&file))
                        .unwrap_or_else(|_| remote_url.clone())
                } else {
                    remote_url.clone()
                };

                webbrowser::open(url_to_open.as_str())?;
                message("↗️ Opened {url_to_open}");
            }
        }

        // Open local preview server if specified or not disabled
        if open_local && !self.no_local {
            // Serve the parent directory of the file
            let dir = file
                .parent()
                .ok_or_eyre("File has no parent")?
                .to_path_buf();

            // Get (or generate) a server token so it can be included in the URL
            let server_token = get_server_token();

            message("🖥️ Starting local preview server");

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
