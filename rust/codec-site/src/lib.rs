use std::path::Path;

use eyre::{Result, bail, eyre};
use futures::future::try_join_all;
use tempfile::TempDir;
use url::Url;

use stencila_cloud::sites::{
    SiteConfig, ensure_site, last_modified, reconcile_prefix, upload_file,
};
use stencila_codec::{Codec, EncodeOptions, stencila_schema::Node};
use stencila_codec_dom::DomCodec;
use stencila_codec_utils::{get_current_branch, slugify_branch_name};
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
pub fn determine_route(
    file_path: &Path,
    workspace_dir: &Path,
    config: &SiteConfig,
) -> Result<String> {
    // Get path relative to project root
    let file_path = file_path.canonicalize()?;
    let rel_path = file_path.strip_prefix(workspace_dir).map_err(|_| {
        eyre!(
            "File path {} is not within project directory {}",
            file_path.display(),
            workspace_dir.display()
        )
    })?;

    // Normalize path separators to forward slashes for consistent comparison
    // (route overrides in site.yaml use forward slashes)
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
    let (site_config, _) = ensure_site(&start_path).await?;
    let site_id = &site_config.id;

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
        determine_route(path, &workspace_dir, &site_config)?
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
                upload_file(site_id, &branch_slug, storage_path, file_path)
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
    upload_file(site_id, &branch_slug, &storage_path, &temp_html).await?;

    // Reconcile media files at this route to clean up orphaned files
    reconcile_prefix(site_id, &branch_slug, &media_prefix, current_media_files).await?;

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
