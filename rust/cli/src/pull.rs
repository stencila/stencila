use std::path::PathBuf;

use clap::Parser;
use eyre::{Result, bail};
use url::Url;

use stencila_cli_utils::{color_print::cstr, message};
use stencila_codecs::{DecodeOptions, EncodeOptions, remotes::RemoteService};
use stencila_document::Document;

/// Pull a document from a remote service
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// The path to the local document
    input: PathBuf,

    /// The URL or service to pull from
    ///
    /// Can be a full URL (e.g., https://docs.google.com/document/d/...) or a
    /// service shorthand (e.g "gdoc" or "m365"). Omit to use the tracked
    /// remote (errors if multiple remotes are tracked).
    url: Option<String>,

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
        let input = self.input.display();

        // Validate input file exists
        if !self.input.exists() {
            bail!("Input file `{input}` does not exist");
        }

        // Open the document
        let doc = Document::open(&self.input, None).await?;

        // Determine the URL to pull from
        let (service, url) = if let Some(url_str) = &self.url {
            // URL or service shorthand specified
            match url_str.as_str() {
                "gdoc" | "gdocs" => {
                    // Find tracked Google Docs remote
                    let remotes = doc.remotes().await?;
                    let url = remotes
                        .iter()
                        .find(|u| RemoteService::GoogleDocs.matches_url(u))
                        .ok_or_else(|| {
                            eyre::eyre!(
                                "No tracked Google Docs remote found for `{input}`. Use a full URL to specify one."
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
                            eyre::eyre!(
                                "No tracked Microsoft 365 remote found for `{input}`. Use a full URL to specify one."
                            )
                        })?
                        .clone();
                    (RemoteService::Microsoft365, url)
                }
                _ => {
                    // Assume it's a URL
                    let url =
                        Url::parse(url_str).map_err(|_| eyre::eyre!("Invalid URL: {url_str}"))?;
                    let service = RemoteService::from_url(&url).ok_or_else(|| {
                        eyre::eyre!("URL {url} is not from a supported remote service")
                    })?;
                    (service, url)
                }
            }
        } else {
            // No URL or service specified, find any tracked remote
            let remotes = doc.remotes().await?;
            if remotes.is_empty() {
                bail!(
                    "No tracked remotes for `{input}`. Specify a URL or service (gdoc/m365) to pull from.",
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
                    "Multiple remotes found for `{input}`:\n{remotes_list}\n\nSpecify which one to pull using a service (gdoc/m365) or full URL."
                );
            }

            // Find which service the tracked remote belongs to
            let remote_url = &remotes[0];
            let service = RemoteService::from_url(remote_url).ok_or_else(|| {
                eyre::eyre!(
                    "Tracked remote {} is not from a supported service",
                    remote_url
                )
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
            &self.input,
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
