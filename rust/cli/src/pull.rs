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
