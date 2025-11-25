use std::{
    collections::{BTreeMap, HashMap},
    env::current_dir,
    path::{Path, PathBuf},
    process::exit,
    str::FromStr,
};

use chrono::Utc;
use clap::Parser;
use eyre::{OptionExt, Result, bail, eyre};
use pathdiff::diff_paths;
use tokio::fs::remove_dir_all;
use url::Url;

use stencila_ask::{Answer, AskLevel, AskOptions, ask_with};
use stencila_cli_utils::{color_print::cstr, message};
use stencila_cloud::{WatchRequest, create_watch};
use stencila_codec_utils::{git_info, validate_file_on_default_branch};
use stencila_codecs::{PushDryRunOptions, PushResult};
use stencila_dirs::closest_workspace_dir;
use stencila_document::Document;
use stencila_remotes::{
    RemoteService, WatchDirection, WatchPrMode, expand_path_to_files, find_remote_for_arguments,
    get_remotes_for_path, get_tracked_remotes_for_path, update_remote_timestamp,
    update_spread_remote_timestamp,
};
use stencila_spread::{Run, SpreadConfig, SpreadMode, apply_template, infer_spread_mode};

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

    /// Do not save remote to stencila.toml
    ///
    /// By default, new remotes are added to stencila.toml so team members
    /// can push/pull the same remote. Use this flag to track locally only
    /// (in .stencila/remotes.json).
    #[arg(long)]
    no_config: bool,

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

    /// Enable spread push mode for multi-variant execution
    ///
    /// Spread mode allows pushing multiple variants of a document to separate
    /// remote documents, each with different parameter values. Supports:
    /// - grid: Cartesian product of all parameter values (default)
    /// - zip: Positional pairing of values (all must have same length)
    /// - cases: Explicit parameter sets via --case
    #[arg(long, value_name = "MODE", num_args = 0..=1, default_missing_value = "grid", conflicts_with = "watch")]
    spread: Option<stencila_spread::SpreadMode>,

    /// Explicit cases for spread=cases mode
    ///
    /// Each --case defines one variant with specific parameter values.
    /// Example: --case="region=north species=ABC"
    #[arg(long, value_name = "PARAMS", action = clap::ArgAction::Append)]
    case: Vec<String>,

    /// Title template for GDocs/M365 spread push
    ///
    /// Placeholders like {param} are replaced with parameter values.
    /// Example: --title="Report - {region}"
    #[arg(long, value_name = "TITLE")]
    title: Option<String>,

    /// Route template for site spread push
    ///
    /// Placeholders like {param} are replaced with parameter values.
    /// Routes always end with / and render as index.html.
    /// Example: --route="/{region}/{species}/"
    #[arg(long, value_name = "ROUTE")]
    route: Option<String>,

    /// Stop on first error instead of continuing with remaining variants
    #[arg(long)]
    fail_fast: bool,

    /// Maximum number of spread runs allowed (default: 100)
    #[arg(long, default_value = "100")]
    spread_max: usize,

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

  <dim># Spread push to GDocs (creates multiple docs)</dim>
  <b>stencila push</> <g>report.smd</> <c>--to</> <g>gdoc</> <c>--spread</> <c>--</> <c>region=north,south</>

  <dim># Spread push with custom title template</dim>
  <b>stencila push</> <g>report.smd</> <c>--to</> <g>gdoc</> <c>--spread</> <c>--title=\"Report - {region}\"</> <c>--</> <c>region=north,south</>

  <dim># Spread push with zip mode (positional pairing)</dim>
  <b>stencila push</> <g>report.smd</> <c>--to</> <g>gdoc</> <c>--spread=zip</> <c>--</> <c>region=north,south code=N,S</>

  <dim># Spread push dry run to preview operations</dim>
  <b>stencila push</> <g>report.smd</> <c>--to</> <g>gdoc</> <c>--spread</> <c>--dry-run</> <c>--</> <c>region=north,south</>

  <dim># Spread push to site with route template</dim>
  <b>stencila push</> <g>report.smd</> <c>--to</> <g>site</> <c>--spread</> <c>--route=\"/{region}/{species}/\"</> <c>--</> <c>region=north,south species=ABC,DEF</>
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
                let dir = PathBuf::from(dir_str);

                if dir.exists() {
                    let answer = ask_with(
                        &format!(
                            "Directory `{}` already exists. Clear it first?",
                            dir.display()
                        ),
                        AskOptions {
                            level: AskLevel::Warning,
                            default: Some(Answer::Yes),
                            ..Default::default()
                        },
                    )
                    .await?;

                    if answer.is_yes() {
                        remove_dir_all(&dir).await?;
                    }
                }

                Some(dir)
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
        let Some(ref path) = self.path else {
            return self.push_all().await;
        };

        // Display early dry-run notification
        if is_dry_run {
            message("⚠️ Performing dry-run, no remotes will actually be created or updated");
        }

        let path_display = path.display();

        // Validate input path exists
        if !path.exists() {
            bail!("Input path `{path_display}` does not exist");
        }

        // Check if path is a directory - if so, use directory push
        if path.is_dir() {
            return self.push_directory(path, dry_run_opts).await;
        }

        // Open the document
        let doc = Document::open(path, None).await?;

        // Infer spread mode if not explicitly set
        let mode = self.spread.or_else(|| {
            // If --case args provided, default to cases mode
            if !self.case.is_empty() {
                return Some(SpreadMode::Cases);
            }

            let arguments: Vec<(&str, &str)> = self
                .args
                .iter()
                .filter_map(|arg| arg.split_once('='))
                .collect();

            // Check --route template (for sites)
            if let Some(ref route) = self.route
                && let Some(mode) = infer_spread_mode(route, &arguments)
            {
                message!("ℹ️ Auto-detected spread mode `{mode}` from --route template");
                return Some(mode);
            }

            // Check --title template (for gdocs/m365)
            if let Some(ref title) = self.title
                && let Some(mode) = infer_spread_mode(title, &arguments)
            {
                message!("ℹ️ Auto-detected spread mode `{mode}` from --title template");
                return Some(mode);
            }

            None
        });

        // Handle spread push mode
        if let Some(mode) = mode {
            // Validate: --case is only valid with --spread=cases
            if !self.case.is_empty() && mode != SpreadMode::Cases {
                bail!("`--case` is only valid with `--spread=cases`, not `--spread={mode}`");
            }

            return self.push_spread(path, &doc, mode, dry_run_opts).await;
        }

        // Early validation: --watch is not compatible with multiple remotes
        if self.watch && self.to.is_none() {
            let remote_infos = get_remotes_for_path(path, None).await?;
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
        let (mut service, explicit_target, execution_args) = if let Some(target_str) = self.to {
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
            message!(
                "⚙️ Executing `{path_display}` before pushing it (use `--no-execute` to skip)"
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
            let remote_infos = get_remotes_for_path(path, None).await?;

            if remote_infos.is_empty() {
                // No explicit remotes configured - try smart fallback to Stencila Sites
                // Only if the file is within the configured site root
                let workspace_dir = closest_workspace_dir(path, false).await?;
                let config = stencila_config::config(&workspace_dir)?;

                let is_in_site_root = config.path_is_in_site_root(path, &workspace_dir);

                if is_in_site_root {
                    // Use Stencila Sites as the default service
                    service = Some(RemoteService::StencilaSites);
                    message!("ℹ️ Path `{path_display}` is in site root, using Stencila Sites");
                    // Service is now set, will fall through to single-remote push logic below
                } else {
                    bail!(
                        "No remotes configured for `{path_display}`. Specify a service push to: `--to <gdoc|m365|site>`",
                    );
                }
            }

            // If multiple remotes, push to all of them
            if remote_infos.len() > 1 {
                // Validate dry-run is not allowed with multiple remotes
                if is_dry_run {
                    bail!(
                        "Cannot use `--dry-run` when pushing to multiple remotes. Specify a target remote to use dry-run."
                    );
                }

                message!(
                    "☁️ Pushing `{path_display}` to {} configured remotes",
                    remote_infos.len()
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

                    message!(
                        "🔄 Updating {} linked to `{path_display}`",
                        remote_service.display_name()
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
                                path,
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
                                message!("✅ Successfully pushed to {display_url}");
                                successes.push(url);
                            }
                        }
                        Err(e) => {
                            message!("❌ Failed to push to {remote_url}: {e}");
                            errors.push((remote_url.clone(), e.to_string()));
                        }
                    }
                }

                // Display summary
                message!(
                    "📊 Push complete: {} succeeded, {} failed",
                    successes.len(),
                    errors.len()
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
            let remote_infos = get_remotes_for_path(path, None).await?;
            if remote_infos.is_empty() {
                bail!(
                    "No remotes configured for `{path_display}`. Add remotes to stencila.toml or specify a service (gdoc/m365/site) to push to.",
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
                message!(
                    "⚠️ Multiple {} remotes found:\n{urls_list}",
                    first_service.display_name_plural()
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
            let remote_infos = get_remotes_for_path(path, None).await?;
            remote_infos
                .iter()
                .find(|info| service.matches_url(&info.url))
                .map(|info| info.url.clone())
        };

        // Display appropriate message
        if existing_url.is_some() {
            message!(
                "🔄 Updating existing {} linked to `{path_display}`",
                service.display_name()
            );
        } else {
            message!("☁️ Creating new {}", service.display_name());
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
                message!("✅ Successfully pushed to {}", display_url);
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

                message!(
                    "📊 Dry-run complete. Would upload {} file(s), total size: {} bytes ({} compressed)",
                    files.len(),
                    total_size,
                    compressed_count
                );

                if let Some(dir) = output_dir {
                    message!("📁 Files written to: {}", dir.display());
                }

                // Display file list
                for file in &files {
                    let compressed_marker = if file.compressed { " (gzipped)" } else { "" };
                    let route_info = if let Some(route) = &file.route {
                        format!(" → {}", route)
                    } else {
                        String::new()
                    };
                    message!(
                        "   {}{}{} ({} bytes)",
                        file.storage_path,
                        compressed_marker,
                        route_info,
                        file.size
                    );
                }

                let display_url = get_display_url(&service, &url, doc.path());
                message!("🔗 Would be available at: {}", display_url);

                url
            }
        };

        if is_dry_run {
            // Skip tracking and watching if this is a dry run
            return Ok(());
        }

        // Track the remote (always use canonical URL for tracking)
        update_remote_timestamp(
            path,
            url.as_ref(),
            None,
            Some(Utc::now().timestamp() as u64),
        )
        .await?;

        // Save to config if this is a new remote (and not --no-config)
        if existing_url.is_none() && !self.no_config {
            if matches!(service, RemoteService::StencilaSites) {
                // For sites, save the route to [routes] section
                let route = url.path().to_string();
                match stencila_config::config_add_route(path, &route) {
                    Ok(config_path) => {
                        let config_path = current_dir()
                            .ok()
                            .and_then(|cwd| diff_paths(&config_path, cwd))
                            .unwrap_or_else(|| config_path.clone());
                        message!("📝 Route added to `{}`", config_path.display());
                    }
                    Err(error) => {
                        message!("⚠️ Could not add route to config: {error}");
                    }
                }
            } else {
                // For other services (gdocs, m365), save to [remotes] section
                match stencila_config::config_add_remote(path, url.as_ref()) {
                    Ok(config_path) => {
                        let config_path = current_dir()
                            .ok()
                            .and_then(|cwd| diff_paths(&config_path, cwd))
                            .unwrap_or_else(|| config_path.clone());
                        message!("📝 Remote added to `{}`", config_path.display());
                    }
                    Err(error) => {
                        message!("⚠️ Could not add to config: {error}");
                    }
                }
            }
        } else if existing_url.is_none() && self.no_config {
            message!("💾 Remote tracked locally (not saved to `stencila.toml`)");
        }

        // Enable watch if requested
        if self.watch {
            // Validate file exists on the default branch (also validates it's in a git repo)
            validate_file_on_default_branch(path)?;

            // Get git repository information
            let git_info = git_info(path)?;
            let repo_url = git_info
                .origin
                .ok_or_eyre("Repository has no origin remote")?;

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

            // Update stencila.toml with watch ID
            stencila_config::config_update_remote_watch(
                path,
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

            message!(
                "👁️ Watching `{path_display}` ({direction_desc}). PRs will be opened/updated on changes from {url_str}.",
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

        let Some(remotes) = &config.remotes else {
            bail!(
                "No remotes configured in `stencila.toml`. Add remotes to the config file to push."
            );
        };

        // Collect all files with their remotes
        let mut files_with_remotes: BTreeMap<PathBuf, Vec<Url>> = BTreeMap::new();

        for (path_key, value) in remotes {
            let config_path =
                stencila_config::ConfigRelativePath(path_key.clone()).resolve(&workspace_dir);

            // Expand path to actual files
            let files = expand_path_to_files(&config_path)?;

            // Process each target for this path (skip Spread targets which don't have URLs)
            for target in value.to_vec() {
                let Some(remote_url) = target.url_owned() else {
                    continue;
                };

                // Add to files map
                for file in &files {
                    files_with_remotes
                        .entry(file.clone())
                        .or_default()
                        .push(remote_url.clone());
                }
            }
        }

        if files_with_remotes.is_empty() {
            bail!(
                "No files found with configured remotes. Check that paths in stencila.toml exist."
            );
        }

        message!(
            "☁️ Pushing {} file(s) with configured remotes",
            files_with_remotes.len()
        );

        let mut total_successes = 0;
        let mut total_errors = 0;
        let mut file_results: Vec<(PathBuf, usize, usize)> = Vec::new();

        for (file_path, remote_urls) in files_with_remotes {
            let file_display = file_path.display();

            message!(
                "📄 Processing `{file_display}` ({} remote(s))",
                remote_urls.len()
            );

            // Open the document
            let doc = match Document::open(&file_path, None).await {
                Ok(d) => d,
                Err(e) => {
                    message!("❌ Failed to open `{file_display}`: {e}");
                    total_errors += remote_urls.len();
                    file_results.push((file_path.clone(), 0, remote_urls.len()));
                    continue;
                }
            };

            // Execute document if needed
            if !self.no_execute {
                message!("⚙️ Executing `{file_display}` before pushing");

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
                    message!("❌ Failed to execute `{file_display}`: {e}");
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
                        message!("⚠️ Skipping unsupported remote: {remote_url}");
                        file_errors += 1;
                        continue;
                    }
                };

                message!(
                    "🔄 Updating {} linked to `{file_display}`",
                    remote_service.display_name()
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
                            message!(
                                "⚠️ Pushed to {display_url} but failed to update tracking: {e}"
                            );
                            file_errors += 1;
                        } else {
                            message!("✅ Successfully pushed to {display_url}");
                            file_successes += 1;
                        }
                    }
                    Err(error) => {
                        message!("❌ Failed to push to {remote_url}: {error}");
                        file_errors += 1;
                    }
                }
            }

            total_successes += file_successes;
            total_errors += file_errors;
            file_results.push((file_path.clone(), file_successes, file_errors));
        }

        // Display summary
        message!(
            "📊 Push complete: {} file(s) processed, {} push(es) succeeded, {} failed",
            file_results.len(),
            total_successes,
            total_errors
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
            message!(
                "{status}   `{}`: {} succeeded, {} failed",
                file_path.display(),
                successes,
                errors
            );
        }

        if total_errors > 0 {
            exit(1)
        }

        Ok(())
    }

    /// Push a directory to a Stencila Site
    async fn push_directory(
        &self,
        path: &Path,
        dry_run_opts: Option<stencila_codecs::PushDryRunOptions>,
    ) -> Result<()> {
        use stencila_cloud::sites::ensure_site;
        use stencila_codec_site::PushProgress;

        let path_display = path.display();

        // Validate: --watch is not supported for directory push
        if self.watch {
            bail!("Watch is not supported for directory push. Sites are write-only remotes.");
        }

        // Ensure site configuration exists
        let (site_id, _) = ensure_site(path).await?;

        // Set up dry-run path
        let dry_run_path = dry_run_opts
            .as_ref()
            .and_then(|opts| opts.output_dir.as_ref());

        // Set up progress channel
        let (tx, mut rx) = tokio::sync::mpsc::channel::<PushProgress>(100);

        // Spawn a task to handle progress updates
        let progress_handle = tokio::spawn(async move {
            while let Some(progress) = rx.recv().await {
                match progress {
                    PushProgress::WalkingDirectory => {
                        message("📁 Walking directory");
                    }
                    PushProgress::FilesFound {
                        documents,
                        static_files,
                    } => {
                        message!("📊 Found {documents} documents, {static_files} static files");
                    }
                    PushProgress::EncodingDocument { path, index, total } => {
                        message!(
                            "📃 Processing document {}/{}: {}",
                            index + 1,
                            total,
                            path.display()
                        );
                    }
                    PushProgress::DocumentEncoded { .. } => {
                        //
                    }
                    PushProgress::DocumentFailed { path, error } => {
                        message!("❌ Failed to encode {}: {}", path.display(), error);
                    }
                    PushProgress::Processing {
                        processed,
                        uploaded,
                        total,
                    } => {
                        if processed == total {
                            let unchanged = total - uploaded;
                            message!(
                                "⚙️ Processed {total}/{total} files ({uploaded} new, {unchanged} unchanged)"
                            );
                        }
                    }
                    PushProgress::Reconciling => {
                        message("🔄 Reconciling files");
                    }
                    PushProgress::Complete(_) => {
                        // Summary is printed separately
                    }
                }
            }
        });

        message!("☁️ Pushing directory `{path_display}` to site `{site_id}`");

        // Determine dry-run state
        let is_dry_run = dry_run_opts.is_some();

        // Call push_directory with a decoder function
        let result = stencila_codec_site::push_directory(
            path,
            &site_id,
            None, // Use current branch
            self.force,
            is_dry_run,
            dry_run_path.map(|p| p.as_path()),
            Some(tx),
            |doc_path| async move { stencila_codecs::from_path(&doc_path, None).await },
        )
        .await;

        // Wait for progress handler to finish (tx is dropped by the block ending)
        let _ = progress_handle.await;

        // Handle result
        let result = result?;

        // Print summary
        let action = if is_dry_run {
            "Dry-run complete"
        } else {
            "Push complete"
        };

        message!(
            "✅ {}: {} documents, {} redirects, {} static files, {} media files",
            action,
            result.documents_ok.len(),
            result.redirects.len(),
            result.static_files_ok.len(),
            result.media_files_count
        );

        if result.media_duplicates_eliminated > 0 {
            message!(
                "♻️ {} media duplicates eliminated",
                result.media_duplicates_eliminated
            );
        }

        if result.files_skipped > 0 {
            message!(
                "⏭️ {} unchanged files skipped (use --force to upload all)",
                result.files_skipped
            );
        }

        if !result.documents_failed.is_empty() {
            message!("⚠️ {} documents failed:", result.documents_failed.len());
            for (path, error) in &result.documents_failed {
                message!("     - {}: {}", path.display(), error);
            }
        }

        if !is_dry_run {
            let url = format!("https://{site_id}.stencila.site");

            update_remote_timestamp(
                path,
                &url,
                None, // pulled_at unchanged
                Some(Utc::now().timestamp() as u64),
            )
            .await?;

            let url = Url::parse(&url)?;
            let url = stencila_codec_site::browseable_url(&url, Some(path))?;
            message!("🔗 Site available at: {url}");
        }
        Ok(())
    }

    /// Push document with spread parameters (multi-variant execution)
    async fn push_spread(
        &self,
        path: &Path,
        doc: &Document,
        mode: SpreadMode,
        dry_run_opts: Option<PushDryRunOptions>,
    ) -> Result<()> {
        let path_display = path.display();
        let is_dry_run = dry_run_opts.as_ref().is_some_and(|opts| opts.enabled);

        // Determine target service - from CLI --to or from config
        let service_from_cli = self
            .to
            .as_deref()
            .map(RemoteService::from_str)
            .transpose()?;

        // Parse arguments from CLI
        let cli_arguments: Vec<(&str, &str)> = self
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

        // Build spread config - either from CLI args or from [remotes] config
        // Returns (spread_config, title_template, route_template, service_from_config)
        let (config, title_template, route_template, service_from_config) = if cli_arguments
            .is_empty()
            && self.case.is_empty()
        {
            // Try to read from [remotes] config (look for Spread target)
            let workspace_dir = closest_workspace_dir(path, false).await?;
            let toml_config = stencila_config::config(&workspace_dir)?;

            let file_key = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            // First check [remotes] for a Spread target
            let spread_from_remotes = toml_config.remotes.as_ref().and_then(|remotes| {
                remotes.get(&file_key).and_then(|value| {
                    value
                        .to_vec()
                        .into_iter()
                        .find_map(|target| target.spread().cloned())
                })
            });

            if let Some(spread_config) = spread_from_remotes {
                // Build arguments from config params
                let config_args: Vec<(String, String)> = spread_config
                    .arguments
                    .iter()
                    .map(|(k, v)| (k.clone(), v.join(",")))
                    .collect();
                let config_args_refs: Vec<(&str, &str)> = config_args
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_str()))
                    .collect();

                let spread_mode = match spread_config.spread {
                    Some(stencila_config::SpreadMode::Zip) => SpreadMode::Zip,
                    _ => mode, // Use CLI mode or default to Grid
                };

                let cfg = SpreadConfig::from_arguments(
                    spread_mode,
                    &config_args_refs,
                    &[],
                    self.spread_max,
                )?;

                // Parse service from config
                let config_service = RemoteService::from_str(&spread_config.service)?;

                (
                    cfg,
                    spread_config.title.clone(),
                    None, // Route templates are now handled via [routes] section, not [remotes]
                    Some(config_service),
                )
            } else {
                bail!(
                    "No spread parameters provided. Use `-- param=val1,val2` or configure spread in [remotes] in stencila.toml"
                );
            }
        } else {
            let cfg =
                SpreadConfig::from_arguments(mode, &cli_arguments, &self.case, self.spread_max)?;
            (cfg, self.title.clone(), self.route.clone(), None)
        };

        // Determine final service: CLI takes precedence over config
        let service = service_from_cli
            .or(service_from_config)
            .ok_or_else(|| eyre!("Spread push requires a target service (--to gdoc/m365/site) or configure service in [remotes.spread]"))?;

        let run_count = config.validate()?;
        let runs = config.generate_runs()?;

        // For site spread pushes, auto-generate a route template if not provided
        // This ensures each variant gets a unique route
        // Priority: CLI --route > config route_template > auto-generate
        let effective_route_template = if matches!(service, RemoteService::StencilaSites) {
            if let Some(ref route) = self.route {
                // CLI --route takes precedence
                Some(route.clone())
            } else if let Some(ref route) = route_template {
                // Use route from config
                Some(route.clone())
            } else {
                // Auto-generate route template from base route + spread parameters
                let workspace_dir = closest_workspace_dir(path, false).await?;
                let site_config = stencila_config::config(&workspace_dir)?;
                let base_route =
                    stencila_codec_site::determine_route(path, &workspace_dir, &site_config)?;

                // Append all spread parameter names as placeholders
                let param_names: Vec<&String> = config.params.iter().map(|(k, _)| k).collect();
                if param_names.is_empty() {
                    Some(base_route)
                } else {
                    let params_path = param_names
                        .iter()
                        .map(|name| format!("{{{name}}}"))
                        .collect::<Vec<_>>()
                        .join("/");
                    let base = base_route.trim_end_matches('/');
                    Some(format!("{base}/{params_path}/"))
                }
            }
        } else {
            self.route.clone().or(route_template)
        };

        // Normalize route template to ensure it starts with '/'
        let effective_route_template = effective_route_template.map(|r| {
            if r.starts_with('/') {
                r
            } else {
                format!("/{r}")
            }
        });

        message!(
            "📊 Spread pushing `{path_display}` to {} ({} mode, {} variants)",
            service.display_name(),
            mode,
            run_count
        );

        // Get existing spread remotes from tracking file (not from config)
        // Spread variants are stored in .stencila/remotes.json, not in stencila.toml
        let tracked_remotes = get_tracked_remotes_for_path(path).await?;
        let existing_spread_remotes: Vec<_> = tracked_remotes
            .into_iter()
            .filter(|r| r.arguments.is_some() && service.matches_url(&r.url))
            .collect();

        // Calculate new vs update counts (using service-filtered remotes)
        let mut creates = Vec::new();
        let mut updates = Vec::new();
        for run in &runs {
            let run_args: HashMap<String, String> = run
                .values
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            if find_remote_for_arguments(&existing_spread_remotes, &run_args).is_some() {
                updates.push(run);
            } else {
                creates.push(run);
            }
        }

        // Dry run - process variants and generate files without uploading
        if is_dry_run {
            let has_output_dir = dry_run_opts
                .as_ref()
                .is_some_and(|opts| opts.output_dir.is_some());

            for (i, run) in runs.iter().enumerate() {
                let run_args: HashMap<String, String> = run
                    .values
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                let action =
                    if find_remote_for_arguments(&existing_spread_remotes, &run_args).is_some() {
                        "update"
                    } else {
                        "create"
                    };

                // Execute document with run parameters
                if !self.no_execute {
                    let run_arguments: Vec<(&str, &str)> = run
                        .values
                        .iter()
                        .map(|(k, v)| (k.as_str(), v.as_str()))
                        .collect();
                    doc.call(&run_arguments, stencila_document::ExecuteOptions::default())
                        .await?;
                }

                // Generate target description for display
                let target_desc = if let (RemoteService::StencilaSites, Some(template)) =
                    (&service, &effective_route_template)
                {
                    generate_route_from_template(template, run)?
                } else {
                    self.generate_title(doc.file_name(), run, title_template.as_deref())
                };

                message!(
                    "📤 Pushing {}/{}: {} → {} `{}`",
                    i + 1,
                    run_count,
                    run.to_terminal(),
                    action,
                    target_desc
                );

                // If output directory specified, generate the files
                if has_output_dir {
                    let generated_route = if let (RemoteService::StencilaSites, Some(template)) =
                        (&service, &effective_route_template)
                    {
                        Some(generate_route_from_template(template, run)?)
                    } else {
                        None
                    };

                    let title =
                        self.generate_title(doc.file_name(), run, title_template.as_deref());

                    // Push with dry-run options to generate files
                    let push_result = if let Some(route) = &generated_route {
                        stencila_codec_site::push_with_route(
                            &doc.root().await,
                            doc.path(),
                            route,
                            dry_run_opts.clone(),
                        )
                        .await
                    } else {
                        stencila_codecs::push(
                            &service,
                            &doc.root().await,
                            doc.path(),
                            Some(&title),
                            None,
                            doc.path(),
                            dry_run_opts.clone(),
                        )
                        .await
                    };

                    match push_result {
                        Ok(PushResult::DryRun { files, .. }) => {
                            for file in files {
                                message!("    → {}", file.storage_path);
                            }
                        }
                        Ok(_) => {}
                        Err(e) => {
                            message!("    ✗ Error generating: {}", e);
                        }
                    }
                }
            }

            message!("✅ Spread push dry-run complete: {run_count} variants previewed");
            return Ok(());
        }

        // Confirmation prompt if creating many new docs
        // Note: The global --yes flag is handled automatically by ask_with()
        let threshold = match service {
            RemoteService::GoogleDocs | RemoteService::Microsoft365 => 5,
            RemoteService::StencilaSites => 20,
        };
        if creates.len() > threshold {
            let answer = ask_with(
                &format!(
                    "This will create {} new {}. Continue?",
                    creates.len(),
                    service.display_name_plural()
                ),
                AskOptions {
                    level: AskLevel::Warning,
                    default: Some(Answer::No),
                    ..Default::default()
                },
            )
            .await?;

            if !answer.is_yes() {
                bail!("Aborted by user");
            }
        }

        // Process each run sequentially
        let total = runs.len();
        let mut successes: Vec<(HashMap<String, String>, Url)> = Vec::new();
        let mut errors: Vec<(HashMap<String, String>, String)> = Vec::new();
        let mut processed_arguments: Vec<HashMap<String, String>> = Vec::new();

        for (i, run) in runs.iter().enumerate() {
            let run_args: HashMap<String, String> = run
                .values
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            processed_arguments.push(run_args.clone());

            // Find existing remote for these arguments (scoped to current service)
            let existing_url = find_remote_for_arguments(&existing_spread_remotes, &run_args)
                .map(|r| r.url.clone());

            // Determine action (create or update)
            let action = if existing_url.is_some() {
                "update"
            } else {
                "create"
            };

            // Generate target description for display
            let target_desc = if let (RemoteService::StencilaSites, Some(template)) =
                (&service, &effective_route_template)
            {
                generate_route_from_template(template, run)?
            } else {
                self.generate_title(doc.file_name(), run, title_template.as_deref())
            };

            message!(
                "📤 Pushing {}/{}: {} → {} `{}`",
                i + 1,
                total,
                run.to_terminal(),
                action,
                target_desc
            );

            // Execute document with run parameters
            if !self.no_execute {
                let run_arguments: Vec<(&str, &str)> = run
                    .values
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_str()))
                    .collect();
                doc.call(&run_arguments, stencila_document::ExecuteOptions::default())
                    .await?;
            }

            // Generate title using template (for GDocs/M365)
            let title = self.generate_title(doc.file_name(), run, title_template.as_deref());

            // For sites with route template, generate the route
            let generated_route = if let (RemoteService::StencilaSites, Some(template)) =
                (&service, &effective_route_template)
            {
                Some(generate_route_from_template(template, run)?)
            } else {
                None
            };

            // Push to service - use push_with_route for sites with custom routes
            let push_result = if let Some(route) = &generated_route {
                stencila_codec_site::push_with_route(&doc.root().await, doc.path(), route, None)
                    .await
            } else {
                stencila_codecs::push(
                    &service,
                    &doc.root().await,
                    doc.path(),
                    Some(&title),
                    existing_url.as_ref(),
                    doc.path(),
                    None,
                )
                .await
            };

            match push_result {
                Ok(result) => {
                    let url = result.url();
                    let past_action = if existing_url.is_some() {
                        "📝 Updated"
                    } else {
                        "✨ Created"
                    };
                    let display_url = get_display_url(&service, &url, doc.path());
                    message!("{} {}", past_action, display_url);

                    // Track the remote with arguments
                    update_spread_remote_timestamp(
                        path,
                        url.as_ref(),
                        &run_args,
                        Utc::now().timestamp() as u64,
                    )
                    .await?;

                    successes.push((run_args, url));
                }
                Err(e) => {
                    message!("  ✗ Error: {}", e);
                    errors.push((run_args.clone(), e.to_string()));

                    // Stop on first error if --fail-fast
                    if self.fail_fast {
                        let skipped = total - i - 1;
                        message!(
                            "Spread push aborted: {} succeeded, {} failed, {} skipped",
                            successes.len(),
                            errors.len(),
                            skipped
                        );
                        bail!("Push failed for {:?}", run_args);
                    }
                }
            }
        }

        // Warn about orphaned remotes (existing but not in current spread)
        for existing in &existing_spread_remotes {
            if let Some(args) = &existing.arguments
                && !processed_arguments.iter().any(|a| a == args)
            {
                message!(
                    "⚠️ Orphaned remote not in current spread: {:?} → {}",
                    args,
                    existing.url
                );
            }
        }

        // Print summary
        message!(
            "📊 Spread push complete: {} succeeded, {} failed",
            successes.len(),
            errors.len()
        );

        if !errors.is_empty() {
            bail!("{} variants failed to push", errors.len());
        }

        // Save spread config to stencila.toml (unless --no-config)
        if !self.no_config {
            if matches!(service, RemoteService::StencilaSites) {
                // For sites, save to [routes] section with RouteSpread
                if let Some(ref route_template) = effective_route_template {
                    let route_spread = stencila_config::RouteSpread {
                        file: path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        spread: Some(match mode {
                            SpreadMode::Grid => stencila_config::SpreadMode::Grid,
                            SpreadMode::Zip => stencila_config::SpreadMode::Zip,
                            SpreadMode::Cases => stencila_config::SpreadMode::Grid,
                        }),
                        arguments: config
                            .params
                            .iter()
                            .map(|(k, v)| (k.clone(), v.0.clone()))
                            .collect(),
                    };

                    match stencila_config::config_set_route_spread(
                        path,
                        route_template,
                        &route_spread,
                    ) {
                        Ok(config_path) => {
                            let config_path = current_dir()
                                .ok()
                                .and_then(|cwd| diff_paths(&config_path, cwd))
                                .unwrap_or_else(|| config_path.clone());
                            message!(
                                "📝 Route spread config saved to `{}`",
                                config_path.display()
                            );
                        }
                        Err(error) => {
                            message!("⚠️ Could not save route spread config: {error}");
                        }
                    }
                }
            } else {
                // For other services (gdocs, m365), save to [remotes] section
                let spread_config = stencila_config::RemoteSpread {
                    service: service.cli_name().to_string(),
                    title: title_template,
                    spread: Some(match mode {
                        SpreadMode::Grid => stencila_config::SpreadMode::Grid,
                        SpreadMode::Zip => stencila_config::SpreadMode::Zip,
                        SpreadMode::Cases => stencila_config::SpreadMode::Grid,
                    }),
                    arguments: config
                        .params
                        .iter()
                        .map(|(k, v)| (k.clone(), v.0.clone()))
                        .collect(),
                };

                match stencila_config::config_set_remote_spread(path, &spread_config) {
                    Ok(config_path) => {
                        let config_path = current_dir()
                            .ok()
                            .and_then(|cwd| diff_paths(&config_path, cwd))
                            .unwrap_or_else(|| config_path.clone());
                        message!("📝 Spread config saved to `{}`", config_path.display());
                    }
                    Err(error) => {
                        message!("⚠️ Could not save spread config: {error}");
                    }
                }
            }
        }

        Ok(())
    }

    /// Generate title for a spread variant
    ///
    /// Uses the provided template if given, otherwise auto-generates from filename and params.
    fn generate_title(&self, filename: Option<&str>, run: &Run, template: Option<&str>) -> String {
        let base = filename.unwrap_or("Document");
        let base = base
            .strip_suffix(".smd")
            .or_else(|| base.strip_suffix(".md"))
            .unwrap_or(base);

        // Priority: CLI --title > config template > auto-generate
        let effective_template = self.title.as_deref().or(template);

        if let Some(tmpl) = effective_template {
            // Use provided template
            apply_template(tmpl, run).unwrap_or_else(|_| base.to_string())
        } else {
            // Auto-generate: "Filename - param1-param2"
            let variant_str = run
                .values
                .iter()
                .map(|(_, v)| v.as_str())
                .collect::<Vec<_>>()
                .join("-");
            format!("{} - {}", base, variant_str)
        }
    }
}

/// Generate route for a spread variant from a template
///
/// Applies the route template with run values.
fn generate_route_from_template(template: &str, run: &Run) -> Result<String> {
    let mut route = apply_template(template, run)?;

    // Ensure route ends with /
    if !route.ends_with('/') {
        route.push('/');
    }

    Ok(route)
}
