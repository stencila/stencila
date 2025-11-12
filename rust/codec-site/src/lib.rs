use std::{collections::HashMap, path::Path};

use eyre::{Result, bail, eyre};
use flate2::{Compression, write::GzEncoder};
use futures::future::join_all;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;
use url::Url;

use stencila_codec::{Codec, EncodeOptions, stencila_schema::Node};
use stencila_codec_dom::DomCodec;
use stencila_dirs::{closest_site_file, closest_stencila_dir, workspace_dir};
use stencila_node_media::{MediaFile, extract_and_collect_media};

/// Configuration for a Stencila Site
///
/// This is stored in `.stencila/site.yaml` in the project directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteConfig {
    /// Schema version for the site.yaml format
    pub version: u8,

    /// Unique site identifier (8-character Base62 string)
    #[serde(rename = "siteId")]
    pub site_id: String,

    /// Optional route overrides
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routes: Option<Vec<Route>>,
}

/// A route mapping or redirect rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    /// URL path (must start with `/`)
    pub path: String,

    /// File to serve at this path (relative to project root)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,

    /// Redirect destination (relative or absolute URL)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<String>,

    /// HTTP status code for redirect (default 302)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
}

/// Response from POST /sites/init
#[derive(Debug, Deserialize)]
struct InitResponse {
    #[serde(rename = "siteId")]
    site_id: String,
}

/// Response from GET /sites/{siteId}/status
#[derive(Debug, Deserialize)]
pub struct SiteStatusResponse {
    /// Map of file path to file status
    pub files: HashMap<String, FileStatus>,
}

/// Status information for a single file in the site
#[derive(Debug, Clone, Deserialize)]
pub struct FileStatus {
    /// Last modified timestamp (Unix timestamp)
    pub modified_at: u64,

    /// ETag for cache validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,
}

/// File status comparison result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatusCheck {
    /// File is up to date (no need to push)
    UpToDate,
    /// File is behind (needs to be pushed)
    Behind,
    /// File doesn't exist on site (needs to be pushed)
    NotFound,
}

/// Read the site configuration from `.stencila/site.yaml`
pub async fn read_site_config(path: &Path) -> Result<SiteConfig> {
    let config_path = closest_site_file(path, false).await?;

    if !config_path.exists() {
        bail!("Site configuration not found at {}", config_path.display());
    }

    let content = tokio::fs::read_to_string(&config_path).await?;
    let config: SiteConfig = serde_yaml::from_str(&content)?;

    Ok(config)
}

/// Write the site configuration to `.stencila/site.yaml`
async fn write_site_config(path: &Path, config: &SiteConfig) -> Result<()> {
    let config_path = closest_site_file(path, true).await?;

    let content = serde_yaml::to_string(config)?;
    tokio::fs::write(&config_path, content).await?;

    tracing::info!("Site configuration written to {}", config_path.display());
    Ok(())
}

/// Initialize a new site by requesting a site ID from the Cloud
async fn init_site(token: &str) -> Result<String> {
    let client = Client::new();
    let response = client
        .post(format!("{}/sites/init", stencila_cloud::base_url()))
        .bearer_auth(token)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        bail!("Failed to initialize site ({status}): {error_text}");
    }

    let init_response: InitResponse = response.json().await?;
    Ok(init_response.site_id)
}

/// Ensure a site configuration exists, creating it if necessary
async fn ensure_site_config(path: &Path) -> Result<SiteConfig> {
    let config_path = closest_site_file(path, false).await?;

    if config_path.exists() {
        // Configuration already exists, read it
        read_site_config(path).await
    } else {
        // Need to create new configuration
        tracing::info!("No site configuration found, initializing new site...");

        // Get API token
        let token = stencila_cloud::api_token()
            .ok_or_else(|| eyre!("No STENCILA_API_TOKEN environment variable or keychain entry found. Please set your API token."))?;

        // Request new site ID from Cloud
        let site_id = init_site(&token).await?;
        tracing::info!("Obtained new site ID: {}", site_id);

        // Create config with defaults
        let config = SiteConfig {
            version: 1,
            site_id,
            routes: None,
        };

        // Write to disk
        write_site_config(path, &config).await?;

        Ok(config)
    }
}

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
    project_dir: &Path,
    config: &SiteConfig,
) -> Result<String> {
    // Get path relative to project root
    let rel_path = file_path.strip_prefix(project_dir).map_err(|_| {
        eyre!(
            "File path {} is not within project directory {}",
            file_path.display(),
            project_dir.display()
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
                format!("/{}/", file_stem)
            } else {
                format!(
                    "/{}/{}/",
                    parent.to_string_lossy().replace('\\', "/"),
                    file_stem
                )
            }
        } else {
            format!("/{}/", file_stem)
        }
    };

    Ok(route)
}

/// Convert a URL route to a storage path for the HTML file
///
/// Routes always map to `index.html` files in their directory.
/// Examples:
/// - `/` → `index.html`
/// - `/report/` → `report/index.html`
/// - `/docs/analysis/` → `docs/analysis/index.html`
pub fn route_to_storage_path(route: &str) -> String {
    let trimmed = route.trim_start_matches('/').trim_end_matches('/');

    if trimmed.is_empty() {
        "index.html".to_string()
    } else {
        format!("{}/index.html", trimmed)
    }
}

/// Build HTML from a node using the DOM codec
async fn build_html(node: &Node, base_url: &str) -> Result<String> {
    let (html, _info) = DomCodec
        .to_string(
            node,
            Some(EncodeOptions {
                standalone: Some(true),
                base_url: Some(base_url.to_string()),
                // TODO: Allow theme customization
                ..Default::default()
            }),
        )
        .await?;
    Ok(html)
}

/// Compress data using gzip
fn compress_data(data: &[u8]) -> Result<Vec<u8>> {
    use std::io::Write;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    let compressed = encoder.finish()?;
    Ok(compressed)
}

/// Upload a single file to the site
async fn upload_file(site_id: &str, path: &str, content: Vec<u8>, token: &str) -> Result<()> {
    let client = Client::new();

    // Compress the content
    let compressed = compress_data(&content)?;

    let url = format!(
        "{}/sites/{}/latest/{}",
        stencila_cloud::base_url(),
        site_id,
        path
    );

    let response = client
        .put(&url)
        .bearer_auth(token)
        .header("Content-Encoding", "gzip")
        .header("Content-Type", "application/octet-stream")
        .body(compressed)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        bail!("Failed to upload {path} ({status}): {error_text}");
    }

    tracing::info!("Uploaded {} to site", path);
    Ok(())
}

/// Upload media files in parallel and return a map of content_url to site URL
async fn upload_media_parallel(
    site_id: &str,
    media_files: Vec<MediaFile>,
    token: &str,
    base_url: &str,
) -> Result<HashMap<String, String>> {
    // Deduplicate by hash
    let mut unique_files: HashMap<String, MediaFile> = HashMap::new();
    let mut url_map: HashMap<String, String> = HashMap::new();

    for media in media_files {
        let key = format!("{}.{}", media.hash, media.extension);

        // If we haven't seen this hash before, add it to unique files
        if !unique_files.contains_key(&key) {
            unique_files.insert(key.clone(), media.clone());
        }

        // Map original content_url to the site URL
        let site_url = format!("{}/media/{}", base_url, key);
        url_map.insert(media.content_url, site_url);
    }

    tracing::info!(
        "Uploading {} unique media files (deduplicated from {} total)",
        unique_files.len(),
        url_map.len()
    );

    // Upload all unique files in parallel
    let upload_futures: Vec<_> = unique_files
        .into_iter()
        .map(|(key, media)| {
            let token = token.to_string();
            let site_id = site_id.to_string();
            let path = format!("media/{}", key);

            async move {
                let content = tokio::fs::read(&media.path).await?;
                upload_file(&site_id, &path, content, &token).await
            }
        })
        .collect();

    let results = join_all(upload_futures).await;

    // Check for errors
    for result in results {
        result?;
    }

    Ok(url_map)
}

/// Rewrite media URLs in the node tree to point to uploaded site URLs
fn rewrite_media_urls(node: &mut Node, url_map: &HashMap<String, String>) {
    use stencila_codec::stencila_schema::{VisitorMut, WalkControl};

    struct UrlRewriter<'a> {
        url_map: &'a HashMap<String, String>,
    }

    impl UrlRewriter<'_> {
        /// Rewrite a single image's content URL
        fn rewrite_image(&self, img: &mut stencila_codec::stencila_schema::ImageObject) {
            if let Some(new_url) = self.url_map.get(&img.content_url) {
                img.content_url = new_url.clone();
            }
        }

        /// Rewrite an array of images (used in math/table options)
        fn rewrite_images(&self, images: &mut [stencila_codec::stencila_schema::ImageObject]) {
            for img in images {
                self.rewrite_image(img);
            }
        }
    }

    impl VisitorMut for UrlRewriter<'_> {
        fn visit_node(&mut self, node: &mut Node) -> WalkControl {
            match node {
                Node::ImageObject(img) => {
                    self.rewrite_image(img);
                }
                Node::AudioObject(audio) => {
                    if let Some(new_url) = self.url_map.get(&audio.content_url) {
                        audio.content_url = new_url.clone();
                    }
                }
                Node::VideoObject(video) => {
                    if let Some(new_url) = self.url_map.get(&video.content_url) {
                        video.content_url = new_url.clone();
                    }
                }
                Node::MathBlock(math) => {
                    if let Some(images) = &mut math.options.images {
                        self.rewrite_images(images);
                    }
                }
                Node::MathInline(math) => {
                    if let Some(images) = &mut math.options.images {
                        self.rewrite_images(images);
                    }
                }
                Node::Table(table) => {
                    if let Some(images) = &mut table.options.images {
                        self.rewrite_images(images);
                    }
                }
                _ => {}
            }
            WalkControl::Continue
        }
    }

    let mut rewriter = UrlRewriter { url_map };
    rewriter.walk(node);
}

/// Get the current status of files on the site from the Cloud API
pub async fn get_site_status(site_id: &str, token: &str) -> Result<Option<SiteStatusResponse>> {
    let client = Client::new();
    let response = client
        .get(format!(
            "{}/sites/{}/status",
            stencila_cloud::base_url(),
            site_id
        ))
        .bearer_auth(token)
        .send()
        .await?;

    if response.status().is_success() {
        let status = response.json::<SiteStatusResponse>().await?;
        Ok(Some(status))
    } else if response.status().as_u16() == 404 {
        // Site doesn't have status yet (new site or API not implemented)
        tracing::info!("No site status available (new site or endpoint not implemented)");
        Ok(None)
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        bail!("Failed to get site status ({status}): {error_text}");
    }
}

/// Check the status of a file by comparing local and remote modification times
pub fn check_file_status(
    storage_path: &str,
    local_modified: u64,
    site_status: &Option<SiteStatusResponse>,
) -> FileStatusCheck {
    match site_status {
        None => {
            // No site status available, assume file needs to be pushed
            FileStatusCheck::NotFound
        }
        Some(status) => {
            if let Some(remote_file) = status.files.get(storage_path) {
                // File exists on site, compare modification times
                if local_modified > remote_file.modified_at {
                    FileStatusCheck::Behind
                } else {
                    FileStatusCheck::UpToDate
                }
            } else {
                // File doesn't exist on site
                FileStatusCheck::NotFound
            }
        }
    }
}

/// Check if a document file should be pushed to a Stencila Site
///
/// This is a convenience function that combines all the steps needed to determine
/// if a file needs to be pushed: fetching site status, getting local file modification
/// time, determining the route, and comparing timestamps.
///
/// Returns `Ok(true)` if the file should be skipped (is up-to-date),
/// `Ok(false)` if it should be pushed, or `Err` if the status check failed.
pub async fn should_skip_push(
    doc_path: &Path,
    project_dir: &Path,
    config: &SiteConfig,
    token: &str,
) -> Result<bool> {
    // Get site status from Cloud API
    let site_status = get_site_status(&config.site_id, token).await?;

    // Get local file modification time
    let metadata = std::fs::metadata(doc_path)?;
    let modified = metadata.modified()?;
    let local_modified = modified.duration_since(std::time::UNIX_EPOCH)?.as_secs();

    // Determine route and storage path for this file
    let route = determine_route(doc_path, project_dir, config)?;
    let storage_path = route_to_storage_path(&route);

    // Check file status
    let status = check_file_status(&storage_path, local_modified, &site_status);

    // Return true (skip push) only if file is up-to-date
    Ok(matches!(status, FileStatusCheck::UpToDate))
}

/// Push a document to a Stencila Site
///
/// If `url` is provided, updates the existing document at that site.
/// Otherwise, creates or updates a document in the configured site.
///
/// Returns the URL of the published document on the site.
pub async fn push(
    node: &Node,
    title: Option<&str>,
    url: Option<&Url>,
    doc_path: Option<&Path>,
) -> Result<Url> {
    // Get API token
    let token = stencila_cloud::api_token()
        .ok_or_else(|| eyre!("No STENCILA_API_TOKEN environment variable or keychain entry found. Please set your API token."))?;

    // Find the workspace root directory
    let start_path = if let Some(path) = doc_path {
        path.to_path_buf()
    } else {
        std::env::current_dir()?
    };

    let stencila_dir = closest_stencila_dir(&start_path, true).await?;
    let project_dir = workspace_dir(&stencila_dir)?;

    tracing::info!("Using project root directory: {}", project_dir.display());

    // Ensure site configuration exists
    let config = ensure_site_config(&start_path).await?;
    let site_id = &config.site_id;

    // Build base URL for the site
    let base_url = if let Some(url) = url {
        // If URL provided, use its host as the base
        format!("{}://{}", url.scheme(), url.host_str().unwrap_or("unknown"))
    } else {
        // Default to the stencila.site subdomain
        format!("https://{}.stencila.site", site_id)
    };

    tracing::info!("Site URL: {}", base_url);

    // Create temporary directory for media extraction
    let temp_dir = TempDir::new()?;
    let media_dir = temp_dir.path().join("media");
    std::fs::create_dir_all(&media_dir)?;

    // Clone the node to avoid mutating the original
    let mut node_copy = node.clone();

    // Extract and collect media files
    let media_files = extract_and_collect_media(&mut node_copy, doc_path, &media_dir)?;

    tracing::info!("Collected {} media files", media_files.len());

    // Upload media files in parallel if any
    let url_map = if !media_files.is_empty() {
        upload_media_parallel(site_id, media_files, &token, &base_url).await?
    } else {
        HashMap::new()
    };

    // Rewrite media URLs in the node
    if !url_map.is_empty() {
        rewrite_media_urls(&mut node_copy, &url_map);
        tracing::info!("Rewrote {} media URLs", url_map.len());
    }

    // Build HTML
    tracing::info!("Building HTML...");
    let html = build_html(&node_copy, &base_url).await?;

    // Determine route based on doc_path or fallback to title-based heuristic
    let route = if let Some(path) = doc_path {
        determine_route(path, &project_dir, &config)?
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

    tracing::info!("Using route: {}", route);

    // Convert route to storage path
    let storage_path = route_to_storage_path(&route);

    // Upload HTML
    tracing::info!("Uploading HTML to {}...", storage_path);
    upload_file(site_id, &storage_path, html.into_bytes(), &token).await?;

    // Return the site URL with the route
    let site_url = format!("{}{}", base_url, route.trim_end_matches('/'));
    tracing::info!("Document published at: {}", site_url);

    Ok(Url::parse(&site_url)?)
}

/// Pull a document from a Stencila Site
///
/// **Note:** Pull is not supported for Stencila Sites. Sites are write-only remotes
/// since the source documents are maintained locally.
pub async fn pull(_url: &Url, _dest: &Path) -> Result<()> {
    bail!("Pull operation is not supported for Stencila Sites which are write-only remotes.")
}

/// Time that a Stencila Site was last modified as a Unix timestamp.
///
/// Queries the Cloud API for the last modification time of the site.
pub async fn modified_at(url: &Url) -> Result<u64> {
    // Extract site ID from URL
    let host = url
        .host_str()
        .ok_or_else(|| eyre!("Invalid site URL: no host"))?;

    // Expected format: {siteId}.stencila.site
    let site_id = host.strip_suffix(".stencila.site").ok_or_else(|| {
        eyre!("Invalid site URL: host should be in format {{siteId}}.stencila.site")
    })?;

    // Get API token
    let token = stencila_cloud::api_token().ok_or_else(|| {
        eyre!("No STENCILA_API_TOKEN environment variable or keychain entry found")
    })?;

    // Query the Cloud API for site status
    // TODO: Implement actual API endpoint once it's defined
    // For now, return current timestamp as a placeholder
    let client = Client::new();
    let response = client
        .get(format!(
            "{}/sites/{}/status",
            stencila_cloud::base_url(),
            site_id
        ))
        .bearer_auth(&token)
        .send()
        .await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            // Try to parse the response
            if let Ok(status) = resp.json::<SiteStatusResponse>().await {
                // Find the most recent modification time across all files
                let max_modified = status
                    .files
                    .values()
                    .map(|f| f.modified_at)
                    .max()
                    .unwrap_or(0);
                Ok(max_modified)
            } else {
                // If parsing fails, return 0
                Ok(0)
            }
        }
        _ => {
            // If the API endpoint doesn't exist yet or fails, return 0
            // This allows the function to work even before the Cloud API is fully implemented
            tracing::warn!("Could not fetch site status, returning 0 for modified_at");
            Ok(0)
        }
    }
}
