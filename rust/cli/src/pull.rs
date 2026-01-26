use std::{env::current_dir, path::PathBuf};

use chrono::Utc;
use clap::Parser;
use eyre::{Result, bail, eyre};
use pathdiff::diff_paths;
use tempfile::tempdir;
use url::Url;

use stencila_cli_utils::{color_print::cstr, message};
use stencila_codec_gdoc::GDocError;
use stencila_codec_m365::M365Error;
use stencila_codec_utils::closest_git_repo;
use stencila_codecs::{DecodeOptions, EncodeOptions};
use stencila_remotes::{
    RemoteService, WatchDirection, WatchPrMode, create_and_save_watch, get_remotes_for_path,
    update_remote_timestamp,
};

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

    /// Do not save remote to stencila.toml
    ///
    /// By default, new remotes are added to stencila.toml so team members
    /// can push/pull the same remote. Use this flag to track locally only
    /// (in .stencila/remotes.json).
    #[arg(long)]
    no_config: bool,

    /// Enable watch after successful pull
    ///
    /// Creates a watch in Stencila Cloud to automatically sync changes
    /// between the remote and repository via pull requests.
    #[arg(long, short)]
    watch: bool,

    /// The sync direction (only used with --watch)
    #[arg(long, short, requires = "watch")]
    direction: Option<WatchDirection>,

    /// The GitHub PR mode (only used with --watch)
    #[arg(long, short, requires = "watch")]
    pr_mode: Option<WatchPrMode>,

    /// Debounce time in seconds (10-86400, only used with --watch)
    ///
    /// Time to wait after detecting changes before syncing to avoid
    /// too frequent updates. Minimum 10 seconds, maximum 24 hours (86400 seconds).
    #[arg(long, value_parser = clap::value_parser!(u64).range(10..=86400), requires = "watch")]
    debounce_seconds: Option<u64>,
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

  <dim># Pull without saving to stencila.toml</dim>
  <b>stencila pull</> <g>document.smd</> <c>--from</> <g>gdoc</> <c>--no-config</>

  <dim># Pull and enable bi-directional watch</dim>
  <b>stencila pull</> <g>document.smd</> <c>--from</> <g>gdoc</> <c>--watch</>

  <dim># Pull all documents from email attachments using embedded path metadata</dim>
  <b>stencila pull</> <g>-</> <c>--from</> <g>https://api.stencila.cloud/v1/watches/wAbC12345/email/attachments</>
"
);

impl Cli {
    pub async fn run(self) -> Result<()> {
        // Validate --watch is not used with batch pull mode
        if self.path == PathBuf::from("-") && self.watch {
            bail!("Cannot use --watch with batch pull mode");
        }

        // Validate --watch is not used with --no-config (watch ID must be stored in config)
        if self.watch && self.no_config {
            bail!("Cannot use --watch with --no-config (watch ID must be stored in stencila.toml)");
        }

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
                    // Find configured Microsoft 365 remote, or trigger picker if none
                    match remote_infos
                        .iter()
                        .find(|info| RemoteService::Microsoft365.matches_url(&info.url))
                    {
                        Some(info) => (RemoteService::Microsoft365, info.url.clone()),
                        None => {
                            // No configured remote - use browse workflow with picker
                            return self.run_m365_picker_pull().await;
                        }
                    }
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

        // Check if this is a new remote (not already configured)
        let existing_remote = remote_infos.iter().find(|r| r.url == url);
        let is_new_remote = existing_remote.is_none();
        let has_existing_watch = existing_remote.is_some_and(|r| r.watch_id.is_some());

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

            // Handle config saving and watch creation
            self.post_pull_config_and_watch(&url, is_new_remote, has_existing_watch)
                .await?;

            return Ok(());
        }

        // For Microsoft 365, use the picker-based pull with access denied retry
        if matches!(service, RemoteService::Microsoft365) {
            self.pull_m365_with_retry(&url).await?;

            // Track the remote pull
            update_remote_timestamp(
                &self.path,
                url.as_ref(),
                Some(Utc::now().timestamp() as u64),
                None,
            )
            .await?;

            // Handle config saving and watch creation
            self.post_pull_config_and_watch(&url, is_new_remote, has_existing_watch)
                .await?;

            return Ok(());
        }

        // Pull and update the local file (for other services)
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

        // Handle config saving and watch creation
        self.post_pull_config_and_watch(&url, is_new_remote, has_existing_watch)
            .await?;

        Ok(())
    }

    /// Run Google Docs picker pull mode (browse workflow)
    ///
    /// Opens the Google Picker to let the user browse and select a document,
    /// then pulls and merges it with the local file.
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

        // Handle config saving and watch creation (picker flows always have new remotes, no existing watch)
        self.post_pull_config_and_watch(&selected_url, true, false)
            .await?;

        Ok(())
    }

    /// Pull from a Google Doc URL with interactive retry on access denied
    async fn pull_gdoc_with_retry(&self, url: &Url) -> Result<()> {
        // Create temp directory for the pulled DOCX
        let temp_dir = tempdir()?;
        let pulled_path = temp_dir.path().join("pulled.docx");

        // First attempt
        let result = stencila_codec_gdoc::pull(url, &pulled_path).await;

        match result {
            Ok(()) => {}
            Err(GDocError::NotLinked { connect_url }) => {
                // Google account not linked - open picker to connect and grant access
                let doc_id = stencila_codec_gdoc::extract_doc_id(url)?;
                let picker_url =
                    connect_url.unwrap_or_else(|| stencila_cloud::google_picker_url(Some(&doc_id)));

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
            Err(GDocError::NotFound { doc_id }) => {
                // Document not found - don't trigger picker, just show error
                bail!(
                    "Google Doc '{}' not found. The document may have been deleted or the URL is incorrect.",
                    doc_id
                );
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

    /// Run Microsoft 365 picker pull mode (browse workflow)
    ///
    /// Opens the Microsoft Picker to let the user browse and select a document,
    /// then pulls and merges it with the local file.
    async fn run_m365_picker_pull(&self) -> Result<()> {
        // Open the Microsoft Picker in browser
        let picker_url = stencila_cloud::microsoft_picker_url(None);

        message!("📂 Opening Microsoft Picker to select a document...");

        if let Err(err) = webbrowser::open(&picker_url) {
            message!("⚠️ Failed to open browser: {}", err);
            message!("Please visit manually: {}", picker_url);
        }

        // Wait for user to paste the selected URL
        let url_str =
            stencila_ask::input("After selecting a file, paste the OneDrive URL here").await?;

        let selected_url = Url::parse(url_str.trim()).map_err(|e| eyre!("Invalid URL: {e}"))?;

        // Validate it's a Microsoft 365 URL before proceeding
        if !RemoteService::Microsoft365.matches_url(&selected_url) {
            bail!(
                "URL is not a Microsoft 365 document: {}\n\
                 Expected format: OneDrive or SharePoint URL",
                selected_url
            );
        }

        // Pull from the selected URL (with access denied retry)
        self.pull_m365_with_retry(&selected_url).await?;

        // Track the remote for future pulls
        update_remote_timestamp(
            &self.path,
            selected_url.as_ref(),
            Some(Utc::now().timestamp() as u64),
            None,
        )
        .await?;

        message!("✅ Linked `{}` to {}", self.path.display(), selected_url);

        // Handle config saving and watch creation (picker flows always have new remotes, no existing watch)
        self.post_pull_config_and_watch(&selected_url, true, false)
            .await?;

        Ok(())
    }

    /// Pull from a Microsoft 365 URL with interactive retry on access denied
    async fn pull_m365_with_retry(&self, url: &Url) -> Result<()> {
        // Create temp directory for the pulled DOCX
        let temp_dir = tempdir()?;
        let pulled_path = temp_dir.path().join("pulled.docx");

        // First attempt
        let result = stencila_codec_m365::pull(url, &pulled_path).await;

        match result {
            Ok(()) => {}
            Err(M365Error::NotLinked { connect_url }) => {
                // Microsoft account not linked - open picker to connect and grant access
                let doc_id = stencila_codec_m365::extract_item_id(url).ok();
                let picker_url = connect_url
                    .unwrap_or_else(|| stencila_cloud::microsoft_picker_url(doc_id.as_deref()));

                message!("🔗 Microsoft account not linked to Stencila");
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
                stencila_codec_m365::pull(url, &pulled_path)
                    .await
                    .map_err(|e| {
                        eyre::Report::from(e).wrap_err("Pull failed after connecting account")
                    })?;
            }
            Err(M365Error::RefreshFailed { connect_url }) => {
                // Token refresh failed - open picker to reconnect account
                let picker_url =
                    connect_url.unwrap_or_else(|| stencila_cloud::microsoft_picker_url(None));

                message!("🔄 Microsoft token refresh failed");
                message!("🌐 Opening browser to reconnect your account...");

                if let Err(err) = webbrowser::open(&picker_url) {
                    message!("⚠️ Failed to open browser: {}", err);
                    message!("Please visit manually: {}", picker_url);
                }

                stencila_ask::wait_for_enter(
                    "After reconnecting your account in the picker, press Enter to retry",
                )
                .await?;

                message!("🔄 Retrying download...");

                // Retry after user reconnects
                stencila_codec_m365::pull(url, &pulled_path)
                    .await
                    .map_err(|e| {
                        eyre::Report::from(e).wrap_err("Pull failed after reconnecting account")
                    })?;
            }
            Err(M365Error::AccessDenied { doc_id }) => {
                // Access denied - open picker with specific doc_id so user can grant access
                let picker_url = stencila_cloud::microsoft_picker_url(Some(&doc_id));

                message!("🔒 Access denied to this Microsoft 365 document");
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
                stencila_codec_m365::pull(url, &pulled_path)
                    .await
                    .map_err(|e| {
                        eyre::Report::from(e).wrap_err("Pull failed after granting access")
                    })?;
            }
            Err(M365Error::NotFound { doc_id }) => {
                // Document not found - don't trigger picker, just show error
                bail!(
                    "Microsoft 365 document '{}' not found. The document may have been deleted or the URL is incorrect.",
                    doc_id
                );
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
            message!(
                "✅ Created `{}` from Microsoft 365 doc",
                self.path.display()
            );
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

    /// Handle post-pull config saving and watch creation
    ///
    /// This is called after a successful pull to:
    /// 1. Save the remote to stencila.toml (if new and not --no-config)
    /// 2. Create a watch (if --watch is specified and no existing watch)
    async fn post_pull_config_and_watch(
        &self,
        url: &Url,
        is_new_remote: bool,
        has_existing_watch: bool,
    ) -> Result<()> {
        // Save to config if this is a new remote (and not --no-config)
        if is_new_remote && !self.no_config {
            match stencila_config::config_add_remote(&self.path, url.as_ref()) {
                Ok(config_path) => {
                    let config_path = current_dir()
                        .ok()
                        .and_then(|cwd| diff_paths(&config_path, cwd))
                        .unwrap_or(config_path);
                    message!("📝 Remote added to `{}`", config_path.display());
                }
                Err(error) => {
                    if self.watch {
                        // If --watch is set and config write failed, bail out to avoid orphaned watch
                        bail!(
                            "Could not add remote to config: {}\n\
                             Watch creation skipped to avoid orphaned watch in Cloud.",
                            error
                        );
                    }
                    message!("⚠️ Could not add to config: {}", error);
                }
            }
        } else if is_new_remote && self.no_config {
            // Note: --watch with --no-config is rejected at the start of run()
            message!("💾 Remote tracked locally (not saved to `stencila.toml`)");
        }

        // Enable watch if requested
        if self.watch {
            // Skip if a watch already exists (to avoid orphaning the existing watch)
            if has_existing_watch {
                message!("👁️ Remote already has a watch configured, skipping watch creation");
                return Ok(());
            }

            // For pull, just verify we're in a git repo (file may be new/uncommitted)
            closest_git_repo(&self.path)?;

            create_and_save_watch(
                &self.path,
                url,
                self.direction,
                self.pr_mode,
                self.debounce_seconds,
            )
            .await?;

            let direction_desc = match self.direction.unwrap_or_default() {
                WatchDirection::Bi => "bi-directional",
                WatchDirection::FromRemote => "from remote only",
                WatchDirection::ToRemote => "to remote only",
            };
            message!(
                "👁️ Watch created ({}). PRs will sync changes from {}",
                direction_desc,
                url
            );
        }

        Ok(())
    }
}
