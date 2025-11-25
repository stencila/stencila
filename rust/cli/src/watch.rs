use std::{path::PathBuf, str::FromStr};

use clap::Parser;
use eyre::{OptionExt, Result, bail, eyre};

use pathdiff::diff_paths;
use stencila_cli_utils::{color_print::cstr, message};
use stencila_cloud::{WatchRequest, create_watch};
use stencila_codec_utils::{git_info, validate_file_on_default_branch};
use stencila_config::{config_update_remote_watch, config_update_site_watch};
use stencila_dirs::closest_workspace_dir;
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

  <dim># Enable watch with one-way sync from remote to repo</dim>
  <b>stencila watch</> <g>report.md</> <g>gdoc</> <c>--direction from-remote</>

  <dim># Enable watch with ready-for-review PRs</dim>
  <b>stencila watch</> <g>report.md</> <g>gdoc</> <c>--pr-mode ready</>

  <dim># Watch a site directory for one-way sync from repo to site</dim>
  <b>stencila watch</> <g>site/</>

  <dim># Note: The document must already be pushed to a remote</dim>
  <b>stencila push</> <g>report.md</> <c>--to</> <g>gdoc</>
  <b>stencila watch</> <g>report.md</>
"
);

impl Cli {
    pub async fn run(self) -> Result<()> {
        let path_display = self.path.display();

        // Validate path exists
        if !self.path.exists() {
            bail!("Path `{path_display}` does not exist");
        }

        // Check if this is a directory
        if self.path.is_dir() {
            // Get workspace directory and config
            let workspace_dir = closest_workspace_dir(&self.path, false).await?;
            let config = stencila_config::config(&workspace_dir)?;

            // Check if this path is exactly the site root (not a subdirectory)
            if config.path_is_site_root(&self.path, &workspace_dir) {
                return self.watch_site(&config).await;
            }

            // Provide helpful error message based on whether site.root is configured
            if let Some(site_config) = &config.site
                && site_config.root.is_some()
            {
                // Site root is configured but this path doesn't match it exactly
                // (could be a subdirectory or a completely different directory)
                bail!(
                    "Only the site root directory can be watched. `{path_display}` is not the configured site root."
                );
            } else {
                let workspace_relative_path =
                    diff_paths(&self.path, &workspace_dir).unwrap_or_else(|| self.path.clone());
                let workspace_relative_path = workspace_relative_path.display();
                bail!(
                    "Directory `{path_display}` is not the workspace's site root. Add `root = \"{workspace_relative_path}\"` to the `[site]` section in `stencila.toml`."
                );
            }
        }

        // Validate file exists on the default branch (also validates it's in a git repo)
        validate_file_on_default_branch(&self.path)?;

        // Get git repository information
        let git_info = git_info(&self.path)?;
        let repo_url = git_info
            .origin
            .ok_or_eyre("Repository has no origin remote")?;

        let no_remotes = || {
            message!(
                "⚠️ File `{path_display}` has no remotes to watch yet. Please push the document to a remote first e.g. *stencila push {path_display} --to gdoc --watch*"
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
            message!("👁️ File `{path_display}` is already being watched on `{service_name}`.");
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

        message!(
            "👁️ Watching `{path_display}` ({direction_desc}). PRs will be opened/updated on changes from {remote_url_str}."
        );

        Ok(())
    }

    /// Watch a site directory
    ///
    /// This enables unidirectional sync from the repository to a Stencila Site.
    /// When changes are pushed to the repository, the site is automatically updated.
    ///
    /// Note: This function assumes the caller has already verified that the path
    /// is within a configured site root.
    async fn watch_site(&self, config: &stencila_config::Config) -> Result<()> {
        use stencila_cloud::sites::ensure_site;

        let path_display = self.path.display();

        // Ensure site exists (creates if necessary)
        let (site_id, _) = ensure_site(&self.path).await?;

        // Check if site is already being watched
        if let Some(site_config) = &config.site
            && site_config.watch.is_some()
        {
            message!("👁️ Site `{path_display}` is already being watched for changes.");
            return Ok(());
        }

        // Get git repository information
        let git_info = git_info(&self.path)?;
        let repo_url = git_info
            .origin
            .ok_or_eyre("Repository has no origin remote (required to setup a watch)")?;

        // Get directory path relative to repo root (must end with / for API)
        let dir_path = git_info.path.unwrap_or_else(|| {
            self.path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(".")
                .to_string()
        });
        let dir_path = if dir_path.ends_with('/') {
            dir_path
        } else {
            format!("{dir_path}/")
        };

        // Build the site URL
        let site_url = format!("https://{site_id}.stencila.site");

        // Site watches are always to-remote (unidirectional)
        let direction = self.direction.unwrap_or(WatchDirection::ToRemote);
        if direction != WatchDirection::ToRemote {
            message!(
                "⚠️ Site watches only support `to-remote` direction (sites are write-only). Using `to-remote`."
            );
        }

        // Create watch request
        let request = WatchRequest {
            remote_url: site_url.clone(),
            repo_url,
            file_path: dir_path,
            direction: Some(WatchDirection::ToRemote.to_string()),
            pr_mode: self.pr_mode.map(|mode| mode.to_string()),
            debounce_seconds: self.debounce_seconds,
        };

        // Call Cloud API to create watch
        let response = create_watch(request).await?;

        // Update stencila.toml with watch ID under [site]
        config_update_site_watch(&self.path, Some(response.id.to_string()))?;

        message!(
            "👁️ Watching site directory. Site will be updated on pushes to the repository within `{path_display}`."
        );

        Ok(())
    }
}
