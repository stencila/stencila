use std::path::PathBuf;

use chrono::Utc;
use clap::Parser;
use eyre::{Result, bail, eyre};
use tempfile::tempdir;
use url::Url;

use stencila_cli_utils::{color_print::cstr, message};
use stencila_codec_gdoc::GDocError;
use stencila_codecs::{DecodeOptions, EncodeOptions};
use stencila_remotes::{RemoteService, get_remotes_for_path, update_remote_timestamp};

/// Pull a document from a remote service
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// The path to the local document
    ///
    /// Use `-` to pull all documents from a multi-file remote (like email
    /// attachments or GitHub Issues) using their embedded path metadata.
    path: PathBuf,

    /// The URL to pull from
    ///
    /// Can be a full URL (e.g., https://docs.google.com/document/d/...).
    /// If omitted, pulls from the tracked remote for this file.
    url: Option<String>,

    /// Select which remote service to pull from
    ///
    /// Use a service shorthand (e.g., "gdoc", "m365", or "ghi") to select
    /// from tracked remotes when multiple exist, or to trigger the picker
    /// workflow when no remotes are configured.
    #[arg(long, short, conflicts_with = "url")]
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
  <dim># Pull from a specific Google Doc URL</dim>
  <b>stencila pull</> <g>document.smd</> <g>https://docs.google.com/document/d/abc123</>

  <dim># Pull from the tracked remote (if only one exists)</dim>
  <b>stencila pull</> <g>document.smd</>

  <dim># Pull from tracked Google Doc (when multiple remotes exist)</dim>
  <b>stencila pull</> <g>document.smd</> <c>--from</> <g>gdoc</>

  <dim># Pull from tracked Microsoft 365 document</dim>
  <b>stencila pull</> <g>document.smd</> <c>--from</> <g>m365</>

  <dim># Pull from a GitHub Issue URL</dim>
  <b>stencila pull</> <g>document.smd</> <g>https://github.com/org/repo/issues/123</>

  <dim># Pull without merging (replace local file)</dim>
  <b>stencila pull</> <g>document.smd</> <c>--no-merge</>

  <dim># Pull all documents from email attachments using embedded path metadata</dim>
  <b>stencila pull</> <g>-</> <c>--from</> <g>https://api.stencila.cloud/v1/watches/wAbC12345/email/attachments</>
"
);

impl Cli {
    pub async fn run(self) -> Result<()> {
        // Check for batch pull mode (path is "-")
        if self.path == PathBuf::from("-") {
            return self.run_batch_pull().await;
        }

        let path_display = self.path.display();

        // Get remotes for the path (only if file exists)
        let remote_infos = if self.path.exists() {
            get_remotes_for_path(&self.path, None).await?
        } else {
            Vec::new()
        };

        // Determine the target to pull from
        let (service, url) = if let Some(url_str) = &self.url {
            // Positional URL provided - use it directly
            let url = Url::parse(url_str).map_err(|_| eyre!("Invalid URL: {url_str}"))?;
            let service = RemoteService::from_url(&url)
                .ok_or_else(|| eyre!("URL {url} is not from a supported remote service"))?;
            (service, url)
        } else if let Some(service_str) = &self.from {
            // Service shorthand specified - select from tracked remotes or trigger picker
            match service_str.as_str() {
                "gdoc" | "gdocs" => {
                    // Find configured Google Docs remote, or trigger picker if none
                    match remote_infos
                        .iter()
                        .find(|info| RemoteService::GoogleDocs.matches_url(&info.url))
                    {
                        Some(info) => (RemoteService::GoogleDocs, info.url.clone()),
                        None => {
                            // No configured remote - use browse workflow with picker
                            return self.run_gdoc_picker_pull().await;
                        }
                    }
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
                "ghi" => {
                    // Find configured GitHub Issue remote
                    let url = remote_infos
                        .iter()
                        .find(|info| RemoteService::GitHubIssues.matches_url(&info.url))
                        .ok_or_else(|| eyre!("No GitHub Issue configured for `{path_display}`"))?
                        .url
                        .clone();
                    (RemoteService::GitHubIssues, url)
                }
                _ => {
                    // Try parsing as URL for backwards compatibility
                    if let Ok(url) = Url::parse(service_str) {
                        if let Some(service) = RemoteService::from_url(&url) {
                            message!(
                                "⚠️ Passing URLs via --from is deprecated. Use positional argument instead:\n   \
                                 stencila pull {} {}",
                                path_display,
                                url
                            );
                            (service, url)
                        } else {
                            bail!("URL {url} is not from a supported remote service");
                        }
                    } else {
                        bail!("Unknown service: `{service_str}`. Use 'gdoc', 'm365', or 'ghi'.");
                    }
                }
            }
        } else {
            // No target or service specified, find any configured remote
            if remote_infos.is_empty() {
                if !self.path.exists() {
                    bail!(
                        "File `{path_display}` does not exist.\n\
                         Provide a URL to pull from: stencila pull {path_display} <URL>"
                    );
                }
                bail!("No remotes configured for `{path_display}`");
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
                    "Multiple remotes configured for `{path_display}`:\n{remotes_list}\n\nSpecify which one to pull using a service (gdoc/m365/ghi/site) or full URL."
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

        message!("⬇️ Pulling from {} at {}", service.display_name(), url);

        // For Google Docs, use the picker-based pull with access denied retry
        if matches!(service, RemoteService::GoogleDocs) {
            self.pull_gdoc_with_retry(&url).await?;

            // Track the remote pull
            update_remote_timestamp(
                &self.path,
                url.as_ref(),
                Some(Utc::now().timestamp() as u64),
                None,
            )
            .await?;

            return Ok(());
        }

        // Pull and update the local file (for non-Google Docs services)
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

    /// Run Google Docs picker pull mode (browse workflow)
    ///
    /// Opens the Google Picker to let the user browse and select a document,
    /// then pulls and merges it with the local file.
    #[allow(clippy::print_stderr)]
    async fn run_gdoc_picker_pull(&self) -> Result<()> {
        // Open the Google Picker in browser
        let picker_url = stencila_cloud::google_picker_url(None);

        message!("📂 Opening Google Picker to select a document...");

        if let Err(err) = webbrowser::open(&picker_url) {
            message!("⚠️ Failed to open browser: {}", err);
            message!("Please visit manually: {}", picker_url);
        }

        // Wait for user to paste the selected URL
        let url_str =
            stencila_ask::input("After selecting a file, paste the Google Docs URL here").await?;

        let selected_url = Url::parse(url_str.trim()).map_err(|e| eyre!("Invalid URL: {e}"))?;

        // Validate it's a Google Docs URL before proceeding
        if !RemoteService::GoogleDocs.matches_url(&selected_url) {
            bail!(
                "URL is not a Google Docs document: {}\n\
                 Expected format: https://docs.google.com/document/d/...",
                selected_url
            );
        }

        // Pull from the selected URL (with access denied retry)
        self.pull_gdoc_with_retry(&selected_url).await?;

        // Track the remote for future pulls
        update_remote_timestamp(
            &self.path,
            selected_url.as_ref(),
            Some(Utc::now().timestamp() as u64),
            None,
        )
        .await?;

        message!("✅ Linked `{}` to {}", self.path.display(), selected_url);

        Ok(())
    }

    /// Pull from a Google Doc URL with interactive retry on access denied
    #[allow(clippy::print_stderr)]
    async fn pull_gdoc_with_retry(&self, url: &Url) -> Result<()> {
        // Create temp directory for the pulled DOCX
        let temp_dir = tempdir()?;
        let pulled_path = temp_dir.path().join("pulled.docx");

        // First attempt
        let result = stencila_codec_gdoc::pull(url, &pulled_path).await;

        match result {
            Ok(()) => {}
            Err(GDocError::NotLinked) => {
                // Google account not linked - open picker to connect and grant access
                let doc_id = stencila_codec_gdoc::extract_doc_id(url)?;
                let picker_url = stencila_cloud::google_picker_url(Some(&doc_id));

                message!("🔗 Google account not linked to Stencila");
                message!("🌐 Opening browser to connect your account and grant access...");

                if let Err(err) = webbrowser::open(&picker_url) {
                    message!("⚠️ Failed to open browser: {}", err);
                    message!("Please visit manually: {}", picker_url);
                }

                stencila_ask::wait_for_enter(
                    "After connecting and granting access in the picker, press Enter to retry",
                )
                .await?;

                message!("🔄 Retrying download...");

                // Retry after user connects and grants access
                stencila_codec_gdoc::pull(url, &pulled_path)
                    .await
                    .map_err(|e| {
                        eyre::Report::from(e).wrap_err("Pull failed after connecting account")
                    })?;
            }
            Err(GDocError::AccessDenied { doc_id }) => {
                // Access denied - open picker with specific doc_id so user can grant access
                let picker_url = stencila_cloud::google_picker_url(Some(&doc_id));

                message!("🔒 Access denied to this Google Doc");
                message!("🌐 Opening browser so you can grant Stencila access to this document...");

                if let Err(err) = webbrowser::open(&picker_url) {
                    message!("⚠️ Failed to open browser: {}", err);
                    message!("Please visit manually: {}", picker_url);
                }

                stencila_ask::wait_for_enter(
                    "After granting access in the picker, press Enter to retry",
                )
                .await?;

                message!("🔄 Retrying download...");

                // Retry after user grants access via picker
                stencila_codec_gdoc::pull(url, &pulled_path)
                    .await
                    .map_err(|e| {
                        eyre::Report::from(e).wrap_err("Pull failed after granting access")
                    })?;
            }
            Err(e) => return Err(eyre::Report::from(e)),
        }

        // Merge or replace the local file
        if !self.no_merge && self.path.exists() {
            let modified_files = stencila_codecs::merge(
                &pulled_path,
                Some(&self.path),
                None,
                None,
                false,
                DecodeOptions::default(),
                EncodeOptions::default(),
                None,
            )
            .await?;

            if let Some(modified_files) = modified_files {
                message!(
                    "✅ Merge completed, {}",
                    match modified_files.len() {
                        0 => "no changes detected".to_string(),
                        1 => "1 file modified".to_string(),
                        count => format!("{count} files modified"),
                    },
                );
            } else {
                message("🚫 Merge cancelled");
            }
        } else {
            // Convert to target format (or file doesn't exist yet)
            stencila_codecs::convert(
                Some(&pulled_path),
                Some(&self.path),
                Some(DecodeOptions::default()),
                Some(EncodeOptions::default()),
            )
            .await?;
            message!("✅ Created `{}` from Google Doc", self.path.display());
        }

        Ok(())
    }

    /// Run batch pull mode, pulling all documents from a multi-file remote
    async fn run_batch_pull(&self) -> Result<()> {
        // Get URL from positional argument or --from flag
        let url_str = self.url.as_ref().or(self.from.as_ref()).ok_or_else(|| {
            eyre!(
                "A URL is required when using `-` to pull from path metadata.\n\
                   Usage: stencila pull - <URL>"
            )
        })?;

        // Parse URL and get service
        let url = Url::parse(url_str).map_err(|_| {
            // Check if it's a service shorthand - those don't work in batch mode
            if matches!(url_str.as_str(), "gdoc" | "gdocs" | "m365" | "ghi") {
                eyre!(
                    "Batch pull requires a full URL, not a service name.\n\
                     Usage: stencila pull - <URL>"
                )
            } else {
                eyre!("Invalid URL for batch pull: {url_str}")
            }
        })?;
        let service = RemoteService::from_url(&url)
            .ok_or_else(|| eyre!("URL {url} is not from a supported remote service"))?;

        message!(
            "⬇️ Pulling from {} at {}",
            service.display_name_plural(),
            url
        );

        // Pull all files in one operation (fetches attachments once)
        // Returns (target_path, temp_file) pairs for us to convert/merge
        let Some(pulled_files) = service.pull_all(&url).await? else {
            bail!(
                "{} does not support batch pull with `-`",
                service.display_name()
            );
        };

        if pulled_files.is_empty() {
            bail!("No documents with path metadata found");
        }

        // Convert/merge each file and collect the paths that were updated
        let mut pulled_paths = Vec::new();
        for (target_path, temp_file) in pulled_files {
            // Create parent directories if needed
            if let Some(parent) = target_path.parent()
                && !parent.as_os_str().is_empty()
            {
                tokio::fs::create_dir_all(parent).await?;
            }

            if !self.no_merge && target_path.exists() {
                // Merge the pulled version with the local file
                stencila_codecs::merge(
                    temp_file.path(),
                    Some(&target_path),
                    None,
                    None,
                    false,
                    DecodeOptions::default(),
                    EncodeOptions::default(),
                    None,
                )
                .await?;
            } else {
                // Convert to target format (or replace if --no-merge)
                stencila_codecs::convert(
                    Some(temp_file.path()),
                    Some(&target_path),
                    Some(DecodeOptions::default()),
                    Some(EncodeOptions::default()),
                )
                .await?;
            }

            pulled_paths.push(target_path);
        }

        // Track each pulled file
        for path in &pulled_paths {
            update_remote_timestamp(
                path,
                url.as_ref(),
                Some(Utc::now().timestamp() as u64),
                None,
            )
            .await?;
        }

        message!("✅ Successfully pulled {} document(s):", pulled_paths.len());
        for path in &pulled_paths {
            message!("  - {}", path.display());
        }

        Ok(())
    }
}
