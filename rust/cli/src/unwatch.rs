use std::path::PathBuf;

use clap::Parser;
use eyre::{Result, bail, eyre};

use stencila_cli_utils::{color_print::cstr, message};
use stencila_cloud::delete_watch;
use stencila_codecs::remotes::RemoteService;
use stencila_document::Document;
use url::Url;

/// Disable automatic sync for a document
///
/// Removes the watch from Stencila Cloud, stopping automatic sync.
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// The path to the document to unwatch
    path: PathBuf,

    /// The target remote to unwatch
    ///
    /// If the document has multiple watched remotes, you must specify which one
    /// to unwatch. Can be the full URL or a service shorthand: "gdoc" or
    /// "m365".
    target: Option<String>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
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
        let path_display = self.path.display();

        // Validate file exists
        if !self.path.exists() {
            bail!("File `{path_display}` does not exist");
        }

        let not_watched = || {
            message(&format!("File `{path_display}` is not begin watched."), Some("ℹ️"));
            Ok(())
        };

        // Open the document and get tracking information
        let doc = Document::open(&self.path, None).await?;
        let Some((.., Some(tracking))) = doc.tracking().await? else {
            return not_watched();
        };

        // Get tracked remotes
        let Some(remotes) = tracking.remotes else {
            return not_watched();
        };
        if remotes.is_empty() {
            return not_watched();
        }

        // Determine which remote to unwatch based on target argument
        let (remote_url, mut remote_info) = if let Some(target_str) = self.target {
            // Parse target or service shorthand
            let target_url = match target_str.as_str() {
                "gdoc" | "gdocs" => {
                    // Find the Google Docs remote
                    remotes
                        .iter()
                        .find(|(url, _)| RemoteService::GoogleDocs.matches_url(url))
                        .ok_or_else(|| eyre!("No Google Docs remote found for `{path_display}`"))?
                        .0
                        .clone()
                }
                "m365" => {
                    // Find the M365 remote
                    remotes
                        .iter()
                        .find(|(url, _)| RemoteService::Microsoft365.matches_url(url))
                        .ok_or_else(|| eyre!("No Microsoft 365 remote found for `{path_display}`"))?
                        .0
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

            // Find the remote in the tracked remotes
            remotes
                .into_iter()
                .find(|(url, _)| url == &target_url)
                .ok_or_else(|| {
                    eyre!("Remote target not found in tracked remotes: {}", target_url)
                })?
        } else {
            // No target specified - check if there's only one watched remote
            let watched_remotes: Vec<_> = remotes
                .iter()
                .filter(|(_, info)| info.is_watched())
                .collect();

            if watched_remotes.is_empty() {
                return  not_watched();
            }

            if watched_remotes.len() > 1 {
                let urls_list = watched_remotes
                    .iter()
                    .map(|(url, _)| {
                        let service = RemoteService::from_url(url)
                            .map(|s| s.cli_name().to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        format!("  - {}: {}", service, url)
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                bail!(
                    "Multiple watched remotes found for `{path_display}`:\n{urls_list}\n\nSpecify which one to unwatch using a service (gdoc/m365) or full URL."
                );
            }

            // Get the single watched remote
            let (url, info) = watched_remotes[0];
            (url.clone(), info.clone())
        };

        // Check if remote is actually being watched
        if !remote_info.is_watched() {
            bail!("Remote `{}` is not being watched.", remote_url);
        }

        // Call Cloud API to delete watch
        let watch_id = remote_info
            .watch_id
            .as_ref()
            .ok_or_else(|| eyre!("No watch ID found"))?;
        delete_watch(watch_id).await?;

        // Clear watch metadata (but preserve other tracking info)
        remote_info.watch_id = None;
        remote_info.watch_direction = None;

        // Update docs.json
        doc.track(Some((remote_url, remote_info))).await?;

        // Success message
        message(
            &format!(
                "Stopped watching `{path_display}` (link to remote remains, see `stencila status`)"
            ),
            None,
        );

        Ok(())
    }
}
