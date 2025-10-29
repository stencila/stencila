use std::path::PathBuf;

use clap::Parser;
use eyre::{Result, bail};
use url::Url;

use stencila_cli_utils::{color_print::cstr, message};
use stencila_cloud::{WatchRequest, create_watch};
use stencila_codec_utils::git_info;
use stencila_codecs::remotes::RemoteService;
use stencila_document::{Document, WatchDirection, WatchPrMode};

/// Push a document to a remote service
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// The path to the document to push
    input: PathBuf,

    /// The URL or service to push to
    ///
    /// Can be a full URL (e.g., https://docs.google.com/document/d/...) or a
    /// service shorthand (e.g "gdoc" or "m365"). Omit to use any tracked
    /// remote.
    url: Option<String>,

    /// Create a new document instead of updating an existing one
    ///
    /// By default, if a remote is already tracked for the document,
    /// it will be updated. Use this flag to create a new document.
    #[arg(long, short = 'n')]
    force_new: bool,

    /// Do not execute the document before pushing it
    ///
    /// By default, the document will be executed to ensure that
    /// it is up-to-date before pushing it. Use this flag to skip execution.
    #[arg(long)]
    no_execute: bool,

    /// Enable watch after successful push
    ///
    /// Creates a watch in Stencila Cloud to automatically sync changes
    /// between the remote and repository via pull requests.
    #[arg(long, short)]
    watch: bool,

    /// The sync direction (only used with --watch)
    #[arg(long, short, requires = "watch")]
    direction: WatchDirection,

    /// The GitHub PR mode (only used with --watch)
    #[arg(long, short, requires = "watch")]
    pr_mode: WatchPrMode,

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
  <b>stencila push</> <g>document.smd</> <g>gdoc</>

  <dim># Push a document to Microsoft 365</dim>
  <b>stencila push</> <g>document.smd</> <g>m365</>

  <dim># Push and update existing tracked document</dim>
  <b>stencila push</> <g>document.smd</>

  <dim># Push to specific URL</dim>
  <b>stencila push</> <g>document.smd</> <g>https://docs.google.com/document/d/abc123</>

  <dim># Push with execution first</dim>
  <b>stencila push</> <g>report.smd</> <g>gdoc</> <c>--</> <c>arg1=value1</>

  <dim># Force create new document</dim>
  <b>stencila push</> <g>document.smd</> <g>gdoc</> <c>--force-new</>
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

        // Determine target remote service, explicit URL, and execution args
        // If the url string looks like an execution arg (starts with '-' or contains '='), treat it as such
        let (service, explicit_url, execution_args) = if let Some(url_str) = self.url {
            if url_str.starts_with('-') || url_str.contains('=') {
                // Looks like an execution arg, not a URL/service
                let mut args = vec![url_str];
                args.extend(self.args);
                (None, None, args)
            } else {
                // Try to determine if it's a service shorthand or a URL
                match url_str.as_str() {
                    "gdoc" | "gdocs" => (Some(RemoteService::GoogleDocs), None, self.args),
                    "m365" => (Some(RemoteService::Microsoft365), None, self.args),
                    _ => {
                        // Try to parse as URL
                        let url = Url::parse(&url_str).map_err(|_| {
                            eyre::eyre!(
                                "Invalid URL or service: '{}'. Use 'gdoc', 'm365', or a full URL.",
                                url_str
                            )
                        })?;
                        let service = RemoteService::from_url(&url).ok_or_else(|| {
                            eyre::eyre!("URL {} is not from a supported remote service", url)
                        })?;
                        (Some(service), Some(url), self.args)
                    }
                }
            }
        } else {
            (None, None, self.args)
        };

        // Execute document if args provided
        if !self.no_execute {
            message(
                &format!("Executing `{input}` before pushing it (use `--no-execute` to skip)"),
                Some("⚙️ "),
            );

            // Parse arguments as key=value pairs
            let arguments: Vec<(&str, &str)> = execution_args
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

        // Determine target remote service from tracked remotes if not specified
        let service = if let Some(svc) = service {
            svc
        } else {
            // Check tracked remotes
            let remotes = doc.remotes().await?;
            if remotes.is_empty() {
                bail!(
                    "No tracked remotes for `{input}`. Specify a service (gdoc/m365) to push to.",
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
                    "No supported remotes tracked for `{input}`:\n{urls_list}\n\nSpecify a service (gdoc/m365) to push to.",
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
                    "Specify '{}' with `--force-new` to create a new document, or untrack remotes you don't want.",
                    first_service.cli_name()
                );
            }

            first_service
        };

        // Determine existing URL for this service
        let existing_url = if let Some(url) = explicit_url {
            // Explicit URL provided - use it directly
            if self.force_new {
                bail!("Cannot use both an explicit URL and --force-new flag");
            }
            Some(url)
        } else if self.force_new {
            // Force new document creation
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

        // Enable watch if requested
        if self.watch {
            // Get git repository information
            let git_info = git_info(&self.input)?;
            let Some(repo_url) = git_info.origin else {
                bail!(
                    "File is not in a git repository. Cannot enable watch without git repository."
                );
            };

            // Get tracking information to get doc_id
            let Some((.., Some(tracking))) = doc.tracking().await? else {
                bail!("Failed to get tracking information for document");
            };

            // Get file path relative to repo root
            let file_path = git_info.path.unwrap_or_else(|| {
                self.input
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            });

            // Call Cloud API to create watch
            let request = WatchRequest {
                remote_url: url.to_string(),
                repo_url,
                file_path,
                direction: self.direction.to_string(),
                pr_mode: Some(self.pr_mode.to_string()),
            };
            let response = create_watch(request).await?;

            // Update docs.json with watch metadata
            let mut remote_info = tracking
                .remotes
                .and_then(|mut remotes| remotes.remove(&url))
                .unwrap_or_default();
            remote_info.watch_id = Some(response.id.to_string());
            remote_info.watch_direction = Some(self.direction);
            doc.track(Some((url, remote_info))).await?;

            // Success message
            let direction_desc = match self.direction {
                WatchDirection::Bi => "bi-directional",
                WatchDirection::FromRemote => "from remote only",
                WatchDirection::ToRemote => "to remote only",
            };

            message(
                &format!(
                    "Watching `{input}` ({direction_desc}). PRs will be opened/updated on changes from the remote."
                ),
                Some("👁️ "),
            );
        }

        Ok(())
    }
}
