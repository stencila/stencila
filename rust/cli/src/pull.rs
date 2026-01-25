use std::{
    io::{BufRead, Write},
    path::PathBuf,
};

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

  <dim># Pull from untracked Google Doc</dim>
  <b>stencila pull</> <g>document.smd</> <c>--from</> <g>https://docs.google.com/document/d/abc123</>

  <dim># Pull from tracked Microsoft 365 document</dim>
  <b>stencila pull</> <g>document.smd</> <c>--from</> <g>m365</>

  <dim># Pull from GitHub Issue</dim>
  <b>stencila pull</> <g>document.smd</> <c>--from</> <g>https://github.com/org/repo/issues/123</>

  <dim># Pull without merging (replace local file)</dim>
  <b>stencila pull</> <g>document.smd</> <c>--from</> <g>gdoc</> <c>--no-merge</>

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
        let (service, url) = if let Some(target_str) = &self.from {
            // Target or service shorthand specified
            match target_str.as_str() {
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
                if !self.path.exists() {
                    bail!(
                        "File `{path_display}` does not exist.\n\
                         Use --from with a URL to pull and create the file."
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

        // Wait for user to paste the selected URL (use eprint! for same-line prompt)
        eprint!("After selecting a file, paste the Google Docs URL here: ");
        std::io::stderr().flush()?;
        let url_str = Self::read_line()?;

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

                // Use eprint! for same-line prompt
                eprint!(
                    "After connecting and granting access in the picker, press Enter to retry: "
                );
                std::io::stderr().flush()?;
                Self::wait_for_enter()?;

                message!("🔄 Retrying download...");

                // Retry after user connects and grants access
                stencila_codec_gdoc::pull(url, &pulled_path)
                    .await
                    .map_err(|e| eyre!("{e}"))?;
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

                // Use eprint! for same-line prompt
                eprint!("After granting access in the picker, press Enter to retry: ");
                std::io::stderr().flush()?;
                Self::wait_for_enter()?;

                message!("🔄 Retrying download...");

                // Retry after user grants access via picker
                stencila_codec_gdoc::pull(url, &pulled_path)
                    .await
                    .map_err(|e| eyre!("{e}"))?;
            }
            Err(e) => bail!("{e}"),
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

    /// Wait for user to press Enter
    fn wait_for_enter() -> Result<()> {
        let stdin = std::io::stdin();
        let mut line = String::new();
        stdin.lock().read_line(&mut line)?;
        Ok(())
    }

    /// Read a line from stdin
    fn read_line() -> Result<String> {
        let stdin = std::io::stdin();
        let mut line = String::new();
        stdin.lock().read_line(&mut line)?;
        Ok(line)
    }

    /// Run batch pull mode, pulling all documents from a multi-file remote
    async fn run_batch_pull(&self) -> Result<()> {
        // Require --from argument
        let Some(target_str) = &self.from else {
            bail!("The --from argument is required when using `-` to pull from path metadata");
        };

        // Parse URL and get service
        let url = Url::parse(target_str)
            .map_err(|_| eyre!("Invalid URL for batch pull: {target_str}"))?;
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
