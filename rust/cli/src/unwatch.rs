use std::{path::PathBuf, str::FromStr};

use clap::Parser;
use eyre::{Result, bail, eyre};

use stencila_cli_utils::{color_print::cstr, message};
use stencila_cloud::{delete_watch, ensure_workspace};
use stencila_config::{ConfigTarget, config_unset, config_update_remote_watch};
use stencila_remotes::{RemoteService, get_remotes_for_path};
use url::Url;

/// Disable automatic sync for the workspace or a document
///
/// When run without a path, disables workspace-level watching that runs
/// `update.sh` on each git push.
///
/// When run with a path, removes the watch from Stencila Cloud for that
/// document, stopping automatic sync with its remote.
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// The path to the document to unwatch (optional)
    ///
    /// If omitted, disables workspace-level watching.
    path: Option<PathBuf>,

    /// The target remote to unwatch (only used with a file path)
    ///
    /// If the document has multiple watched remotes, you must specify which one
    /// to unwatch. Can be the full URL or a service shorthand: "gdoc" or
    /// "m365".
    target: Option<String>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Disable workspace watch</dim>
  <b>stencila unwatch</>

  <dim># Disable watch for a document</dim>
  <b>stencila unwatch</> <g>report.md</>

  <dim># Unwatch a specific remote (if document has multiple)</dim>
  <b>stencila unwatch</> <g>report.md</> <g>gdoc</>

  <dim># Note: Remote linkage is preserved, you can re-enable watch later</dim>
  <b>stencila unwatch</> <g>report.md</>
  <b>stencila watch</> <g>report.md</>
"
);

impl Cli {
    pub async fn run(self) -> Result<()> {
        // If no path provided, disable workspace-level watch
        let Some(ref path) = self.path else {
            return self.run_workspace_unwatch().await;
        };

        let path_display = path.display();

        // Validate file exists
        if !path.exists() {
            bail!("File `{path_display}` does not exist");
        }

        let not_watched = || {
            message!("ℹ️ File `{}` is not being watched.", path_display);
            Ok(())
        };

        // Get remotes from config
        let remote_infos = get_remotes_for_path(path, None).await?;
        if remote_infos.is_empty() {
            return not_watched();
        }

        // Determine which remote to unwatch based on target argument
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
                            "Invalid target or service: `{target_str}`. Use `gdoc`, `m365`, or a full URL.",
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
            // No target specified - check if there's only one watched remote
            let watched_remotes: Vec<_> = remote_infos
                .iter()
                .filter(|info| info.watch_id.is_some())
                .collect();

            if watched_remotes.is_empty() {
                return not_watched();
            }

            if watched_remotes.len() > 1 {
                let urls_list = watched_remotes
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
                    "Multiple watched remotes found for `{path_display}`:\n{urls_list}\n\nSpecify which one to unwatch using a service (gdoc/m365) or full URL."
                );
            }

            // Get the single watched remote
            watched_remotes[0].clone()
        };

        // Check if remote is actually being watched
        if remote_info.watch_id.is_none() {
            bail!("Remote {} is not being watched.", remote_info.url);
        }

        // Get workspace ID (required for delete_watch)
        let (workspace_id, _) = ensure_workspace(path).await?;

        // Call Cloud API to delete watch
        let watch_id = remote_info
            .watch_id
            .as_ref()
            .ok_or_else(|| eyre!("No watch ID found"))?;
        delete_watch(&workspace_id, watch_id).await?;

        // Remove watch ID from stencila.toml
        config_update_remote_watch(path, remote_info.url.as_ref(), None)?;

        // Success message
        message!(
            "👁️ Stopped watching `{}` (link to remote remains, see *stencila status*)",
            path_display
        );

        Ok(())
    }

    /// Disable workspace-level watch
    ///
    /// This removes the workspace watch that runs `update.sh` on each git push.
    async fn run_workspace_unwatch(&self) -> Result<()> {
        let cwd = std::env::current_dir()?;

        // Check if workspace is being watched
        let cfg = stencila_config::config(&cwd)?;
        let Some(watch_id) = cfg.workspace.as_ref().and_then(|w| w.watch.clone()) else {
            message!("ℹ️ Workspace is not being watched");
            return Ok(());
        };

        // Get workspace ID (required for delete_workspace_watch)
        let (workspace_id, _) = ensure_workspace(&cwd).await?;

        // Call Cloud API to delete workspace watch
        delete_watch(&workspace_id, &watch_id).await?;

        // Remove watch ID from stencila.toml
        config_unset("workspace.watch", ConfigTarget::Nearest)?;

        message!("👁️ Workspace watch disabled");

        Ok(())
    }
}
