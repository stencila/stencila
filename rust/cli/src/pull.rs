use std::path::PathBuf;

use chrono::Utc;
use clap::Parser;
use eyre::{Result, bail, eyre};
use url::Url;

use stencila_cli_utils::{color_print::cstr, message};
use stencila_codecs::{DecodeOptions, EncodeOptions};
use stencila_remotes::{RemoteService, get_remotes_for_path, update_remote_timestamp};

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
    #[arg(long, short)]
    from: Option<String>,

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
  <b>stencila pull</> <g>document.smd</> <c>--from</> <g>gdoc</>

  <dim># Pull from tracked Microsoft 365 document</dim>
  <b>stencila pull</> <g>document.smd</> <c>--from</> <g>m365</>

  <dim># Pull from specific URL</dim>
  <b>stencila pull</> <g>document.smd</> <c>--from</> <g>https://docs.google.com/document/d/abc123</>

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

        // Get remotes for the path
        let remote_infos = get_remotes_for_path(&self.path, None).await?;

        // Determine the target to pull from
        let (service, url) = if let Some(target_str) = &self.from {
            // Target or service shorthand specified
            match target_str.as_str() {
                "gdoc" | "gdocs" => {
                    // Find configured Google Docs remote
                    let url = remote_infos
                        .iter()
                        .find(|info| RemoteService::GoogleDocs.matches_url(&info.url))
                        .ok_or_else(|| eyre!("No Google Doc configured for `{path_display}`"))?
                        .url
                        .clone();
                    (RemoteService::GoogleDocs, url)
                }
                "m365" => {
                    // Find configured Microsoft 365 remote
                    let url = remote_infos
                        .iter()
                        .find(|info| RemoteService::Microsoft365.matches_url(&info.url))
                        .ok_or_else(|| {
                            eyre!("No Microsoft 365 document configured for `{path_display}`")
                        })?
                        .url
                        .clone();
                    (RemoteService::Microsoft365, url)
                }
                "site" | "sites" => {
                    // Find configured Stencila Site remote
                    let url = remote_infos
                        .iter()
                        .find(|info| RemoteService::StencilaSites.matches_url(&info.url))
                        .ok_or_else(|| eyre!("No Stencila Site configured for `{path_display}`"))?
                        .url
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
            // No target or service specified, find any configured remote
            if remote_infos.is_empty() {
                bail!("No remotes configured for `{path_display}`",);
            }

            // Error if multiple remotes are configured
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
                    "Multiple remotes configured for `{path_display}`:\n{remotes_list}\n\nSpecify which one to pull using a service (gdoc/m365/site) or full URL."
                );
            }

            // Find which service the configured remote belongs to
            let remote_info = &remote_infos[0];
            let service = RemoteService::from_url(&remote_info.url).ok_or_else(|| {
                eyre!(
                    "Configured remote {} is not from a supported service",
                    remote_info.url
                )
            })?;

            (service, remote_info.url.clone())
        };

        message!("⬇️ Pulling from {} at {url}", service.display_name());

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

        message("✅ Successfully pulled from remote");

        if let Some(modified_files) = modified_files {
            if self.no_merge {
                message("✅ Local file replaced successfully");
            } else {
                message!(
                    "✅ Merge completed, {}",
                    match modified_files.len() {
                        0 => "no changes detected".to_string(),
                        1 => "1 file modified".to_string(),
                        count => format!("{count} files modified"),
                    },
                );
            }
        } else {
            message("🚫 Merge cancelled");
        }

        // Track the remote pull
        update_remote_timestamp(
            &self.path,
            url.as_ref(),
            Some(Utc::now().timestamp() as u64),
            None,
        )
        .await?;

        Ok(())
    }
}
