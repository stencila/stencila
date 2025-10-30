use std::path::PathBuf;

use clap::Parser;
use eyre::{Result, bail};

use stencila_cli_utils::{color_print::cstr, message};
use stencila_cloud::{WatchRequest, create_watch};
use stencila_codec_utils::git_info;
use stencila_codecs::remotes::RemoteService;
use stencila_document::{Document, WatchDirection, WatchPrMode};
use url::Url;

/// Enable automatic sync between a document and its remote
///
/// Creates a watch in Stencila Cloud that automatically syncs changes
/// between a remote (Google Docs or M365) and a GitHub repository.
/// When changes are detected in the remote, a pull request will be
/// created or updated in the repository.
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// The path to the document to watch
    input: PathBuf,

    /// The remote URL to watch
    ///
    /// If the document has multiple remotes (e.g., both Google Docs and M365),
    /// you must specify which one to watch. Can be the full URL or a service
    /// shorthand: "gdoc" or "m365".
    url: Option<String>,

    /// The sync direction
    #[arg(long, short)]
    direction: Option<WatchDirection>,

    /// The GitHub PR mode
    #[arg(long, short)]
    pr_mode: Option<WatchPrMode>,

    /// Debounce time in seconds (10-86400)
    ///
    /// Time to wait after detecting changes before syncing to avoid
    /// too frequent updates. Minimum 10 seconds, maximum 24 hours (86400 seconds).
    #[arg(long, value_parser = clap::value_parser!(u64).range(10..=86400))]
    debounce_seconds: Option<u64>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Enable watch on the tracked remote</dim>
  <b>stencila watch</> <g>report.md</>

  <dim># Watch a specific remote (if document has multiple)</dim>
  <b>stencila watch</> <g>report.md</> <g>gdoc</>
  <b>stencila watch</> <g>report.md</> <g>https://docs.google.com/document/d/abc123</>

  <dim># Enable watch with one-way sync from remote</dim>
  <b>stencila watch</> <g>report.md</> <g>gdoc</> <c>--direction from-remote</>

  <dim># Enable watch with ready-for-review PRs</dim>
  <b>stencila watch</> <g>report.md</> <g>gdoc</> <c>--pr-mode ready</>

  <dim># Note: The document must already be pushed to a remote</dim>
  <b>stencila push</> <g>report.md</> <g>gdoc</>
  <b>stencila watch</> <g>report.md</>
"
);

impl Cli {
    pub async fn run(self) -> Result<()> {
        let input = self.input.display();

        // Validate input file exists
        if !self.input.exists() {
            bail!("Input file `{input}` does not exist");
        }

        // Get git repository information
        let git_info = git_info(&self.input)?;
        let Some(repo_url) = git_info.origin else {
            bail!("File is not in a git repository. Please initialize a git repository first.");
        };

        // Open the document and get tracking information
        let doc = Document::open(&self.input, None).await?;
        let Some((.., Some(tracking))) = doc.tracking().await? else {
            std::process::exit(3); // Exit code 3: missing remote linkage
        };

        // Get tracked remotes
        let Some(remotes) = tracking.remotes else {
            bail!(
                "No remote linkage found for `{input}`.\nPlease push the document to a remote first:\n  stencila push {input} gdoc"
            );
        };

        if remotes.is_empty() {
            bail!(
                "No remote linkage found for `{input}`.\nPlease push the document to a remote first:\n  stencila push {input} gdoc"
            );
        }

        // Determine which remote to watch based on URL argument or tracked remotes
        let (remote_url, mut remote_info) = if let Some(url_str) = self.url {
            // Parse URL or service shorthand
            let target_url = match url_str.as_str() {
                "gdoc" | "gdocs" => {
                    // Find the Google Docs remote
                    remotes
                        .iter()
                        .find(|(url, _)| RemoteService::GoogleDocs.matches_url(url))
                        .ok_or_else(|| eyre::eyre!("No Google Docs remote found for `{input}`"))?
                        .0
                        .clone()
                }
                "m365" => {
                    // Find the M365 remote
                    remotes
                        .iter()
                        .find(|(url, _)| RemoteService::Microsoft365.matches_url(url))
                        .ok_or_else(|| eyre::eyre!("No Microsoft 365 remote found for `{input}`"))?
                        .0
                        .clone()
                }
                _ => {
                    // Try to parse as URL
                    Url::parse(&url_str).map_err(|_| {
                        eyre::eyre!(
                            "Invalid URL or service: '{}'. Use 'gdoc', 'm365', or a full URL.",
                            url_str
                        )
                    })?
                }
            };

            // Find the remote in the tracked remotes
            remotes
                .into_iter()
                .find(|(url, _)| url == &target_url)
                .ok_or_else(|| {
                    eyre::eyre!("Remote URL not found in tracked remotes: {}", target_url)
                })?
        } else {
            // No URL specified - check if there's only one remote
            if remotes.len() > 1 {
                let remotes_list = remotes
                    .keys()
                    .map(|url| {
                        let service = RemoteService::from_url(url)
                            .map(|s| s.cli_name().to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        format!("  - {}: {}", service, url)
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                bail!(
                    "Multiple remotes found for `{input}`:\n{remotes_list}\n\nSpecify which one to watch using a service (gdoc/m365) or full URL."
                );
            }

            // Get the single remote
            remotes
                .into_iter()
                .next()
                .ok_or_else(|| eyre::eyre!("No remote found (this should not happen)"))?
        };

        // Check if already being watched
        if remote_info.is_watched() {
            message(&format!("File `{input}` is already being watched."), None);
            return Ok(());
        }

        // Get file path relative to repo root
        let file_path = git_info.path.unwrap_or_else(|| {
            self.input
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string()
        });

        // Create watch request
        let request = WatchRequest {
            remote_url: remote_url.to_string(),
            repo_url,
            file_path,
            direction: self.direction.map(|dir| dir.to_string()),
            pr_mode: self.pr_mode.map(|mode| mode.to_string()),
            debounce_seconds: self.debounce_seconds,
        };

        // Call Cloud API to create watch
        let response = create_watch(request).await?;

        // Update docs.json with watch metadata
        remote_info.watch_id = Some(response.id.to_string());
        remote_info.watch_direction = self.direction;
        doc.track(Some((remote_url, remote_info))).await?;

        // Success message
        let direction_desc = match self.direction.unwrap_or_default() {
            WatchDirection::Bi => "bi-directional",
            WatchDirection::FromRemote => "from remote only",
            WatchDirection::ToRemote => "to remote only",
        };

        message(
            &format!(
                "Watching `{input}` ({direction_desc}). PRs will be opened/updated on changes from the remote."
            ),
            None,
        );

        Ok(())
    }
}
