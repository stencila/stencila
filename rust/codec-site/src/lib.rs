use std::{
    collections::{HashMap, HashSet},
    io::Write as IoWrite,
    path::{Path, PathBuf},
};

use md5::{Digest, Md5};

use eyre::{Result, bail, eyre};
use flate2::{Compression, write::GzEncoder};
use futures::future::try_join_all;
use ignore::WalkBuilder;
use serde_json::json;
use tempfile::TempDir;
use tokio::fs::{copy, create_dir_all, metadata, read, read_dir, write};
use url::Url;

use stencila_cloud::sites::{ensure_site, get_etags, last_modified, reconcile_prefix, upload_file};
use stencila_codec::{
    Codec, EncodeOptions, PushDryRunFile, PushDryRunOptions, PushResult, stencila_schema::Node,
};
use stencila_codec_dom::DomCodec;
use stencila_codec_utils::{get_current_branch, git_info, slugify_branch_name};
use stencila_config::{Config, RedirectStatus, RouteRedirect, RouteTarget};
use stencila_dirs::{closest_stencila_dir, workspace_dir};
use stencila_format::Format;

// ============================================================================
// Types for directory push
// ============================================================================

/// Category of a file for directory push
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileCategory {
    /// A document that should be decoded and encoded to HTML
    Document,
    /// A media file (image/audio/video) - standalone, treat as static
    Media,
    /// A static asset (CSS, JS, fonts, etc.) - copy as-is
    Static,
}

/// A document encoded and ready for upload (but not yet uploaded)
#[derive(Debug)]
pub struct EncodedDocument {
    /// The source file path
    pub source_path: PathBuf,
    /// The computed route (e.g., "/report/")
    pub route: String,
    /// The storage path for HTML (e.g., "report/index.html")
    pub html_storage_path: String,
    /// The HTML content (not yet compressed)
    pub html_content: Vec<u8>,
    /// Media files collected for this document: (filename, file_path)
    /// Note: The filename already contains the SeaHash (e.g., "a1b2c3d4.png")
    /// as computed by node-media's Collector (see collect.rs:164)
    /// These paths point into the shared media directory.
    pub media_files: Vec<(String, PathBuf)>,
}

/// Result of encoding a single document
#[derive(Debug)]
pub enum EncodeResult {
    /// Document was encoded to HTML
    Document(EncodedDocument),
    /// Document has a redirect configured - no HTML generated
    Redirect {
        /// The route path
        route: String,
        /// The redirect target URL
        target: String,
        /// The redirect status code
        status: RedirectStatus,
    },
}

/// Result of pushing a directory
#[derive(Debug, Clone)]
pub struct DirectoryPushResult {
    /// Documents that were successfully processed: (source_path, route)
    pub documents_ok: Vec<(PathBuf, String)>,
    /// Documents that failed to process: (source_path, error_message)
    pub documents_failed: Vec<(PathBuf, String)>,
    /// Redirects that were uploaded: (route, target)
    pub redirects: Vec<(String, String)>,
    /// Static files that were uploaded
    pub static_files_ok: Vec<PathBuf>,
    /// Static files that failed: (path, error_message)
    pub static_files_failed: Vec<(PathBuf, String)>,
    /// Total unique media files uploaded (after deduplication)
    pub media_files_count: usize,
    /// Number of media file duplicates eliminated
    pub media_duplicates_eliminated: usize,
    /// Number of files skipped because content unchanged (ETag match)
    pub files_skipped: usize,
}

/// Progress events emitted during directory push
#[derive(Debug, Clone)]
pub enum PushProgress {
    /// Starting to walk the directory
    WalkingDirectory,
    /// Found files to process
    FilesFound {
        documents: usize,
        static_files: usize,
    },
    /// Encoding a document
    EncodingDocument {
        path: PathBuf,
        index: usize,
        total: usize,
    },
    /// Document encoded successfully
    DocumentEncoded { path: PathBuf, route: String },
    /// Document encoding failed (continues with next)
    DocumentFailed { path: PathBuf, error: String },
    /// Processing files (uploading or skipping unchanged)
    Processing {
        processed: usize,
        uploaded: usize,
        total: usize,
    },
    /// Reconciling files
    Reconciling,
    /// Push complete
    Complete(DirectoryPushResult),
}

// ============================================================================
// Helper functions
// ============================================================================

/// Categorize a file for directory push
///
/// Uses existing `Format` infrastructure to categorize files.
pub fn categorize_file(path: &Path) -> FileCategory {
    let format = Format::from_path(path);

    // Quick checks for common static assets
    if matches!(format, Format::Css | Format::JavaScript) {
        return FileCategory::Static;
    }

    // Media files are treated as static assets when found at top level
    // (embedded media in documents is handled by the media collector)
    if format.is_media() {
        return FileCategory::Media;
    }

    // Check if this is a document format that we can decode
    // These are the formats that have codecs supporting decode
    if matches!(
        format,
        // Markup formats
        Format::Html
            | Format::Jats
            // Markdown flavors
            | Format::Markdown
            | Format::Smd
            | Format::Qmd
            | Format::Myst
            | Format::Llmd
            // Typesetting
            | Format::Latex
            | Format::Rnw
            // Notebook formats
            | Format::Ipynb
            // Word processor formats
            | Format::Docx
            | Format::GDocx
            | Format::Odt
            // Data serialization formats (lossless)
            | Format::Json
            | Format::Json5
            | Format::JsonLd
            | Format::Cbor
            | Format::CborZstd
            | Format::Yaml
            // Tabular data
            | Format::Csv
            // Spreadsheets
            | Format::Xlsx
            // Other
            | Format::Lexical
            | Format::Directory
            | Format::Swb
            | Format::Meca
            | Format::PmcOa
    ) {
        return FileCategory::Document;
    }

    // Everything else is a static asset (fonts, data files, etc.)
    FileCategory::Static
}

/// Extract the prefix (directory path) from a storage path.
///
/// # Examples
/// - `"report/index.html"` -> `"report/"`
/// - `"index.html"` -> `""`
/// - `"assets/css/style.css"` -> `"assets/css/"`
pub fn extract_prefix(storage_path: &str) -> String {
    match storage_path.rfind('/') {
        Some(pos) => format!("{}/", &storage_path[..pos]),
        None => String::new(),
    }
}

/// Extract the filename from a storage path.
///
/// # Examples
/// - `"report/index.html"` -> `"index.html"`
/// - `"index.html"` -> `"index.html"`
pub fn extract_filename(storage_path: &str) -> String {
    storage_path
        .rsplit('/')
        .next()
        .unwrap_or(storage_path)
        .to_string()
}

/// Normalize a path to use forward slashes for cloud storage keys.
///
/// On Windows, `Path::to_string_lossy()` produces backslashes which are
/// invalid for cloud storage keys and break URL routing.
///
/// # Examples
/// - `"assets/style.css"` -> `"assets/style.css"` (unchanged on Unix)
/// - `"assets\\style.css"` -> `"assets/style.css"` (normalized on Windows)
pub fn normalize_storage_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Calculate an ETag for content (MD5 hash in quoted hex format).
///
/// This must match how Stencila Cloud calculates ETags for uploaded files.
/// Format: `"<hex-encoded-md5>"` (quotes included as per HTTP ETag spec)
pub fn calculate_etag(content: &[u8]) -> String {
    let hash = Md5::digest(content);
    format!("\"{:x}\"", hash)
}

/// Convert a route to redirect storage path
///
/// # Examples
/// - `"/"` -> `"redirect.json"`
/// - `"/old-page/"` -> `"old-page/redirect.json"`
/// - `"/docs/old/"` -> `"docs/old/redirect.json"`
pub fn route_to_redirect_storage_path(route: &str) -> String {
    let trimmed = route.trim_matches('/');
    if trimmed.is_empty() {
        "redirect.json".to_string()
    } else {
        format!("{trimmed}/redirect.json")
    }
}

// ============================================================================
// Route and URL functions
// ============================================================================

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

/// A spread route variant ready for pushing
#[derive(Debug, Clone)]
pub struct SpreadRouteVariant {
    /// The source file path
    pub file: String,
    /// The generated route (with placeholders filled in)
    pub route: String,
    /// The arguments for this variant
    pub arguments: HashMap<String, String>,
}

/// Generate all spread route variants from config
///
/// Reads `[routes]` from config and expands all `RouteTarget::Spread` variants.
/// Returns a list of (route_template, file, variants) tuples.
pub fn generate_spread_routes(config: &Config) -> Result<Vec<SpreadRouteVariant>> {
    let Some(routes) = &config.routes else {
        return Ok(Vec::new());
    };

    let mut variants = Vec::new();

    for (route_template, target) in routes {
        let Some(spread) = target.spread() else {
            continue;
        };

        let file = &spread.file;
        let mode = spread.spread.unwrap_or_default();

        // Build runs from arguments
        let runs = generate_spread_runs(mode, &spread.arguments)?;

        for run in runs {
            // Apply template to generate route
            let route = apply_spread_template(route_template, &run)?;

            variants.push(SpreadRouteVariant {
                file: file.clone(),
                route,
                arguments: run,
            });
        }
    }

    Ok(variants)
}

/// Generate spread runs from params based on mode
fn generate_spread_runs(
    mode: stencila_config::SpreadMode,
    params: &HashMap<String, Vec<String>>,
) -> Result<Vec<HashMap<String, String>>> {
    if params.is_empty() {
        return Ok(Vec::new());
    }

    let keys: Vec<&String> = params.keys().collect();
    let values: Vec<&Vec<String>> = keys.iter().map(|k| &params[*k]).collect();

    match mode {
        stencila_config::SpreadMode::Grid => {
            // Cartesian product
            let mut runs = vec![HashMap::new()];
            for (key, vals) in keys.iter().zip(values.iter()) {
                let mut new_runs = Vec::new();
                for run in &runs {
                    for val in *vals {
                        let mut new_run = run.clone();
                        new_run.insert((*key).clone(), val.clone());
                        new_runs.push(new_run);
                    }
                }
                runs = new_runs;
            }
            Ok(runs)
        }
        stencila_config::SpreadMode::Zip => {
            // Positional pairing - all must have same length
            let len = values.first().map(|v| v.len()).unwrap_or(0);
            for vals in &values {
                if vals.len() != len {
                    bail!("Zip mode requires all parameters to have the same number of values");
                }
            }

            let mut runs = Vec::new();
            for i in 0..len {
                let mut run = HashMap::new();
                for (key, vals) in keys.iter().zip(values.iter()) {
                    run.insert((*key).clone(), vals[i].clone());
                }
                runs.push(run);
            }
            Ok(runs)
        }
    }
}

/// Apply a template by replacing {placeholder} with values
fn apply_spread_template(template: &str, values: &HashMap<String, String>) -> Result<String> {
    let mut result = template.to_string();
    for (key, value) in values {
        let placeholder = format!("{{{}}}", key);
        result = result.replace(&placeholder, value);
    }

    // Check for unresolved placeholders
    if result.contains('{') && result.contains('}') {
        bail!(
            "Template '{}' has unresolved placeholders after applying values: {:?}",
            template,
            values
        );
    }

    // Ensure route ends with /
    if !result.ends_with('/') {
        result.push('/');
    }

    Ok(result)
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
            if let Some(parent) = dest_path.parent() {
                create_dir_all(parent).await?;
            }
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

// ============================================================================
// Document encoding
// ============================================================================

/// Encode a document to HTML without uploading.
///
/// This function extracts the encoding logic from `push()` to allow:
/// 1. Encoding multiple documents with a shared media directory (for deduplication)
/// 2. Generating HTML with correct media paths upfront (no post-hoc rewriting needed)
///
/// # Arguments
/// * `node` - The decoded document node
/// * `path` - Original source file path (for route determination)
/// * `workspace_dir` - The workspace root directory
/// * `config` - Site configuration
/// * `base_url` - Base URL for the site (e.g., "https://mysite.stencila.site")
/// * `site_temp_root` - Temporary directory that mirrors the site structure. Media is placed
///   at `{site_temp_root}/media/` and HTML at `{site_temp_root}/{route}/index.html` so that
///   relative paths in the generated HTML correctly point to `/media/{hash}.ext`.
///
/// # Returns
/// The encoded document with HTML content and list of media files collected,
/// or a Redirect if the document's route is configured as a redirect.
pub async fn encode_document(
    node: &Node,
    path: Option<&Path>,
    workspace_dir: &Path,
    config: &Config,
    base_url: &str,
    site_temp_root: &Path,
) -> Result<EncodeResult> {
    // Determine route
    let route = if let Some(p) = path {
        determine_route(p, workspace_dir, config)?
    } else {
        "/document/".to_string()
    };

    // Check if this route has a redirect configured
    if let Some((route_path, target)) = find_route_config(path, workspace_dir, config)?
        && let Some(redirect) = target.redirect()
    {
        let status = redirect.status.unwrap_or(RedirectStatus::Found);
        return Ok(EncodeResult::Redirect {
            route: route_path,
            target: redirect.redirect.clone(),
            status,
        });
    }

    // Convert route to storage path (e.g., "/docs/report/" -> "docs/report/index.html")
    let trimmed = route.trim_start_matches('/').trim_end_matches('/');
    let html_storage_path = if trimmed.is_empty() {
        "index.html".to_string()
    } else {
        format!("{trimmed}/index.html")
    };

    // Create temp HTML file at a path that mirrors the final site structure.
    // This ensures relative paths from HTML to media are correct.
    // For route "/docs/report/", HTML goes to "{site_temp_root}/docs/report/index.html"
    // and media goes to "{site_temp_root}/media/", so relative path "../../media/hash.png" works.
    let temp_html = site_temp_root.join(&html_storage_path);
    if let Some(parent) = temp_html.parent() {
        create_dir_all(parent).await?;
    }

    // Media directory at site root level for shared deduplication
    let media_dir = site_temp_root.join("media");
    create_dir_all(&media_dir).await?;

    // Capture existing media files before encoding to detect new files
    let mut existing_media: HashSet<String> = HashSet::new();
    if media_dir.exists() {
        let mut entries = read_dir(&media_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            if let Some(filename) = entry.file_name().to_str() {
                existing_media.insert(filename.to_string());
            }
        }
    }

    // Encode HTML with media collection to shared directory
    // Media files are named with SeaHash of their content, so duplicates
    // across documents automatically deduplicate
    DomCodec
        .to_path(
            node,
            &temp_html,
            Some(EncodeOptions {
                standalone: Some(true),
                base_url: Some(base_url.to_string()),
                from_path: path.map(|p| p.to_path_buf()),
                to_path: Some(temp_html.clone()),
                // Collect and extract media to the shared media directory
                extract_media: Some(media_dir.clone()),
                collect_media: Some(media_dir.clone()),
                // Use static view for site publishing
                view: Some("static".into()),
                ..Default::default()
            }),
        )
        .await?;

    // Read the generated HTML
    let html_content = read(&temp_html).await?;

    // Collect only the NEW media files created during this document's encoding
    // by comparing against the snapshot taken before encoding
    let mut media_files = Vec::new();
    if media_dir.exists() {
        let mut entries = read_dir(&media_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            if entry_path.is_file()
                && let Some(filename) = entry_path.file_name().and_then(|n| n.to_str())
            {
                // Only include files that didn't exist before encoding
                if !existing_media.contains(filename) {
                    media_files.push((filename.to_string(), entry_path));
                }
            }
        }
    }

    Ok(EncodeResult::Document(EncodedDocument {
        source_path: path.map(|p| p.to_path_buf()).unwrap_or_default(),
        route,
        html_storage_path,
        html_content: html_content.to_vec(),
        media_files,
    }))
}

// ============================================================================
// Single document push
// ============================================================================

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
                        if let Some(parent) = dest_path.parent() {
                            create_dir_all(parent).await?;
                        }
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
            if let Some(parent) = dest_path.parent() {
                create_dir_all(parent).await?;
            }

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

        // Get repo URL for PR comments
        let repo_url = git_info(&start_path)
            .ok()
            .and_then(|info| info.origin)
            .unwrap_or_default();

        // Reconcile media files at this route to clean up orphaned files
        reconcile_prefix(
            &site_id,
            &repo_url,
            &branch_name,
            &branch_slug,
            &media_prefix,
            current_media_files,
        )
        .await?;
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

        if is_site_root_dir && let Some(routes) = &cfg.routes {
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

/// Push a document to a site with an explicit route
///
/// This is used for spread routes where the route is generated from a template
/// rather than derived from the file path.
///
/// # Arguments
/// * `node` - The document node to push
/// * `path` - Optional path to the source file (for media collection)
/// * `route` - The explicit route to use (e.g., "/north/ABC/")
/// * `dry_run` - Optional dry-run configuration
///
/// Returns the result of the push operation.
pub async fn push_with_route(
    node: &Node,
    path: Option<&Path>,
    route: &str,
    dry_run: Option<PushDryRunOptions>,
) -> Result<PushResult> {
    // Find the workspace root directory
    let start_path = if let Some(path) = path {
        path.to_path_buf()
    } else {
        std::env::current_dir()?
    };

    let stencila_dir = closest_stencila_dir(&start_path, true).await?;
    let _workspace_dir = workspace_dir(&stencila_dir)?;

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
    let base_url = format!("https://{site_id}.stencila.site");

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
                view: Some("static".into()),
                ..Default::default()
            }),
        )
        .await?;

    // Use the provided route, ensuring it ends with /
    let route = if route.ends_with('/') {
        route.to_string()
    } else {
        format!("{}/", route)
    };

    // Calculate storage path from route
    let trimmed = route.trim_start_matches('/').trim_end_matches('/');
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
        let local_path = if let Some(output_dir) = &dry_run_output_dir {
            let dest_path = output_dir.join(&full_html_path);
            if let Some(parent) = dest_path.parent() {
                create_dir_all(parent).await?;
            }

            // Compress HTML before writing
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
    } else {
        // Normal mode: upload to R2
        // Upload media files FIRST (before HTML) to avoid broken image references
        let media_prefix = if trimmed.is_empty() {
            "media/".to_string()
        } else {
            format!("{trimmed}/media/")
        };

        let mut current_media_files = Vec::new();
        if media_dir.exists() {
            let mut entries = read_dir(&media_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let media_path = entry.path();
                if let Some(filename) = media_path.file_name().and_then(|n| n.to_str()) {
                    // Include route prefix so media paths match HTML references
                    let media_storage_path = if trimmed.is_empty() {
                        format!("media/{filename}")
                    } else {
                        format!("{trimmed}/media/{filename}")
                    };
                    upload_file(&site_id, &branch_slug, &media_storage_path, &media_path).await?;
                    current_media_files.push(media_storage_path);
                }
            }
        }

        // Get repo URL for PR comments
        let repo_url = git_info(&start_path)
            .ok()
            .and_then(|info| info.origin)
            .unwrap_or_default();

        // Reconcile media files at this route to clean up orphaned files
        reconcile_prefix(
            &site_id,
            &repo_url,
            &branch_name,
            &branch_slug,
            &media_prefix,
            current_media_files,
        )
        .await?;

        // Upload HTML LAST (after media) so visitors always see complete content
        upload_file(&site_id, &branch_slug, &storage_path, &temp_html).await?;
    }

    // Build URL for result
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

// ============================================================================
// Directory push
// ============================================================================

/// Walk a directory and categorize files for site push
///
/// Walks the directory respecting `.gitignore` and config exclude patterns,
/// categorizing files as documents or static assets.
///
/// # Arguments
/// * `path` - The directory path to walk (must be the site root)
///
/// # Returns
/// A tuple of (document_paths, static_file_paths)
pub async fn walk_directory_for_push(path: &Path) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    // Find workspace root
    let stencila_dir = closest_stencila_dir(path, true).await?;
    let workspace_dir = workspace_dir(&stencila_dir)?;

    // Load config from workspace
    let config = stencila_config::config(&workspace_dir)?;

    // Resolve site root
    let site_root = if let Some(site) = &config.site
        && let Some(root) = &site.root
    {
        root.resolve(&workspace_dir)
    } else {
        workspace_dir.clone()
    };

    // Validate that the requested path is the site root
    let canonical_path = path.canonicalize()?;
    let canonical_root = site_root.canonicalize()?;
    if canonical_path != canonical_root {
        bail!(
            "Directory push requires the site root. Got: {}\nSite root is: {}\n\
             Hint: Use `stencila push {}` or adjust site.root in config",
            path.display(),
            site_root.display(),
            site_root.display()
        );
    }

    // Build walker using ignore crate
    let mut builder = WalkBuilder::new(&site_root);
    builder
        .hidden(false) // Don't skip hidden files by default (allows .htaccess, etc.)
        .git_ignore(true) // Respect .gitignore
        .git_global(true) // Respect global gitignore
        .git_exclude(true); // Respect .git/info/exclude

    // Build overrides to exclude sensitive directories and user-configured patterns
    let mut overrides = ignore::overrides::OverrideBuilder::new(&site_root);

    // Always exclude sensitive hidden directories that should never be uploaded:
    // - .git: Repository metadata, history, and potentially sensitive config
    // - .stencila: Workspace cache, auth tokens, remotes.json, secrets
    // - .env files: Environment variables often contain secrets
    // - node_modules: Large dependency directories
    // These patterns use '!' prefix which in overrides means "ignore/exclude"
    const SENSITIVE_PATTERNS: &[&str] = &[
        "!.git/",
        "!.stencila/",
        "!.env",
        "!.env.*",
        "!node_modules/",
    ];
    for pattern in SENSITIVE_PATTERNS {
        overrides.add(pattern)?;
    }

    // Add user-configured exclude patterns from site config
    if let Some(site) = &config.site
        && let Some(excludes) = &site.exclude
    {
        for pattern in excludes {
            // In OverrideBuilder, '!' prefix means "ignore/exclude" (inverted from gitignore)
            let exclude_pattern = format!("!{pattern}");
            overrides.add(&exclude_pattern)?;
        }
    }

    builder.overrides(overrides.build()?);

    // Walk and categorize files
    let mut documents: Vec<PathBuf> = Vec::new();
    let mut static_files: Vec<PathBuf> = Vec::new();

    for entry in builder.build() {
        let entry = entry?;
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue; // Skip directories
        }
        let file_path = entry.path().to_path_buf();

        match categorize_file(&file_path) {
            FileCategory::Document => documents.push(file_path),
            FileCategory::Static | FileCategory::Media => static_files.push(file_path),
        }
    }

    Ok((documents, static_files))
}

/// Push a directory to a Stencila Site
///
/// Walks the directory, encodes documents provided via the `decode_fn` callback
/// to HTML with shared media deduplication, and uploads all files to the site.
///
/// # Arguments
/// * `path` - The directory path to push (must be the site root)
/// * `site_id` - The site ID to push to
/// * `branch` - Optional branch name (defaults to current git branch or "main")
/// * `force` - Force upload all files even if unchanged (skip ETag comparison)
/// * `is_dry_run` - Whether this is a dry run (skip uploads even if no output path)
/// * `dry_run_output` - Optional path to write files for dry run inspection
/// * `progress` - Optional channel to send progress events
/// * `decode_fn` - Async function to decode a document from a path
///
/// # Error Handling
/// - **Encoding phase**: Continue on error - if one document fails, log it and continue
/// - **Upload phase**: Stop on first error - partial uploads leave site inconsistent
/// - **Reconciliation phase**: Stop on first error
#[allow(clippy::too_many_arguments)]
pub async fn push_directory<F, Fut>(
    path: &Path,
    site_id: &str,
    branch: Option<&str>,
    force: bool,
    is_dry_run: bool,
    dry_run_output: Option<&Path>,
    progress: Option<tokio::sync::mpsc::Sender<PushProgress>>,
    decode_fn: F,
) -> Result<DirectoryPushResult>
where
    F: Fn(PathBuf) -> Fut,
    Fut: std::future::Future<Output = Result<Node>>,
{
    // Helper macro to send progress events
    macro_rules! send_progress {
        ($event:expr) => {
            if let Some(tx) = &progress {
                let _ = tx.send($event).await;
            }
        };
    }

    send_progress!(PushProgress::WalkingDirectory);

    // Find workspace root
    let stencila_dir = closest_stencila_dir(path, true).await?;
    let workspace_dir = workspace_dir(&stencila_dir)?;

    // Load config from workspace
    let config = stencila_config::config(&workspace_dir)?;

    // Resolve site root
    let site_root = if let Some(site) = &config.site
        && let Some(root) = &site.root
    {
        root.resolve(&workspace_dir)
    } else {
        workspace_dir.clone()
    };

    // Walk and categorize files
    let (documents, static_files) = walk_directory_for_push(path).await?;

    send_progress!(PushProgress::FilesFound {
        documents: documents.len(),
        static_files: static_files.len(),
    });

    // Get branch info
    let branch_name = branch.map(String::from).unwrap_or_else(|| {
        get_current_branch(Some(&site_root)).unwrap_or_else(|| "main".to_string())
    });
    let branch_slug = slugify_branch_name(&branch_name);

    // Build base URL - prefer custom domain if configured, otherwise use default
    let base_url = if let Some(site) = &config.site
        && let Some(domain) = &site.domain
    {
        format!("https://{domain}")
    } else {
        format!("https://{site_id}.stencila.site")
    };

    // Create temp directory that mirrors the final site structure.
    // HTML files are placed at their route paths (e.g., docs/report/index.html)
    // and media files are placed in media/ subdirectory.
    // This ensures relative paths in HTML correctly reference media.
    let site_temp_root = TempDir::new()?;

    // Encode all documents
    let mut encoded_docs: Vec<EncodedDocument> = Vec::new();
    let mut redirects: Vec<(String, String, RedirectStatus)> = Vec::new();
    let mut documents_failed: Vec<(PathBuf, String)> = Vec::new();

    // Track total media files created by all documents (for duplicate counting)
    let mut total_media_created: usize = 0;

    for (index, doc_path) in documents.iter().enumerate() {
        send_progress!(PushProgress::EncodingDocument {
            path: doc_path.clone(),
            index,
            total: documents.len(),
        });

        let result = async {
            let node = decode_fn(doc_path.clone()).await?;
            encode_document(
                &node,
                Some(doc_path),
                &workspace_dir,
                &config,
                &base_url,
                site_temp_root.path(),
            )
            .await
        }
        .await;

        match result {
            Ok(EncodeResult::Document(encoded)) => {
                // Count media files created by this document
                total_media_created += encoded.media_files.len();
                send_progress!(PushProgress::DocumentEncoded {
                    path: doc_path.clone(),
                    route: encoded.route.clone(),
                });
                encoded_docs.push(encoded);
            }
            Ok(EncodeResult::Redirect {
                route,
                target,
                status,
            }) => {
                redirects.push((route, target, status));
            }
            Err(e) => {
                let error_msg = e.to_string();
                send_progress!(PushProgress::DocumentFailed {
                    path: doc_path.clone(),
                    error: error_msg.clone(),
                });
                documents_failed.push((doc_path.clone(), error_msg));
                // Continue with next document
            }
        }
    }

    // Add site-level redirects from config (redirect routes not tied to files)
    // This mirrors the logic in single-file push (handle_redirect_route loop)
    if let Some(routes) = &config.routes {
        for (route_path, target) in routes {
            if let Some(redirect_config) = target.redirect() {
                // Only add if not already covered by a document redirect
                let already_exists = redirects.iter().any(|(r, _, _)| r == route_path);
                if !already_exists {
                    redirects.push((
                        route_path.clone(),
                        redirect_config.redirect.clone(),
                        redirect_config
                            .status
                            .unwrap_or(RedirectStatus::TemporaryRedirect),
                    ));
                }
            }
        }
    }

    // Collect unique media files from shared media directory
    let mut media_to_upload: Vec<(String, PathBuf)> = Vec::new();
    let media_dir = site_temp_root.path().join("media");
    if media_dir.exists() {
        let mut entries = read_dir(&media_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let filename = entry.file_name().to_string_lossy().to_string();
            media_to_upload.push((filename, entry.path()));
        }
    }

    // Track ALL uploaded files for full-branch reconciliation.
    // We collect every file path that should exist on the site after this push,
    // then reconcile the entire branch to remove any files that weren't uploaded.
    // This ensures deleted documents/files are removed from the site.
    let mut all_uploaded_files: Vec<String> = Vec::new();

    // Calculate total uploads
    let total_uploads =
        encoded_docs.len() + redirects.len() + media_to_upload.len() + static_files.len();
    let mut uploaded_count = 0;

    // Track files skipped due to ETag match (only in upload mode, not dry-run)
    let mut files_skipped: usize = 0;

    // Handle dry-run vs actual upload
    if is_dry_run {
        // DRY RUN MODE: Write files locally if output path provided, otherwise just skip uploads

        if let Some(dry_run_path) = dry_run_output {
            // Write HTML files (gzipped to match production)
            for doc in &encoded_docs {
                let html_path = dry_run_path.join(format!(
                    "{}/{}/{}.gz",
                    site_id, branch_slug, doc.html_storage_path
                ));
                if let Some(parent) = html_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(&doc.html_content)?;
                let compressed = encoder.finish()?;
                std::fs::write(&html_path, compressed)?;

                uploaded_count += 1;
                send_progress!(PushProgress::Processing {
                    processed: uploaded_count,
                    uploaded: uploaded_count,
                    total: total_uploads,
                });
            }

            // Write redirect files
            for (route, target, status) in &redirects {
                let storage_path = route_to_redirect_storage_path(route);
                let redirect_path =
                    dry_run_path.join(format!("{}/{}/{}", site_id, branch_slug, storage_path));
                if let Some(parent) = redirect_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                let redirect_content = serde_json::to_string(&json!({
                    "location": target,
                    "status": status
                }))?;
                std::fs::write(&redirect_path, redirect_content)?;

                uploaded_count += 1;
                send_progress!(PushProgress::Processing {
                    processed: uploaded_count,
                    uploaded: uploaded_count,
                    total: total_uploads,
                });
            }

            // Copy media files
            let media_dest = dry_run_path.join(format!("{}/{}/media", site_id, branch_slug));
            std::fs::create_dir_all(&media_dest)?;
            for (filename, src_path) in &media_to_upload {
                std::fs::copy(src_path, media_dest.join(filename))?;

                uploaded_count += 1;
                send_progress!(PushProgress::Processing {
                    processed: uploaded_count,
                    uploaded: uploaded_count,
                    total: total_uploads,
                });
            }

            // Copy static files (preserving relative paths)
            for static_path in &static_files {
                let relative = static_path.strip_prefix(&site_root)?;
                let dest_path = dry_run_path.join(format!(
                    "{}/{}/{}",
                    site_id,
                    branch_slug,
                    relative.display()
                ));
                if let Some(parent) = dest_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::copy(static_path, &dest_path)?;

                uploaded_count += 1;
                send_progress!(PushProgress::Processing {
                    processed: uploaded_count,
                    uploaded: uploaded_count,
                    total: total_uploads,
                });
            }
        }
        // If no output path, just skip uploads (dry run without file output)
    } else {
        // UPLOAD MODE: Actually upload to R2

        // Prepare all files to upload with their content and storage paths
        struct FileToUpload {
            storage_path: String,
            content: Vec<u8>,
            /// For files that can be uploaded directly from disk
            source_path: Option<PathBuf>,
        }

        let mut files_to_upload: Vec<FileToUpload> = Vec::new();

        // Collect HTML files
        for doc in &encoded_docs {
            // Note: upload_file adds .gz suffix for HTML, but we calculate ETag on compressed content
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&doc.html_content)?;
            let compressed = encoder.finish()?;

            files_to_upload.push(FileToUpload {
                storage_path: format!("{}.gz", doc.html_storage_path),
                content: compressed,
                source_path: None,
            });
        }

        // Collect redirect files
        for (route, target, status) in &redirects {
            let storage_path = route_to_redirect_storage_path(route);
            let redirect_content = serde_json::to_string(&json!({
                "location": target,
                "status": status
            }))?;

            files_to_upload.push(FileToUpload {
                storage_path,
                content: redirect_content.into_bytes(),
                source_path: None,
            });
        }

        // Collect media files
        for (filename, file_path) in &media_to_upload {
            let storage_path = format!("media/{}", filename);
            let content = tokio::fs::read(file_path).await?;

            files_to_upload.push(FileToUpload {
                storage_path,
                content,
                source_path: Some(file_path.clone()),
            });
        }

        // Collect static files
        for static_path in &static_files {
            let relative = static_path.strip_prefix(&site_root)?;
            let storage_path = normalize_storage_path(&relative.to_string_lossy());
            let content = tokio::fs::read(static_path).await?;

            files_to_upload.push(FileToUpload {
                storage_path,
                content,
                source_path: Some(static_path.clone()),
            });
        }

        // Get server ETags for incremental upload (unless --force)
        let server_etags: HashMap<String, String> = if force {
            HashMap::new()
        } else {
            let paths: Vec<String> = files_to_upload
                .iter()
                .map(|f| f.storage_path.clone())
                .collect();
            get_etags(site_id, &branch_slug, paths)
                .await
                .unwrap_or_default()
        };

        // Upload files, skipping unchanged ones
        let mut actually_uploaded_count = 0;
        for file in files_to_upload {
            // Calculate local ETag
            let local_etag = calculate_etag(&file.content);

            // Check if file is unchanged (ETag matches)
            let should_skip = !force
                && server_etags
                    .get(&file.storage_path)
                    .map(|server_etag| server_etag == &local_etag)
                    .unwrap_or(false);

            // Always track for reconciliation (even if skipped)
            all_uploaded_files.push(file.storage_path.clone());

            if should_skip {
                files_skipped += 1;
            } else {
                // Upload the file
                // For HTML files (.gz suffix), we already have compressed content
                // For other files, upload directly
                if file.storage_path.ends_with(".html.gz") {
                    // Write compressed content to temp file and upload
                    // Note: upload_file expects uncompressed HTML and compresses it,
                    // but we already compressed it. We need to upload the raw bytes.
                    // For now, write to temp and use a direct upload approach.
                    let temp_dir = TempDir::new()?;
                    let temp_path = temp_dir.path().join("content.gz");
                    tokio::fs::write(&temp_path, &file.content).await?;

                    // Upload using the storage path without .gz (upload_file adds it)
                    let html_path = file.storage_path.trim_end_matches(".gz");
                    let temp_html = temp_dir.path().join("index.html");
                    // Decompress to get original HTML for upload_file
                    let mut decoder = flate2::read::GzDecoder::new(&file.content[..]);
                    let mut html_content = Vec::new();
                    std::io::Read::read_to_end(&mut decoder, &mut html_content)?;
                    tokio::fs::write(&temp_html, &html_content).await?;

                    upload_file(site_id, &branch_slug, html_path, &temp_html).await?;
                } else if let Some(source) = &file.source_path {
                    // Upload directly from source file
                    upload_file(site_id, &branch_slug, &file.storage_path, source).await?;
                } else {
                    // Write content to temp file and upload
                    let temp_dir = TempDir::new()?;
                    let temp_path = temp_dir.path().join("content");
                    tokio::fs::write(&temp_path, &file.content).await?;
                    upload_file(site_id, &branch_slug, &file.storage_path, &temp_path).await?;
                }
                actually_uploaded_count += 1;
            }

            uploaded_count += 1;
            send_progress!(PushProgress::Processing {
                processed: uploaded_count,
                uploaded: actually_uploaded_count,
                total: total_uploads,
            });
        }

        // Get repo URL for PR comments
        let repo_url = git_info(path)
            .ok()
            .and_then(|info| info.origin)
            .unwrap_or_default();

        // Reconcile entire branch with empty prefix to clean up ALL stale files.
        // This ensures that when documents/files are deleted locally, they are
        // also removed from the site. The API will delete any files not in
        // all_uploaded_files.
        send_progress!(PushProgress::Reconciling);
        reconcile_prefix(
            site_id,
            &repo_url,
            &branch_name,
            &branch_slug,
            "",
            all_uploaded_files,
        )
        .await?;
    }

    // Calculate how many duplicate media files were eliminated
    // total_media_created tracks media reported by each document
    // media_to_upload.len() is the unique files in the shared directory
    // The difference is the number of duplicates eliminated by SeaHash deduplication
    let duplicate_count = total_media_created.saturating_sub(media_to_upload.len());

    // Build result
    let result = DirectoryPushResult {
        documents_ok: encoded_docs
            .iter()
            .map(|d| (d.source_path.clone(), d.route.clone()))
            .collect(),
        documents_failed,
        redirects: redirects
            .iter()
            .map(|(r, t, _)| (r.clone(), t.clone()))
            .collect(),
        static_files_ok: static_files.clone(),
        static_files_failed: Vec::new(), // We stopped on error, so no partial failures
        media_files_count: media_to_upload.len(),
        media_duplicates_eliminated: duplicate_count,
        files_skipped,
    };

    send_progress!(PushProgress::Complete(result.clone()));

    Ok(result)
}

// ============================================================================
// Pull and other operations
// ============================================================================

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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_file_documents() {
        // Common document formats
        assert_eq!(
            categorize_file(Path::new("report.md")),
            FileCategory::Document
        );
        assert_eq!(
            categorize_file(Path::new("index.html")),
            FileCategory::Document
        );
        assert_eq!(
            categorize_file(Path::new("notebook.ipynb")),
            FileCategory::Document
        );
        assert_eq!(
            categorize_file(Path::new("paper.docx")),
            FileCategory::Document
        );
        assert_eq!(
            categorize_file(Path::new("data.json")),
            FileCategory::Document
        );
        assert_eq!(
            categorize_file(Path::new("config.yaml")),
            FileCategory::Document
        );
        assert_eq!(
            categorize_file(Path::new("article.smd")),
            FileCategory::Document
        );
    }

    #[test]
    fn test_categorize_file_static() {
        // Static assets
        assert_eq!(
            categorize_file(Path::new("style.css")),
            FileCategory::Static
        );
        assert_eq!(categorize_file(Path::new("app.js")), FileCategory::Static);
        assert_eq!(
            categorize_file(Path::new("font.woff2")),
            FileCategory::Static
        );
        assert_eq!(categorize_file(Path::new("data.txt")), FileCategory::Static);
    }

    #[test]
    fn test_categorize_file_media() {
        // Media files (images, audio, video)
        assert_eq!(categorize_file(Path::new("photo.png")), FileCategory::Media);
        assert_eq!(categorize_file(Path::new("image.jpg")), FileCategory::Media);
        assert_eq!(categorize_file(Path::new("clip.mp4")), FileCategory::Media);
        assert_eq!(categorize_file(Path::new("sound.mp3")), FileCategory::Media);
    }

    #[test]
    fn test_extract_prefix() {
        assert_eq!(extract_prefix("report/index.html"), "report/");
        assert_eq!(extract_prefix("index.html"), "");
        assert_eq!(extract_prefix("docs/api/index.html"), "docs/api/");
        assert_eq!(extract_prefix("assets/css/style.css"), "assets/css/");
        assert_eq!(extract_prefix("media/abc123.png"), "media/");
    }

    #[test]
    fn test_extract_filename() {
        assert_eq!(extract_filename("report/index.html"), "index.html");
        assert_eq!(extract_filename("index.html"), "index.html");
        assert_eq!(extract_filename("docs/api/index.html"), "index.html");
        assert_eq!(extract_filename("assets/css/style.css"), "style.css");
        assert_eq!(extract_filename("media/abc123.png"), "abc123.png");
    }

    #[test]
    fn test_normalize_storage_path() {
        // Unix paths (unchanged)
        assert_eq!(
            normalize_storage_path("assets/style.css"),
            "assets/style.css"
        );
        assert_eq!(normalize_storage_path("media/image.png"), "media/image.png");

        // Windows paths (backslashes to forward slashes)
        assert_eq!(
            normalize_storage_path("assets\\style.css"),
            "assets/style.css"
        );
        assert_eq!(
            normalize_storage_path("docs\\api\\index.html"),
            "docs/api/index.html"
        );
    }

    #[test]
    fn test_route_to_redirect_storage_path() {
        assert_eq!(route_to_redirect_storage_path("/"), "redirect.json");
        assert_eq!(
            route_to_redirect_storage_path("/old-page/"),
            "old-page/redirect.json"
        );
        assert_eq!(
            route_to_redirect_storage_path("/docs/old/"),
            "docs/old/redirect.json"
        );
        // Handle routes without trailing slash
        assert_eq!(
            route_to_redirect_storage_path("/old-page"),
            "old-page/redirect.json"
        );
    }
}
