use std::{path::PathBuf, str::FromStr};

use clap::Parser;
use eyre::{Result, bail, eyre};

use stencila_cli_utils::{color_print::cstr, message};
use stencila_cloud::{WatchRequest, create_watch, create_workspace_watch, ensure_workspace};
use stencila_codec_utils::{git_file_info, validate_file_on_default_branch};
use stencila_config::{ConfigTarget, config_update_remote_watch, set_value};
use stencila_remotes::{RemoteService, WatchDirection, WatchPrMode, get_remotes_for_path};
use url::Url;

/// Enable automatic sync for the workspace or a document
///
/// When run without a path, enables workspace-level watching that runs
/// `update.sh` on each git push (for automatic site/outputs publishing).
///
/// When run with a path, creates a watch in Stencila Cloud that automatically
/// syncs changes between a remote (Google Docs or M365) and a GitHub repository.
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// The path to the document to watch (optional)
    ///
    /// If omitted, enables workspace-level watching that runs `update.sh`
    /// on each git push for automatic site and outputs publishing.
    path: Option<PathBuf>,

    /// The target remote to watch (only used with a file path)
    ///
    /// If the document has multiple remotes (e.g., both Google Docs and M365),
    /// you must specify which one to watch. Can be the full URL or a service
    /// shorthand: "gdoc" or "m365".
    target: Option<String>,

    /// The sync direction (only used with a file path)
    #[arg(long, short)]
    direction: Option<WatchDirection>,

    /// The GitHub PR mode (only used with a file path)
    #[arg(long, short)]
    pr_mode: Option<WatchPrMode>,

    /// Debounce time in seconds (10-86400, only used with a file path)
    ///
    /// Time to wait after detecting changes before syncing to avoid
    /// too frequent updates. Minimum 10 seconds, maximum 24 hours (86400 seconds).
    #[arg(long, value_parser = clap::value_parser!(u64).range(10..=86400))]
    debounce_seconds: Option<u64>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Enable workspace watch (runs update.sh on each push)</dim>
  <b>stencila watch</>

  <dim># Enable watch on the tracked remote for a file</dim>
  <b>stencila watch</> <g>report.md</>

  <dim># Watch a specific remote (if document has multiple)</dim>
  <b>stencila watch</> <g>report.md</> <g>gdoc</>
  <b>stencila watch</> <g>report.md</> <g>https://docs.google.com/document/d/abc123</>

  <dim># Enable watch with one-way sync from remote to repo</dim>
  <b>stencila watch</> <g>report.md</> <g>gdoc</> <c>--direction from-remote</>

  <dim># Enable watch with ready-for-review PRs</dim>
  <b>stencila watch</> <g>report.md</> <g>gdoc</> <c>--pr-mode ready</>

  <dim># Note: The document must already be pushed to a remote</dim>
  <b>stencila push</> <g>report.md</> <c>--to</> <g>gdoc</>
  <b>stencila watch</> <g>report.md</>
"
);

impl Cli {
    pub async fn run(self) -> Result<()> {
        // If no path provided, enable workspace-level watch
        let Some(ref path) = self.path else {
            return self.run_workspace_watch().await;
        };

        let path_display = path.display();

        // Validate path exists
        if !path.exists() {
            bail!("Path `{path_display}` does not exist");
        }

        // Check if this is a directory
        if path.is_dir() {
            bail!(
                "Watches are not supported for directories. Use `stencila site push` to publish a directory to a Stencila Site."
            );
        }

        // Validate file exists on the default branch (also validates it's in a git repo)
        validate_file_on_default_branch(path)?;

        // Ensure workspace exists (required for creating watches)
        let (workspace_id, _) = ensure_workspace(path).await?;

        // Get git repository information
        let git_file_info = git_file_info(path)?;

        let no_remotes = || {
            message!(
                "⚠️ File `{}` has no remotes to watch yet. Please push the document to a remote first e.g. *stencila push {} --to gdoc --watch*",
                path_display,
                path_display
            );
            Ok(())
        };

        // Get remotes from config
        let remote_infos = get_remotes_for_path(path, None).await?;
        if remote_infos.is_empty() {
            return no_remotes();
        }

        // Determine which remote to watch based on target argument or configured remotes
        let remote_info = if let Some(target_str) = self.target {
            // Parse target or service shorthand
            let target_url = match RemoteService::from_str(&target_str) {
                Ok(RemoteService::GoogleDocs) => {
                    // Find the Google Docs remote
                    remote_infos
                        .iter()
                        .find(|info| RemoteService::GoogleDocs.matches_url(&info.url))
                        .ok_or_else(|| eyre!("No Google Doc found for `{path_display}`"))?
                        .url
                        .clone()
                }
                Ok(RemoteService::Microsoft365) => {
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
                            "Invalid target or service: `{target_str}`. Use `gdoc`, `m365`, or a full URL."
                        )
                    })?
                }
            };

            // Find the remote in the configured remotes
            remote_infos
                .into_iter()
                .find(|info| info.url == target_url)
                .ok_or_else(|| {
                    eyre!("Remote target not found in configured remotes: {target_url}")
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
        if remote_info.watch_id.is_some() {
            let service_name = RemoteService::from_url(&remote_info.url)
                .map(|s| s.display_name_plural().to_string())
                .unwrap_or_else(|| remote_info.url.to_string());
            message!(
                "👁️ File `{}` is already being watched on `{}`.",
                path_display,
                service_name
            );
            return Ok(());
        }

        // Get file path relative to repo root
        let file_path = git_file_info.path.unwrap_or_else(|| {
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string()
        });

        // Create watch request
        let request = WatchRequest {
            remote_url: remote_info.url.to_string(),
            file_path,
            direction: self.direction.map(|dir| dir.to_string()),
            pr_mode: self.pr_mode.map(|mode| mode.to_string()),
            debounce_seconds: self.debounce_seconds,
        };

        // Call Cloud API to create watch
        let response = create_watch(&workspace_id, request).await?;

        // Update stencila.toml with watch ID
        config_update_remote_watch(
            path,
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

        message!(
            "👁️ Watching `{}` ({}). PRs will be opened/updated on changes from {}.",
            path_display,
            direction_desc,
            remote_url_str
        );

        Ok(())
    }

    /// Enable workspace-level watch
    ///
    /// This creates a workspace watch that runs `update.sh` on each git push.
    async fn run_workspace_watch(&self) -> Result<()> {
        let cwd = std::env::current_dir()?;

        // Ensure workspace exists (required for creating watches)
        let (workspace_id, _) = ensure_workspace(&cwd).await?;

        // Check if already watching
        let cfg = stencila_config::get()?;
        if let Some(workspace) = &cfg.workspace
            && workspace.watch.is_some()
        {
            message!("👁️ Workspace is already being watched. Use `stencila unwatch` to disable.");
            return Ok(());
        }

        // Create workspace watch
        let response = create_workspace_watch(&workspace_id).await?;

        // Update stencila.toml with watch ID
        set_value("workspace.watch", &response.id, ConfigTarget::Nearest)?;

        message!("👁️ Workspace watch enabled.");

        Ok(())
    }
}
