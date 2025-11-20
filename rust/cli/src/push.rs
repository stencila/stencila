use std::{collections::BTreeMap, path::PathBuf, process::exit};

use chrono::Utc;
use clap::Parser;
use eyre::{Result, bail, eyre};
use url::Url;

use stencila_cli_utils::{color_print::cstr, message};
use stencila_cloud::{WatchRequest, create_watch};
use stencila_codec_utils::{git_info, validate_file_on_default_branch};
use stencila_codecs::PushResult;
use stencila_dirs::closest_workspace_dir;
use stencila_document::Document;
use stencila_remotes::{
    RemoteService, WatchDirection, WatchPrMode, expand_path_to_files, get_remotes_for_path,
    update_remote_timestamp,
};

/// Push a document to a remote service
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// The path of the document to push
    ///
    /// If omitted, pushes all tracked files that have remotes.
    path: Option<PathBuf>,

    /// The target to push to
    ///
    /// Can be a full URL (e.g., https://docs.google.com/document/d/...) or a
    /// service shorthand (e.g "gdoc" or "m365"). Omit to push to all tracked
    /// remotes for the path.
    #[arg(long, short)]
    to: Option<String>,

    /// Create a new document instead of updating an existing one
    ///
    /// By default, if a remote is already tracked for the document,
    /// it will be updated. Use this flag to create a new document.
    #[arg(long, short)]
    new: bool,

    /// Force push even if file is up-to-date
    ///
    /// By default, files pushed to Stencila Sites are only uploaded if they
    /// are out of date compared to the remote. Use this flag to push regardless
    /// of status. This flag only affects Stencila Sites pushes.
    #[arg(long, short)]
    force: bool,

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

    /// Perform a dry run (Stencila Sites only)
    ///
    /// Instead of uploading to the remote site, write the generated files
    /// to a local directory for inspection. If no directory is specified,
    /// files are generated in memory without being written to disk.
    ///
    /// The directory structure mirrors the R2 bucket layout:
    /// {output_dir}/{site_id}/{branch_slug}/{path}
    #[arg(long, value_name = "DIR", num_args = 0..=1, default_missing_value = "", conflicts_with = "watch")]
    dry_run: Option<String>,

    /// Arguments to pass to the document for execution
    ///
    /// If provided, the document will be executed with these arguments
    /// before being pushed. Use -- to separate these from other options.
    #[arg(last = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Push all files with tracked remotes</dim>
  <b>stencila push</>

  <dim># Push a document to Google Docs</dim>
  <b>stencila push</> <g>document.smd</> <c>--to</> <g>gdoc</>

  <dim># Push a document to Microsoft 365</dim>
  <b>stencila push</> <g>document.smd</> <c>--to</> <g>m365</>

  <dim># Push a document to a Stencila Site</dim>
  <b>stencila push</> <g>document.smd</> <c>--to</> <g>site</>

  <dim># Push to file to all tracked remotes</dim>
  <b>stencila push</> <g>document.smd</>

  <dim># Push to specific remote</dim>
  <b>stencila push</> <g>document.smd</> <c>--to</> <g>https://docs.google.com/document/d/abc123</>

  <dim># Push with execution first</dim>
  <b>stencila push</> <g>report.smd</> <c>--to</> <g>gdoc</> <c>--</> <c>arg1=value1</>

  <dim># Force create new document</dim>
  <b>stencila push</> <g>document.smd</> <c>--to</> <g>gdoc</> <c>--new</>

  <dim># Force push even if up-to-date (useful for sites)</dim>
  <b>stencila push</> <g>document.smd</> <c>--to</> <g>site</> <c>--force</>

  <dim># Perform a dry-run to inspect generated files without uploading</dim>
  <b>stencila push</> <g>document.smd</> <c>--to</> <g>site</> <c>--dry-run=./temp</>
"
);

/// Get the display URL for a pushed document
///
/// For Stencila Sites, returns the browseable (branch-aware) URL.
/// For other services, returns the canonical URL.
fn get_display_url(service: &RemoteService, url: &Url, doc_path: Option<&std::path::Path>) -> Url {
    if matches!(service, RemoteService::StencilaSites) {
        stencila_codec_site::browseable_url(url, doc_path).unwrap_or_else(|_| url.clone())
    } else {
        url.clone()
    }
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        // Validate and construct dry-run options
        let dry_run_opts = if let Some(ref dir_str) = self.dry_run {
            let output_dir = if dir_str.is_empty() {
                None
            } else {
                Some(PathBuf::from(dir_str))
            };
            Some(stencila_codecs::PushDryRunOptions {
                enabled: true,
                output_dir,
            })
        } else {
            None
        };

        let is_dry_run = dry_run_opts.is_some();

        // Handle pushing all tracked files when no input is provided
        let Some(path) = self.path else {
            return self.push_all().await;
        };

        // Display early dry-run notification
        if is_dry_run {
            message(
                "Performing dry-run, no remotes will actually be created or updated",
                Some("ℹ️ "),
            );
        }

        let path_display = path.display();

        // Validate input file exists
        if !path.exists() {
            bail!("Input file `{path_display}` does not exist");
        }

        // Open the document
        let doc = Document::open(&path, None).await?;

        // Early validation: --watch is not compatible with multiple remotes
        if self.watch && self.to.is_none() {
            let remote_infos = get_remotes_for_path(&path, None).await?;
            if remote_infos.len() > 1 {
                let urls_list = remote_infos
                    .iter()
                    .map(|info| format!("  - {}", info.url))
                    .collect::<Vec<_>>()
                    .join("\n");
                bail!(
                    "Cannot enable watch with multiple tracked remotes:\n{urls_list}\n\nSpecify a remote target to watch."
                );
            }
        }

        // Determine target remote service, explicit URL, and execution args
        // If the target string looks like an execution arg (starts with '-' or contains '='), treat it as such
        let (service, explicit_target, execution_args) = if let Some(target_str) = self.to {
            if target_str.starts_with('-') || target_str.contains('=') {
                // Looks like an execution arg, not a target/service
                let mut args = vec![target_str];
                args.extend(self.args);
                (None, None, args)
            } else {
                // Try to determine if it's a service shorthand or a URL
                match target_str.as_str() {
                    "gdoc" | "gdocs" => (Some(RemoteService::GoogleDocs), None, self.args),
                    "m365" => (Some(RemoteService::Microsoft365), None, self.args),
                    "site" | "sites" => (Some(RemoteService::StencilaSites), None, self.args),
                    _ => {
                        // Try to parse as URL
                        let url = Url::parse(&target_str).map_err(|_| {
                            eyre!("Invalid target or service: `{target_str}`. Use 'gdoc', 'm365', or a full URL.")
                        })?;
                        let service = RemoteService::from_url(&url).ok_or_else(|| {
                            eyre!("URL {url} is not from a supported remote service")
                        })?;
                        (Some(service), Some(url), self.args)
                    }
                }
            }
        } else {
            (None, None, self.args)
        };

        // Validate: --watch is not supported with Stencila Sites
        if self.watch && matches!(service, Some(RemoteService::StencilaSites)) {
            bail!(
                "Watch is not supported for Stencila Sites. Sites are write-only remotes that don't support bidirectional sync."
            );
        }

        // Execute document if args provided
        if !self.no_execute {
            message(
                &format!(
                    "Executing `{path_display}` before pushing it (use `--no-execute` to skip)"
                ),
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

        // Handle multi-remote push when no service/target is specified
        if service.is_none() && explicit_target.is_none() {
            let remote_infos = get_remotes_for_path(&path, None).await?;

            if remote_infos.is_empty() {
                bail!(
                    "No remotes configured for `{path_display}`. Specify a service (gdoc/m365/site) to push to.",
                );
            }

            // If multiple remotes, push to all of them
            if remote_infos.len() > 1 {
                // Validate dry-run is not allowed with multiple remotes
                if is_dry_run {
                    bail!(
                        "Cannot use `--dry-run` when pushing to multiple remotes. Specify a target remote to use dry-run."
                    );
                }

                message(
                    &format!(
                        "Pushing `{path_display}` to {} configured remotes",
                        remote_infos.len()
                    ),
                    Some("☁️ "),
                );

                let mut successes: Vec<Url> = Vec::new();
                let mut errors: Vec<(Url, String)> = Vec::new();

                for remote_info in remote_infos {
                    let remote_url = &remote_info.url;
                    let remote_service = match RemoteService::from_url(remote_url) {
                        Some(svc) => svc,
                        None => {
                            errors.push((
                                remote_url.clone(),
                                format!(
                                    "URL {} is not from a supported remote service",
                                    remote_url
                                ),
                            ));
                            continue;
                        }
                    };

                    message(
                        &format!(
                            "Updating {} linked to `{path_display}`",
                            remote_service.display_name()
                        ),
                        Some("🔄"),
                    );

                    match stencila_codecs::push(
                        &remote_service,
                        &doc.root().await,
                        doc.path(),
                        doc.file_name(),
                        Some(remote_url),
                        doc.path(),
                        dry_run_opts.clone(),
                    )
                    .await
                    {
                        Ok(result) => {
                            let url = result.url();
                            if let Err(e) = update_remote_timestamp(
                                &path,
                                url.as_ref(),
                                None,
                                Some(Utc::now().timestamp() as u64),
                            )
                            .await
                            {
                                errors.push((
                                    remote_url.clone(),
                                    format!("Failed to track remote: {}", e),
                                ));
                            } else {
                                let display_url =
                                    get_display_url(&remote_service, &url, doc.path());
                                message(
                                    &format!("Successfully pushed to {display_url}"),
                                    Some("✅"),
                                );
                                successes.push(url);
                            }
                        }
                        Err(e) => {
                            message(&format!("Failed to push to {remote_url}: {e}"), Some("❌"));
                            errors.push((remote_url.clone(), e.to_string()));
                        }
                    }
                }

                // Display summary
                message(
                    &format!(
                        "Push complete: {} succeeded, {} failed",
                        successes.len(),
                        errors.len()
                    ),
                    Some("📊"),
                );

                if !errors.is_empty() {
                    exit(1)
                }

                return Ok(());
            }
        }

        // Determine target remote service from config remotes if not specified
        let service = if let Some(svc) = service {
            svc
        } else {
            // Check config remotes
            let remote_infos = get_remotes_for_path(&path, None).await?;
            if remote_infos.is_empty() {
                bail!(
                    "No remotes configured for `{path_display}`. Add remotes to stencila.yaml or specify a service (gdoc/m365/site) to push to.",
                );
            }

            // Find which service(s) the configured remotes belong to
            let remote_services: Vec<(RemoteService, &Url)> = remote_infos
                .iter()
                .filter_map(|info| {
                    RemoteService::from_url(&info.url).map(|service| (service, &info.url))
                })
                .collect();

            if remote_services.is_empty() {
                let urls_list = remote_infos
                    .iter()
                    .map(|info| format!("  - {}", info.url))
                    .collect::<Vec<_>>()
                    .join("\n");
                bail!(
                    "No supported remotes configured for `{path_display}`:\n{urls_list}\n\nSpecify a service (gdoc/m365/site) to push to.",
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
                    "Specify '{}' with `--new` to create a new document, or use a specific URL as target.",
                    first_service.cli_name()
                );
            }

            first_service
        };

        // Validate: --watch is not supported with Stencila Sites (check after service resolution)
        if self.watch && matches!(service, RemoteService::StencilaSites) {
            bail!(
                "Watch is not supported for Stencila Sites. Sites are write-only remotes that don't support bidirectional sync."
            );
        }

        // Determine existing URL for this service
        let existing_url = if let Some(url) = explicit_target {
            // Explicit target provided - use it directly
            if self.new {
                bail!("Cannot use both an explicit target and --new flag");
            }
            Some(url)
        } else if self.new {
            // Force new document creation
            None
        } else {
            // Get configured remotes for this service
            let remote_infos = get_remotes_for_path(&path, None).await?;
            remote_infos
                .iter()
                .find(|info| service.matches_url(&info.url))
                .map(|info| info.url.clone())
        };

        // Display appropriate message
        if existing_url.is_some() {
            message(
                &format!(
                    "Updating existing {} linked to `{path_display}`",
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
        let result = stencila_codecs::push(
            &service,
            &doc.root().await,
            doc.path(),
            doc.file_name(),
            existing_url.as_ref(),
            doc.path(),
            dry_run_opts.clone(),
        )
        .await?;

        // Handle the result based on whether it was a dry-run or actual push
        let url = match result {
            PushResult::Uploaded(url) => {
                let display_url = get_display_url(&service, &url, doc.path());
                message(&format!("Successfully pushed to {display_url}"), Some("✅"));
                url
            }
            PushResult::DryRun {
                url,
                files,
                output_dir,
            } => {
                // Display dry-run results
                let total_size: u64 = files.iter().map(|f| f.size).sum();
                let compressed_count = files.iter().filter(|f| f.compressed).count();

                message(
                    &format!(
                        "Dry-run complete. Would upload {} file(s), total size: {} bytes ({} compressed)",
                        files.len(),
                        total_size,
                        compressed_count
                    ),
                    Some("📊"),
                );

                if let Some(dir) = output_dir {
                    message(&format!("Files written to: {}", dir.display()), Some("📁"));
                }

                // Display file list
                for file in &files {
                    let compressed_marker = if file.compressed { " (gzipped)" } else { "" };
                    let route_info = if let Some(route) = &file.route {
                        format!(" → {}", route)
                    } else {
                        String::new()
                    };
                    message(
                        &format!(
                            "  {}{}{} ({} bytes)",
                            file.storage_path, compressed_marker, route_info, file.size
                        ),
                        Some("  "),
                    );
                }

                let display_url = get_display_url(&service, &url, doc.path());
                message(
                    &format!("Would be available at: {}", display_url),
                    Some("🔗"),
                );

                url
            }
        };

        if is_dry_run {
            // Skip tracking and watching if this is a dry run
            return Ok(());
        }

        // Track the remote (always use canonical URL for tracking)
        update_remote_timestamp(
            &path,
            url.as_ref(),
            None,
            Some(Utc::now().timestamp() as u64),
        )
        .await?;

        if existing_url.is_none() {
            message(
                &format!(
                    "New {} remote for `{path_display}` (add to stencila.yaml to track)",
                    service.display_name()
                ),
                Some("💾"),
            );
        }

        // Enable watch if requested
        if self.watch {
            // Validate file exists on the default branch (also validates it's in a git repo)
            validate_file_on_default_branch(&path)?;

            // Get git repository information
            let git_info = git_info(&path)?;
            let Some(repo_url) = git_info.origin else {
                bail!(
                    "File is not in a git repository. Cannot enable watch without git repository."
                );
            };

            // Verify tracking information exists
            let Some(..) = doc.tracking().await? else {
                bail!("Failed to get tracking information for document");
            };

            // Get file path relative to repo root
            let file_path = git_info.path.unwrap_or_else(|| {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            });

            // Call Cloud API to create watch
            let request = WatchRequest {
                remote_url: url.to_string(),
                repo_url,
                file_path,
                direction: self.direction.map(|dir| dir.to_string()),
                pr_mode: self.pr_mode.map(|mode| mode.to_string()),
                debounce_seconds: self.debounce_seconds,
            };
            let response = create_watch(request).await?;

            // Update stencila.yaml with watch ID
            stencila_config::config_update_remote_watch(
                &path,
                url.as_ref(),
                Some(response.id.to_string()),
            )?;

            let url_str = url.to_string();

            // Success message
            let direction_desc = match self.direction.unwrap_or_default() {
                WatchDirection::Bi => "bi-directional",
                WatchDirection::FromRemote => "from remote only",
                WatchDirection::ToRemote => "to remote only",
            };

            message(
                &format!(
                    "Watching `{path_display}` ({direction_desc}). PRs will be opened/updated on changes from {url_str}."
                ),
                Some("👁️ "),
            );
        }

        Ok(())
    }

    /// Push all tracked files that have remotes
    #[allow(clippy::print_stderr)]
    async fn push_all(&self) -> Result<()> {
        // Validate watch flag is not allowed with multiple files
        if self.watch {
            bail!(
                "Cannot use `--watch` when pushing multiple files. Specify a file path to enable watch."
            );
        }

        // Validate dry-run flag is not allowed with multiple files
        if self.dry_run.is_some() {
            bail!(
                "Cannot use `--dry-run` when pushing multiple files. Specify a file path to use dry-run."
            );
        }

        // Load config to find remotes
        let cwd = std::env::current_dir()?;

        // Find workspace directory to resolve config paths correctly
        let workspace_dir = closest_workspace_dir(&cwd, false).await?;
        let config = stencila_config::config(&workspace_dir)?;

        let Some(remotes_config) = config.remotes else {
            bail!(
                "No remotes configured in `stencila.yaml`. Add remotes to the config file to push."
            );
        };

        // Collect all files with their remotes
        let mut files_with_remotes: BTreeMap<PathBuf, Vec<Url>> = BTreeMap::new();

        for remote_config in remotes_config {
            let config_path = remote_config.path.resolve(&workspace_dir);

            // Expand path to actual files
            let files = expand_path_to_files(&config_path)?;

            // Parse remote URL
            let remote_url = match Url::parse(&remote_config.url) {
                Ok(url) => url,
                Err(e) => {
                    message(
                        &format!(
                            "Skipping invalid URL '{}' in config: {}",
                            remote_config.url, e
                        ),
                        Some("⚠️"),
                    );
                    continue;
                }
            };

            // Add to files map
            for file in files {
                files_with_remotes
                    .entry(file)
                    .or_default()
                    .push(remote_url.clone());
            }
        }

        if files_with_remotes.is_empty() {
            bail!(
                "No files found with configured remotes. Check that paths in stencila.yaml exist."
            );
        }

        message(
            &format!(
                "Pushing {} file(s) with configured remotes",
                files_with_remotes.len()
            ),
            Some("☁️ "),
        );

        let mut total_successes = 0;
        let mut total_errors = 0;
        let mut file_results: Vec<(PathBuf, usize, usize)> = Vec::new();

        for (file_path, remote_urls) in files_with_remotes {
            let file_display = file_path.display();

            message(
                &format!(
                    "Processing `{file_display}` ({} remote(s))",
                    remote_urls.len()
                ),
                Some("📄"),
            );

            // Open the document
            let doc = match Document::open(&file_path, None).await {
                Ok(d) => d,
                Err(e) => {
                    message(&format!("Failed to open `{file_display}`: {e}"), Some("❌"));
                    total_errors += remote_urls.len();
                    file_results.push((file_path.clone(), 0, remote_urls.len()));
                    continue;
                }
            };

            // Execute document if needed
            if !self.no_execute {
                message(
                    &format!("Executing `{file_display}` before pushing"),
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

                if let Err(e) = doc
                    .call(&arguments, stencila_document::ExecuteOptions::default())
                    .await
                {
                    message(
                        &format!("Failed to execute `{file_display}`: {e}"),
                        Some("❌"),
                    );
                    total_errors += remote_urls.len();
                    file_results.push((file_path.clone(), 0, remote_urls.len()));
                    continue;
                }
            }

            // Push to each remote for this file
            let mut file_successes = 0;
            let mut file_errors = 0;

            for remote_url in remote_urls {
                let remote_service = match RemoteService::from_url(&remote_url) {
                    Some(svc) => svc,
                    None => {
                        message(
                            &format!("Skipping unsupported remote: {remote_url}"),
                            Some("⚠️"),
                        );
                        file_errors += 1;
                        continue;
                    }
                };

                message(
                    &format!(
                        "Updating {} linked to `{file_display}`",
                        remote_service.display_name()
                    ),
                    Some("🔄"),
                );

                match stencila_codecs::push(
                    &remote_service,
                    &doc.root().await,
                    doc.path(),
                    doc.file_name(),
                    Some(&remote_url),
                    doc.path(),
                    None, // Dry-run not supported in push_all
                )
                .await
                {
                    Ok(result) => {
                        let url = result.url();
                        let display_url = get_display_url(&remote_service, &url, doc.path());

                        if let Err(e) = update_remote_timestamp(
                            &file_path,
                            url.as_ref(),
                            None,
                            Some(Utc::now().timestamp() as u64),
                        )
                        .await
                        {
                            message(
                                &format!(
                                    "Pushed to {display_url} but failed to update tracking: {e}"
                                ),
                                Some("⚠️"),
                            );
                            file_errors += 1;
                        } else {
                            message(&format!("Successfully pushed to {display_url}"), Some("✅"));
                            file_successes += 1;
                        }
                    }
                    Err(e) => {
                        message(&format!("Failed to push to {remote_url}: {e}"), Some("❌"));
                        file_errors += 1;
                    }
                }
            }

            total_successes += file_successes;
            total_errors += file_errors;
            file_results.push((file_path.clone(), file_successes, file_errors));
        }

        // Display summary
        eprintln!(); // Empty line for spacing
        message(
            &format!(
                "Push complete: {} file(s) processed, {} push(es) succeeded, {} failed",
                file_results.len(),
                total_successes,
                total_errors
            ),
            Some("📊"),
        );

        // Show per-file summary
        for (file_path, successes, errors) in file_results {
            let status = if errors == 0 {
                "✅"
            } else if successes == 0 {
                "❌"
            } else {
                "⚠️"
            };
            message(
                &format!(
                    "  {}: {} succeeded, {} failed",
                    file_path.display(),
                    successes,
                    errors
                ),
                Some(status),
            );
        }

        if total_errors > 0 {
            exit(1)
        }

        Ok(())
    }
}
