use std::path::PathBuf;

use clap::Parser;
use eyre::{Result, bail, eyre};
use url::Url;

use stencila_cli_utils::{color_print::cstr, message};
use stencila_codecs::{DecodeOptions, EncodeOptions, remotes::RemoteService};
use stencila_document::Document;

/// Pull a document from a remote service
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// The path to the local document
    path: PathBuf,

    /// The target to pull from
    ///
    /// Can be a full URL (e.g., https://docs.google.com/document/d/...) or a
    /// service shorthand (e.g "gdoc" or "m365"). Omit to use the tracked
    /// remote (errors if multiple remotes are tracked).
    target: Option<String>,

    /// Do not merge, just replace
    ///
    /// By default, the pulled document will be merged with the local version.
    /// Use this flag to skip merging and just replace the local file.
    #[arg(long)]
    no_merge: bool,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Pull from the tracked remote (if only one exists)</dim>
  <b>stencila pull</> <g>document.smd</>

  <dim># Pull from tracked Google Doc</dim>
  <b>stencila pull</> <g>document.smd</> <g>gdoc</>

  <dim># Pull from tracked Microsoft 365 document</dim>
  <b>stencila pull</> <g>document.smd</> <g>m365</>

  <dim># Pull from specific URL</dim>
  <b>stencila pull</> <g>document.smd</> <g>https://docs.google.com/document/d/abc123</>

  <dim># Pull without merging (replace local file)</dim>
  <b>stencila pull</> <g>document.smd</> <g>gdoc</> <c>--no-merge</>
"
);

impl Cli {
    pub async fn run(self) -> Result<()> {
        let path_display = self.path.display();

        // Validate input file exists
        if !self.path.exists() {
            bail!("File `{path_display}` does not exist");
        }

        // Open the document
        let doc = Document::open(&self.path, None).await?;

        // Determine the target to pull from
        let (service, url) = if let Some(target_str) = &self.target {
            // Target or service shorthand specified
            match target_str.as_str() {
                "gdoc" | "gdocs" => {
                    // Find tracked Google Docs remote
                    let remotes = doc.remotes().await?;
                    let url = remotes
                        .iter()
                        .find(|u| RemoteService::GoogleDocs.matches_url(u))
                        .ok_or_else(|| {
                            eyre!(
                                "No tracked Google Doc found for `{path_display}`. Use a full URL to specify one."
                            )
                        })?
                        .clone();
                    (RemoteService::GoogleDocs, url)
                }
                "m365" => {
                    // Find tracked Microsoft 365 remote
                    let remotes = doc.remotes().await?;
                    let url = remotes
                        .iter()
                        .find(|u| RemoteService::Microsoft365.matches_url(u))
                        .ok_or_else(|| {
                            eyre!(
                                "No tracked Microsoft 365 document found for `{path_display}`. Use a full URL to specify one."
                            )
                        })?
                        .clone();
                    (RemoteService::Microsoft365, url)
                }
                "site" | "sites" => {
                    // Find tracked Stencila Site remote
                    let remotes = doc.remotes().await?;
                    let url = remotes
                        .iter()
                        .find(|u| RemoteService::StencilaSites.matches_url(u))
                        .ok_or_else(|| {
                            eyre!(
                                "No tracked Stencila Site found for `{path_display}`. Use a full URL to specify one."
                            )
                        })?
                        .clone();
                    (RemoteService::StencilaSites, url)
                }
                _ => {
                    // Assume it's a URL
                    let url = Url::parse(target_str)
                        .map_err(|_| eyre!("Invalid target: {target_str}"))?;
                    let service = RemoteService::from_url(&url)
                        .ok_or_else(|| eyre!("URL {url} is not from a supported remote service"))?;
                    (service, url)
                }
            }
        } else {
            // No target or service specified, find any tracked remote
            let remotes = doc.remotes().await?;
            if remotes.is_empty() {
                bail!(
                    "No tracked remotes for `{path_display}`. Specify a target or service to pull from.",
                );
            }

            // Error if multiple remotes are tracked
            if remotes.len() > 1 {
                let remotes_list = remotes
                    .iter()
                    .map(|url| {
                        let service = RemoteService::from_url(url)
                            .map(|s| s.cli_name().to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        format!("  - {}: {}", service, url)
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                bail!(
                    "Multiple remotes found for `{path_display}`:\n{remotes_list}\n\nSpecify which one to pull using a service (gdoc/m365) or full URL."
                );
            }

            // Find which service the tracked remote belongs to
            let remote_url = &remotes[0];
            let service = RemoteService::from_url(remote_url).ok_or_else(|| {
                eyre!("Tracked remote {remote_url} is not from a supported service",)
            })?;

            (service, remote_url.clone())
        };

        message(
            &format!("Pulling from {} at {url}", service.display_name()),
            Some("⬇️ "),
        );

        // Pull and update the local file
        let modified_files = stencila_codecs::pull(
            &service,
            &url,
            &self.path,
            !self.no_merge,
            DecodeOptions::default(),
            EncodeOptions::default(),
        )
        .await?;

        message("Successfully pulled from remote", Some("✅"));

        if let Some(modified_files) = modified_files {
            if self.no_merge {
                message("Local file replaced successfully", Some("✅"));
            } else {
                message(
                    &format!(
                        "Merge completed, {}",
                        match modified_files.len() {
                            0 => "no changes detected".to_string(),
                            1 => "1 file modified".to_string(),
                            count => format!("{count} files modified"),
                        }
                    ),
                    Some("✅"),
                );
            }
        } else {
            message("Merge cancelled", Some("🚫"));
        }

        // Track the remote pull
        doc.track_remote_pulled(url.clone()).await?;

        Ok(())
    }
}
