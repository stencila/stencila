use std::path::PathBuf;

use clap::Parser;
use eyre::{Result, bail};
use url::Url;

use stencila_cli_utils::{color_print::cstr, message};
use stencila_codecs::remotes::RemoteService;
use stencila_document::Document;

/// Push a document to a remote service
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// The path to the document to push
    input: PathBuf,

    /// The remote service to push to
    ///
    /// If not specified, will use tracked remotes.
    #[arg(long)]
    to: Option<RemoteService>,

    /// Create a new document instead of updating an existing one
    ///
    /// By default, if a remote is already tracked for the document,
    /// it will be updated. Use this flag to create a new document.
    #[arg(long)]
    force_new: bool,

    /// Do not execute the document before pushing it
    ///
    /// By default, the document will be executed to ensure that
    /// it is up-to-date before pushing it. Use this flag to skip execution.
    #[arg(long)]
    no_execute: bool,

    /// Arguments to pass to the document for execution
    ///
    /// If provided, the document will be executed with these arguments
    /// before being pushed. Use -- to separate these from other options.
    #[arg(last = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Push a document to Google Docs</dim>
  <b>stencila push</> <g>document.smd</> <c>--to</> <g>gdocs</>

  <dim># Push and update existing tracked document</dim>
  <b>stencila push</> <g>document.smd</>

  <dim># Push with execution first</dim>
  <b>stencila push</> <g>report.smd</> <c>--to</> <g>gdocs</> <c>--</> <c>arg1=value1</>

  <dim># Force create new document</dim>
  <b>stencila push</> <g>document.smd</> <c>--to</> <g>gdocs</> <c>--force-new</>
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

        // Execute document if args provided
        if !self.no_execute {
            message(
                &format!("Executing `{input}` before pushing it (use `--no-execute` to skip)"),
                Some("⚙️ "),
            );

            // Parse arguments as key=value pairs
            let arguments: Vec<(&str, &str)> = self
                .args
                .iter()
                .filter_map(|arg| {
                    let parts: Vec<&str> = arg.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        Some((parts[0], parts[1]))
                    } else {
                        None
                    }
                })
                .collect();

            doc.call(&arguments, stencila_document::ExecuteOptions::default())
                .await?;
        }

        // Determine target remote service
        let service = if let Some(to) = self.to {
            to
        } else {
            // Check tracked remotes
            let remotes = doc.remotes().await?;
            if remotes.is_empty() {
                bail!("No tracked remotes for `{input}`. Use `--to` to specify target service.",);
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
                    "No supported remotes tracked for `{input}`:\n{urls_list}\n\nUse `--to` to specify target service.",
                );
            }

            // Get the first service
            let (first_service, _) = remote_services[0];

            // Check for multiple remotes for the same service
            let service_remotes: Vec<&Url> = remote_services
                .iter()
                .filter(|(s, _)| *s as u8 == first_service as u8)
                .map(|(_, url)| *url)
                .collect();

            if service_remotes.len() > 1 {
                let urls_list = service_remotes
                    .iter()
                    .map(|url| format!("  - {}", url))
                    .collect::<Vec<_>>()
                    .join("\n");
                message(
                    &format!(
                        "Multiple {} remotes found:\n{urls_list}",
                        first_service.display_name_plural()
                    ),
                    Some("⚠️"),
                );
                bail!(
                    "Use `--to {}` with `--force-new` to create a new document, or untrack remotes you don't want.",
                    first_service.cli_name()
                );
            }

            first_service
        };

        // Determine existing URL for this service
        let existing_url = if self.force_new {
            None
        } else {
            // Get tracked remotes for this service
            let remotes = doc.remotes().await?;
            remotes.iter().find(|url| service.matches_url(url)).cloned()
        };

        // Display appropriate message
        if existing_url.is_some() {
            message(
                &format!(
                    "Updating existing {} linked to `{input}`",
                    service.display_name()
                ),
                Some("🔄"),
            );
        } else {
            message(
                &format!("Creating new {}", service.display_name()),
                Some("☁️ "),
            );
        }

        // Push to the remote service
        let url = stencila_codecs::push(
            &service,
            &doc.root().await,
            doc.file_name(),
            existing_url.as_ref(),
        )
        .await?;

        message(&format!("Successfully pushed to {url}"), Some("✅"));

        // Track the remote
        doc.track_remote_pushed(url.clone()).await?;

        if existing_url.is_none() {
            message(
                &format!(
                    "Tracking new {} as remote for `{input}`",
                    service.display_name()
                ),
                Some("💾"),
            );
        }

        Ok(())
    }
}
