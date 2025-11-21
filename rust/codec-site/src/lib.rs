use std::path::{Path, PathBuf};

use eyre::{Result, bail, eyre};
use futures::future::try_join_all;
use serde_json::json;
use tempfile::TempDir;
use tokio::fs::{copy, create_dir_all, metadata, read, read_dir, write};
use url::Url;

use stencila_cloud::sites::{ensure_site, last_modified, reconcile_prefix, upload_file};
use stencila_codec::{
    Codec, EncodeOptions, PushDryRunFile, PushDryRunOptions, PushResult, stencila_schema::Node,
};
use stencila_codec_dom::DomCodec;
use stencila_codec_utils::{get_current_branch, slugify_branch_name};
use stencila_config::{Config, RedirectStatus, RouteRedirect, RouteTarget};
use stencila_dirs::{closest_stencila_dir, workspace_dir};

/// Determine the URL route for a document file
///
/// First checks route overrides in site.yaml, then falls back to file-based routing.
///
/// # File-based routing rules:
/// - Extensions are stripped: `report.ipynb` → `/report/`
/// - Index files (`index.*`, `main.*`, `README.*`) → `/`
/// - Subdirectories are preserved: `docs/report.md` → `/docs/report/`
/// - All routes end with trailing slash
///
/// # Site root:
/// - If `config.site.root` is set, routes are calculated relative to that directory
/// - Otherwise, routes are relative to the workspace directory
pub fn determine_route(file_path: &Path, workspace_dir: &Path, config: &Config) -> Result<String> {
    // Determine the base directory for route calculation
    // If site.root is configured, resolve it relative to the workspace directory
    // (in the future, this could be enhanced to resolve relative to the config file)
    let base_dir = if let Some(site) = &config.site
        && let Some(root) = &site.root
    {
        root.resolve(workspace_dir)
    } else {
        workspace_dir.to_path_buf()
    };

    // Get path relative to base directory
    let file_path = file_path.canonicalize()?;
    let base_dir = base_dir.canonicalize()?;
    let rel_path = file_path.strip_prefix(&base_dir).map_err(|_| {
        eyre!(
            "File path {} is not within site root {}",
            file_path.display(),
            base_dir.display()
        )
    })?;

    // Normalize path separators to forward slashes for consistent comparison
    // (route overrides use forward slashes)
    let rel_path_str = rel_path.to_string_lossy().replace('\\', "/");

    // If the file path equals the base directory (site root), route to /
    if rel_path_str.is_empty() {
        return Ok("/".to_string());
    }

    // Check route overrides first
    if let Some(routes) = &config.routes {
        for (route_path, target) in routes {
            if let Some(file) = target.file()
                && rel_path_str == file.as_str()
            {
                return Ok(route_path.clone());
            }
        }
    }

    // Apply default file-based routing
    let file_stem = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| eyre!("Invalid file path: {}", file_path.display()))?;

    // Check if this is an index file
    let is_index = matches!(file_stem, "index" | "main" | "README");

    // Build the route
    let route = if is_index {
        // Index files map to their directory
        if let Some(parent) = rel_path.parent() {
            if parent == Path::new("") {
                "/".to_string()
            } else {
                format!("/{}/", parent.to_string_lossy().replace('\\', "/"))
            }
        } else {
            "/".to_string()
        }
    } else {
        // Regular files: strip extension, add trailing slash
        if let Some(parent) = rel_path.parent() {
            if parent == Path::new("") {
                format!("/{file_stem}/")
            } else {
                format!(
                    "/{}/{}/",
                    parent.to_string_lossy().replace('\\', "/"),
                    file_stem
                )
            }
        } else {
            format!("/{file_stem}/")
        }
    };

    Ok(route)
}

/// Convert a canonical Stencila Site URL to a browseable URL
///
/// For main/master branches, returns the canonical URL unchanged.
/// For other branches, prefixes the host with the branch slug (e.g., `feature--siteid.stencila.site`).
///
/// # Arguments
///
/// * `canonical` - The canonical site URL (e.g., `https://siteid.stencila.site/route/`)
/// * `path` - Optional file path to determine the git branch context
///
/// # Examples
///
/// ```
/// // On main branch:
/// // https://siteid.stencila.site/test/ -> https://siteid.stencila.site/test/
///
/// // On feature-foo branch:
/// // https://siteid.stencila.site/test/ -> https://feature-foo--siteid.stencila.site/test/
/// ```
pub fn browseable_url(canonical: &Url, path: Option<&Path>) -> Result<Url> {
    // Extract site_id from canonical URL host
    let host = canonical
        .host_str()
        .ok_or_else(|| eyre!("Invalid URL: no host"))?;

    // Get current branch
    let branch_name = get_current_branch(path).unwrap_or_else(|| "main".to_string());
    let branch_slug = slugify_branch_name(&branch_name);

    // For main/master, return canonical URL as-is
    if branch_slug == "main" || branch_slug == "master" {
        return Ok(canonical.clone());
    }

    // For other branches, add branch prefix to host
    let new_host = format!("{branch_slug}--{host}");
    let mut browseable = canonical.clone();
    browseable
        .set_host(Some(&new_host))
        .map_err(|_| eyre!("Failed to set host to {new_host}"))?;

    Ok(browseable)
}

/// Find the route configuration for a given file path
///
/// Searches through the config routes to find a matching route based on the file path.
/// Returns the matching route path and target if found.
fn find_route_config(
    file_path: Option<&Path>,
    workspace_dir: &Path,
    config: &Config,
) -> Result<Option<(String, RouteTarget)>> {
    let Some(path) = file_path else {
        return Ok(None);
    };

    let file_path = path.canonicalize()?;
    let base_dir = if let Some(site) = &config.site
        && let Some(root) = &site.root
    {
        root.resolve(workspace_dir)
    } else {
        workspace_dir.to_path_buf()
    };

    let rel_path = file_path.strip_prefix(&base_dir).map_err(|_| {
        eyre!(
            "File path {} is not within site root {}",
            file_path.display(),
            base_dir.display()
        )
    })?;

    let rel_path_str = rel_path.to_string_lossy().replace('\\', "/");

    if let Some(routes) = &config.routes {
        for (route_path, target) in routes {
            if let Some(file) = target.file()
                && rel_path_str == file.as_str()
            {
                return Ok(Some((route_path.clone(), target.clone())));
            }
        }
    }

    Ok(None)
}

/// Handle uploading a redirect route
///
/// Creates and uploads a special `redirect.json` file that the Cloudflare Worker
/// can recognize and handle. The file contains JSON with the redirect URL
/// and HTTP status code.
///
/// Returns information about the generated file if in dry-run mode.
async fn handle_redirect_route(
    site_id: &str,
    branch_slug: &str,
    route: &str,
    redirect: &RouteRedirect,
    is_dry_run: bool,
    dry_run_output_dir: &Option<PathBuf>,
) -> Result<Option<PushDryRunFile>> {
    // Extract redirect URL and status code
    let location = &redirect.redirect;
    let status_code = redirect.status.unwrap_or(RedirectStatus::Found); // Default to 302 temporary redirect

    const REDIRECT_FILENAME: &str = "redirect.json";

    // Calculate storage path
    let trimmed = route.trim_start_matches('/').trim_end_matches('/');
    let storage_path = if trimmed.is_empty() {
        REDIRECT_FILENAME.to_string()
    } else {
        format!("{trimmed}/{REDIRECT_FILENAME}")
    };

    // Create redirect file content (JSON format)
    let redirect_content = serde_json::to_string(&json!({
        "location": location,
        "status": status_code
    }))?;

    // Write to temp file
    let temp_dir = TempDir::new()?;
    let temp_redirect = temp_dir.path().join(REDIRECT_FILENAME);
    write(&temp_redirect, &redirect_content).await?;

    let full_storage_path = format!("{site_id}/{branch_slug}/{storage_path}");
    let file_size = redirect_content.len() as u64;

    if is_dry_run {
        // Dry-run mode: write to local directory if specified
        let local_path = if let Some(output_dir) = dry_run_output_dir {
            let dest_path = output_dir.join(&full_storage_path);
            create_dir_all(dest_path.parent().unwrap()).await?;
            copy(&temp_redirect, &dest_path).await?;
            Some(dest_path)
        } else {
            None
        };

        tracing::info!(
            "Dry-run: would upload redirect from {} to {} (status {})",
            route,
            location,
            status_code
        );

        Ok(Some(PushDryRunFile {
            storage_path: full_storage_path,
            local_path,
            size: file_size,
            compressed: false,
            route: Some(route.to_string()),
        }))
    } else {
        // Normal mode: upload to R2
        upload_file(site_id, branch_slug, &storage_path, &temp_redirect).await?;

        tracing::info!(
            "Uploaded redirect from {} to {} (status {})",
            route,
            location,
            status_code
        );

        Ok(None)
    }
}

/// Push a document to a Stencila Site
///
/// If `url` is provided, updates the existing document at that site.
/// Otherwise, creates or updates a document in the configured site.
///
/// If `dry_run` is provided, files are generated but not uploaded.
///
/// Returns the result of the push operation.
pub async fn push(
    node: &Node,
    path: Option<&Path>,
    title: Option<&str>,
    url: Option<&Url>,
    dry_run: Option<PushDryRunOptions>,
) -> Result<PushResult> {
    // Find the workspace root directory
    let start_path = if let Some(path) = path {
        path.to_path_buf()
    } else {
        std::env::current_dir()?
    };

    let stencila_dir = closest_stencila_dir(&start_path, true).await?;
    let workspace_dir = workspace_dir(&stencila_dir)?;

    // Ensure site configuration exists
    let (site_id, _) = ensure_site(&start_path).await?;

    // Initialize dry-run tracking
    let mut dry_run_files = Vec::new();
    let is_dry_run = dry_run.as_ref().is_some_and(|opts| opts.enabled);
    let dry_run_output_dir = dry_run.as_ref().and_then(|opts| opts.output_dir.clone());

    // Get branch slug
    let branch_name = get_current_branch(Some(&start_path)).unwrap_or_else(|| "main".to_string());
    let branch_slug = slugify_branch_name(&branch_name);

    // Build base URL for the site
    let base_url = if let Some(url) = url {
        // If URL provided, use its host as the base
        format!("{}://{}", url.scheme(), url.host_str().unwrap_or("unknown"))
    } else {
        // Default to the stencila.site subdomain
        format!("https://{site_id}.stencila.site")
    };

    // Create temporary directory for HTML and extracted and collected media
    let temp_dir = TempDir::new()?;
    let temp_html = temp_dir.path().join("temp.html");
    let media_dir = temp_dir.path().join("media");

    // Encode HTML
    DomCodec
        .to_path(
            node,
            &temp_html,
            Some(EncodeOptions {
                standalone: Some(true),
                base_url: Some(base_url.to_string()),
                from_path: path.map(|path| path.to_path_buf()),
                to_path: Some(temp_html.clone()),
                extract_media: Some(media_dir.clone()),
                collect_media: Some(media_dir.clone()),
                // TODO: Allow theme and view customization
                view: Some("static".into()),
                ..Default::default()
            }),
        )
        .await?;

    // Determine route based on doc_path or fallback to title-based heuristic
    let route = if let Some(path) = path {
        // Get config to check for route overrides and site.root
        let cfg = stencila_config::config(&workspace_dir)?;
        determine_route(path, &workspace_dir, &cfg)?
    } else if let Some(title) = title {
        // Fallback: use a simple heuristic based on title
        let cleaned = title.trim_end_matches(|c: char| !c.is_alphanumeric());
        if cleaned.is_empty() {
            "/document/".to_string()
        } else {
            format!("/{}/", cleaned.to_lowercase().replace(' ', "-"))
        }
    } else {
        "/document/".to_string()
    };

    // Check if this route has a redirect configured
    if let Some(path) = path {
        let cfg = stencila_config::config(&workspace_dir)?;
        if let Some((route_path, target)) = find_route_config(Some(path), &workspace_dir, &cfg)?
            && let Some(redirect) = target.redirect()
        {
            // Handle redirect route - upload .redirect file and return early
            let dry_run_file = handle_redirect_route(
                &site_id,
                &branch_slug,
                &route_path,
                redirect,
                is_dry_run,
                &dry_run_output_dir,
            )
            .await?;

            if let Some(file) = dry_run_file {
                dry_run_files.push(file);
            }

            let url = Url::parse(&format!("{base_url}{route_path}"))?;

            return if is_dry_run {
                Ok(PushResult::DryRun {
                    url,
                    files: dry_run_files,
                    output_dir: dry_run_output_dir,
                })
            } else {
                Ok(PushResult::Uploaded(url))
            };
        }
    }

    // Calculate media prefix for this route
    let trimmed = route.trim_start_matches('/').trim_end_matches('/');
    let media_prefix = if trimmed.is_empty() {
        "media/".to_string()
    } else {
        format!("{trimmed}/media/")
    };

    // Upload media files in parallel and track filenames for reconciliation
    let mut current_media_files = Vec::new();
    if media_dir.exists() {
        let mut entries = read_dir(&media_dir).await?;
        let mut media_files = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file()
                && let Some(filename) = path.file_name().and_then(|n| n.to_str())
            {
                current_media_files.push(filename.to_string());
                let storage_path = format!("{media_prefix}{filename}");
                media_files.push((storage_path, path));
            }
        }

        if !media_files.is_empty() {
            if is_dry_run {
                // Dry-run mode: write media files to local directory if specified
                for (storage_path, file_path) in &media_files {
                    let full_storage_path = format!("{site_id}/{branch_slug}/{storage_path}");
                    let metadata = metadata(file_path).await?;

                    let local_path = if let Some(output_dir) = &dry_run_output_dir {
                        let dest_path = output_dir.join(&full_storage_path);
                        create_dir_all(dest_path.parent().unwrap()).await?;
                        copy(file_path, &dest_path).await?;
                        Some(dest_path)
                    } else {
                        None
                    };

                    dry_run_files.push(PushDryRunFile {
                        storage_path: full_storage_path,
                        local_path,
                        size: metadata.len(),
                        compressed: false,
                        route: None,
                    });
                }
            } else {
                // Normal mode: upload to R2
                let upload_futures = media_files.iter().map(|(storage_path, file_path)| {
                    upload_file(&site_id, &branch_slug, storage_path, file_path)
                });
                try_join_all(upload_futures).await?;
            }
        }
    }

    // Convert route to storage path. Routes always map to `index.html` files in their directory.
    //
    // Examples:
    // - `/` → `index.html`
    // - `/report/` → `report/index.html`
    // - `/docs/analysis/` → `docs/analysis/index.html`
    let storage_path = if trimmed.is_empty() {
        "index.html".to_string()
    } else {
        format!("{trimmed}/index.html")
    };

    // Upload HTML (with compression)
    let html_metadata = metadata(&temp_html).await?;
    let full_html_path = format!("{site_id}/{branch_slug}/{storage_path}.gz");

    if is_dry_run {
        // Dry-run mode: write HTML file to local directory if specified
        // Note: In actual upload, HTML is gzipped, so we simulate that here
        let local_path = if let Some(output_dir) = &dry_run_output_dir {
            let dest_path = output_dir.join(&full_html_path);
            create_dir_all(dest_path.parent().unwrap()).await?;

            // Compress HTML before writing (matching actual upload behavior)
            use flate2::Compression;
            use flate2::write::GzEncoder;
            use std::io::Write;

            let html_content = read(&temp_html).await?;
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&html_content)?;
            let compressed = encoder.finish()?;

            write(&dest_path, &compressed).await?;
            Some(dest_path)
        } else {
            None
        };

        dry_run_files.push(PushDryRunFile {
            storage_path: full_html_path,
            local_path,
            size: html_metadata.len(),
            compressed: true,
            route: Some(route.clone()),
        });

        // Skip reconciliation in dry-run mode
    } else {
        // Normal mode: upload to R2
        upload_file(&site_id, &branch_slug, &storage_path, &temp_html).await?;

        // Reconcile media files at this route to clean up orphaned files
        reconcile_prefix(&site_id, &branch_slug, &media_prefix, current_media_files).await?;
    }

    // Process standalone redirect routes from config
    // Only upload redirects when pushing the site root directory itself (not individual files within it)
    if let Some(path) = path {
        let cfg = stencila_config::config(&workspace_dir)?;

        // Check if the path is exactly the site root directory
        let is_site_root_dir = if let Some(site) = &cfg.site {
            if let Some(root) = &site.root {
                let site_root_path = root.resolve(&workspace_dir);
                if let (Ok(path_canon), Ok(site_canon)) =
                    (path.canonicalize(), site_root_path.canonicalize())
                {
                    path_canon == site_canon
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if is_site_root_dir
            && let Some(routes) = &cfg.routes {
                for (route_path, target) in routes {
                    if let Some(redirect) = target.redirect() {
                        let dry_run_file = handle_redirect_route(
                            &site_id,
                            &branch_slug,
                            route_path,
                            redirect,
                            is_dry_run,
                            &dry_run_output_dir,
                        )
                        .await?;

                        if let Some(file) = dry_run_file {
                            dry_run_files.push(file);
                        }
                    }
                }
            }
    }

    // Return the result
    let url = Url::parse(&format!("{base_url}{route}"))?;

    if is_dry_run {
        Ok(PushResult::DryRun {
            url,
            files: dry_run_files,
            output_dir: dry_run_output_dir,
        })
    } else {
        Ok(PushResult::Uploaded(url))
    }
}

/// Pull a document from a Stencila Site
///
/// **Note:** Pull is not supported for Stencila Sites. Sites are write-only remotes
/// since the source documents are maintained locally.
pub async fn pull(_url: &Url, _dest: &Path) -> Result<()> {
    bail!("Pull operation is not supported for Stencila Sites which are write-only remotes.")
}

/// Time that a Stencila Site was last modified as a Unix timestamp.
pub async fn modified_at(url: &Url) -> Result<u64> {
    last_modified(url).await
}
