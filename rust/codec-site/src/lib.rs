use std::path::Path;

use eyre::{Result, bail, eyre};
use futures::future::try_join_all;
use serde_json::json;
use tempfile::TempDir;
use url::Url;

use stencila_cloud::sites::{ensure_site, last_modified, reconcile_prefix, upload_file};
use stencila_codec::{Codec, EncodeOptions, stencila_schema::Node};
use stencila_codec_dom::DomCodec;
use stencila_codec_utils::{get_current_branch, slugify_branch_name};
use stencila_config::{Config, RouteConfig};
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

    // Check route overrides first
    if let Some(routes) = &config.routes {
        for route in routes {
            if let Some(file) = &route.file
                && rel_path_str == file.as_str()
            {
                return Ok(route.path.clone());
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
/// Returns the matching RouteConfig if found.
fn find_route_config(
    file_path: Option<&Path>,
    workspace_dir: &Path,
    config: &Config,
) -> Result<Option<RouteConfig>> {
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
        for route in routes {
            if let Some(file) = &route.file
                && rel_path_str == file.as_str()
            {
                return Ok(Some(route.clone()));
            }
        }
    }

    Ok(None)
}

/// Handle uploading a redirect route
///
/// Creates and uploads a special `.redirect` file that the Cloudflare Worker
/// can recognize and handle. The file contains JSON with the redirect URL
/// and HTTP status code.
async fn handle_redirect_route(
    site_id: &str,
    branch_slug: &str,
    route: &str,
    config: &RouteConfig,
) -> Result<()> {
    // Extract redirect URL and status code
    let redirect_url = config
        .redirect
        .as_ref()
        .ok_or_else(|| eyre!("Route has no redirect URL"))?;
    let status_code = config.status.unwrap_or(302); // Default to 302 temporary redirect

    // Validate status code is appropriate for redirects
    if ![301, 302, 303, 307, 308].contains(&status_code) {
        bail!(
            "Invalid redirect status code: {}. Must be 301, 302, 303, 307, or 308",
            status_code
        );
    }

    // Calculate storage path
    let trimmed = route.trim_start_matches('/').trim_end_matches('/');
    let storage_path = if trimmed.is_empty() {
        ".redirect".to_string()
    } else {
        format!("{trimmed}/.redirect")
    };

    // Create redirect file content (JSON format)
    let redirect_content = json!({
        "redirect": redirect_url,
        "status": status_code
    });

    // Write to temp file
    let temp_dir = TempDir::new()?;
    let temp_redirect = temp_dir.path().join("redirect");
    tokio::fs::write(&temp_redirect, redirect_content.to_string()).await?;

    // Upload redirect file
    upload_file(site_id, branch_slug, &storage_path, &temp_redirect).await?;

    tracing::info!(
        "Uploaded redirect from {} to {} (status {})",
        route,
        redirect_url,
        status_code
    );

    Ok(())
}

/// Push a document to a Stencila Site
///
/// If `url` is provided, updates the existing document at that site.
/// Otherwise, creates or updates a document in the configured site.
///
/// Returns the URL of the published document on the site.
pub async fn push(
    node: &Node,
    path: Option<&Path>,
    title: Option<&str>,
    url: Option<&Url>,
) -> Result<Url> {
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
        if let Some(route_config) = find_route_config(Some(path), &workspace_dir, &cfg)?
            && route_config.redirect.is_some()
        {
            // Handle redirect route - upload .redirect file and return early
            handle_redirect_route(&site_id, &branch_slug, &route, &route_config).await?;
            return Ok(Url::parse(&format!("{base_url}{route}"))?);
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
        let mut entries = tokio::fs::read_dir(&media_dir).await?;
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
            let upload_futures = media_files.iter().map(|(storage_path, file_path)| {
                upload_file(&site_id, &branch_slug, storage_path, file_path)
            });
            try_join_all(upload_futures).await?;
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

    // Upload HTML
    upload_file(&site_id, &branch_slug, &storage_path, &temp_html).await?;

    // Reconcile media files at this route to clean up orphaned files
    reconcile_prefix(&site_id, &branch_slug, &media_prefix, current_media_files).await?;

    // Return the site URL with the route
    Ok(Url::parse(&format!("{base_url}{route}"))?)
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
