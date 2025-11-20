use std::path::PathBuf;

use clap::Parser;
use eyre::{Result, bail, eyre};

use stencila_cli_utils::{color_print::cstr, message};
use stencila_cloud::{WatchRequest, create_watch};
use stencila_codec_utils::{git_info, validate_file_on_default_branch};
use stencila_config::config_update_remote_watch;
use stencila_remotes::{RemoteService, WatchDirection, WatchPrMode, get_remotes_for_path};
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

        let no_remotes = || {
            message(
                &format!(
                    "File `{path_display}` has no remotes to watch yet. Please push the document to a remote first e.g. *stencila push {path_display} gdoc*"
                ),
                Some("⚠️ "),
            );
            Ok(())
        };

        // Get remotes from config
        let remote_infos = get_remotes_for_path(&self.path, None).await?;
        if remote_infos.is_empty() {
            return no_remotes();
        }

        // Determine which remote to watch based on target argument or configured remotes
        let remote_info = if let Some(target_str) = self.target {
            // Parse target or service shorthand
            let target_url = match target_str.as_str() {
                "gdoc" | "gdocs" => {
                    // Find the Google Docs remote
                    remote_infos
                        .iter()
                        .find(|info| RemoteService::GoogleDocs.matches_url(&info.url))
                        .ok_or_else(|| eyre!("No Google Doc found for `{path_display}`"))?
                        .url
                        .clone()
                }
                "m365" => {
                    // Find the M365 remote
                    remote_infos
                        .iter()
                        .find(|info| RemoteService::Microsoft365.matches_url(&info.url))
                        .ok_or_else(|| {
                            eyre!("No Microsoft 365 document found for `{path_display}`")
                        })?
                        .url
                        .clone()
                }
                _ => {
                    // Try to parse as URL
                    Url::parse(&target_str).map_err(|_| {
                        eyre!(
                            "Invalid target or service: '{}'. Use 'gdoc', 'm365', or a full URL.",
                            target_str
                        )
                    })?
                }
            };

            // Find the remote in the configured remotes
            remote_infos
                .into_iter()
                .find(|info| info.url == target_url)
                .ok_or_else(|| {
                    eyre!(
                        "Remote target not found in configured remotes: {}",
                        target_url
                    )
                })?
        } else {
            // No target specified - check if there's only one remote
            if remote_infos.len() > 1 {
                let remotes_list = remote_infos
                    .iter()
                    .map(|info| {
                        let service = RemoteService::from_url(&info.url)
                            .map(|s| s.cli_name().to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        format!("  - {}: {}", service, info.url)
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                bail!(
                    "Multiple remotes found for `{path_display}`:\n{remotes_list}\n\nSpecify which one to watch using a service (gdoc/m365) or full URL."
                );
            }

            // Get the single remote
            remote_infos
                .into_iter()
                .next()
                .ok_or_else(|| eyre!("No remote found (this should not happen)"))?
        };

        // Check if already being watched
        if remote_info.config.watch.is_some() {
            let service_name = RemoteService::from_url(&remote_info.url)
                .map(|s| s.display_name_plural().to_string())
                .unwrap_or_else(|| remote_info.url.to_string());
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
            remote_url: remote_info.url.to_string(),
            repo_url,
            file_path,
            direction: self.direction.map(|dir| dir.to_string()),
            pr_mode: self.pr_mode.map(|mode| mode.to_string()),
            debounce_seconds: self.debounce_seconds,
        };

        // Call Cloud API to create watch
        let response = create_watch(request).await?;

        // Update stencila.toml with watch ID
        config_update_remote_watch(
            &self.path,
            remote_info.url.as_ref(),
            Some(response.id.to_string()),
        )?;

        let remote_url_str = remote_info.url.to_string();

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
