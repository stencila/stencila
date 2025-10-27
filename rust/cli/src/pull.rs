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

    /// The remote service to pull from
    ///
    /// If not specified, will use tracked remotes.
    #[arg(long)]
    from: Option<RemoteService>,

    /// The URL to pull from
    ///
    /// If not specified, will use the tracked remote for the service.
    #[arg(long)]
    url: Option<Url>,

    /// Do not merge, just download
    ///
    /// By default, the pulled document will be merged with the local version.
    /// Use this flag to skip merging and just replace the local file.
    #[arg(long)]
    no_merge: bool,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Pull from tracked Google Doc</dim>
  <b>stencila pull</> <g>document.smd</>

  <dim># Pull from specific service</dim>
  <b>stencila pull</> <g>document.smd</> <c>--from</> <g>gdocs</>

  <dim># Pull from specific URL</dim>
  <b>stencila pull</> <g>document.smd</> <c>--url</> <g>https://docs.google.com/document/d/abc123</>

  <dim># Pull without merging (replace local file)</dim>
  <b>stencila pull</> <g>document.smd</> <c>--no-merge</>
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
        let (service, url) = if let Some(url) = &self.url {
            // URL specified directly
            let service = RemoteService::from_url(url)
                .ok_or_else(|| eyre::eyre!("URL {} is not from a supported remote service", url))?;
            (service, url.clone())
        } else if let Some(service) = self.from {
            // Service specified, find the tracked remote for it
            let remotes = doc.remotes().await?;
            let url = remotes
                .iter()
                .find(|u| service.matches_url(u))
                .ok_or_else(|| {
                    eyre::eyre!(
                        "No tracked {} remote found for `{input}`. Use `--url` to specify one.",
                        service.display_name()
                    )
                })?
                .clone();
            (service, url)
        } else {
            // No service or URL specified, find any tracked remote
            let remotes = doc.remotes().await?;
            if remotes.is_empty() {
                bail!(
                    "No tracked remotes for `{input}`. Use `--from` or `--url` to specify source.",
                );
            }

            // Find which service(s) the tracked remotes belong to
            let remote_services: Vec<(RemoteService, &Url)> = remotes
                .iter()
                .filter_map(|url| RemoteService::from_url(url).map(|service| (service, url)))
                .collect();

            if remote_services.is_empty() {
                let urls_list = remotes
                    .iter()
                    .map(|url| format!("  - {}", url))
                    .collect::<Vec<_>>()
                    .join("\n");
                bail!(
                    "No supported remotes tracked for `{input}`:\n{urls_list}\n\nUse `--from` or `--url` to specify source.",
                );
            }

            // Use the first service and URL
            let (first_service, first_url) = remote_services[0];
            (first_service, (*first_url).clone())
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
