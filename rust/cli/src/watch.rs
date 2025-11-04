use std::path::PathBuf;

use clap::Parser;
use eyre::{Result, bail};

use stencila_cli_utils::{color_print::cstr, message};
use stencila_cloud::{WatchRequest, create_watch};
use stencila_codec_utils::{git_info, validate_file_on_default_branch};
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
    path: PathBuf,

    /// The target remote to watch
    ///
    /// If the document has multiple remotes (e.g., both Google Docs and M365),
    /// you must specify which one to watch. Can be the full URL or a service
    /// shorthand: "gdoc" or "m365".
    target: Option<String>,

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
        let path_display = self.path.display();

        // Validate file exists
        if !self.path.exists() {
            bail!("File `{path_display}` does not exist");
        }

        // Validate file exists on the default branch (also validates it's in a git repo)
        validate_file_on_default_branch(&self.path)?;

        // Get git repository information
        let git_info = git_info(&self.path)?;
        let Some(repo_url) = git_info.origin else {
            bail!("File is not in a git repository. Please initialize a git repository first.");
        };

        // Open the document and get tracking information
        let doc = Document::open(&self.path, None).await?;
        let Some((.., Some(tracking))) = doc.tracking().await? else {
            std::process::exit(3); // Exit code 3: missing remote linkage
        };

        // Get tracked remotes
        let Some(remotes) = tracking.remotes else {
            bail!(
                "No remote linkage found for `{path_display}`.\nPlease push the document to a remote first:\n  stencila push {path_display} gdoc"
            );
        };

        if remotes.is_empty() {
            bail!(
                "No remote linkage found for `{path_display}`.\nPlease push the document to a remote first:\n  stencila push {path_display} gdoc"
            );
        }

        // Determine which remote to watch based on target argument or tracked remotes
        let (remote_url, mut remote_info) = if let Some(target_str) = self.target {
            // Parse target or service shorthand
            let target_url = match target_str.as_str() {
                "gdoc" | "gdocs" => {
                    // Find the Google Docs remote
                    remotes
                        .iter()
                        .find(|(url, _)| RemoteService::GoogleDocs.matches_url(url))
                        .ok_or_else(|| {
                            eyre::eyre!("No Google Docs remote found for `{path_display}`")
                        })?
                        .0
                        .clone()
                }
                "m365" => {
                    // Find the M365 remote
                    remotes
                        .iter()
                        .find(|(url, _)| RemoteService::Microsoft365.matches_url(url))
                        .ok_or_else(|| {
                            eyre::eyre!("No Microsoft 365 remote found for `{path_display}`")
                        })?
                        .0
                        .clone()
                }
                _ => {
                    // Try to parse as URL
                    Url::parse(&target_str).map_err(|_| {
                        eyre::eyre!(
                            "Invalid target or service: '{}'. Use 'gdoc', 'm365', or a full URL.",
                            target_str
                        )
                    })?
                }
            };

            // Find the remote in the tracked remotes
            remotes
                .into_iter()
                .find(|(url, _)| url == &target_url)
                .ok_or_else(|| {
                    eyre::eyre!("Remote target not found in tracked remotes: {}", target_url)
                })?
        } else {
            // No target specified - check if there's only one remote
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
                    "Multiple remotes found for `{path_display}`:\n{remotes_list}\n\nSpecify which one to watch using a service (gdoc/m365) or full URL."
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
            let service_name = RemoteService::from_url(&remote_url)
                .map(|s| s.display_name_plural().to_string())
                .unwrap_or_else(|| remote_url.to_string());
            message(
                &format!("File `{path_display}` is already being watched on `{service_name}`."),
                Some("👁️ "),
            );
            return Ok(());
        }

        // Get file path relative to repo root
        let file_path = git_info.path.unwrap_or_else(|| {
            self.path
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

        let remote_url_str = remote_url.to_string();
        doc.track(Some((remote_url, remote_info))).await?;

        // Success message
        let direction_desc = match self.direction.unwrap_or_default() {
            WatchDirection::Bi => "bi-directional",
            WatchDirection::FromRemote => "from remote only",
            WatchDirection::ToRemote => "to remote only",
        };

        message(
            &format!(
                "Watching `{path_display}` ({direction_desc}). PRs will be opened/updated on changes from {remote_url_str}."
            ),
            Some("👁️ "),
        );

        Ok(())
    }
}
