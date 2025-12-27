use std::{
    collections::{HashMap, HashSet},
    io::Write as IoWrite,
    path::{Path, PathBuf},
};

use eyre::Result;
use flate2::{Compression, write::GzEncoder};
use md5::{Digest, Md5};
use serde_json::json;
use tempfile::TempDir;
use tokio::{
    fs::{create_dir_all, read, read_dir},
    sync::mpsc,
};

use stencila_cloud::sites::{get_etags, reconcile_prefix, upload_file};
use stencila_codec::{Codec, EncodeOptions, stencila_schema::Node};
use stencila_codec_dom::DomCodec;
use stencila_codec_utils::{get_current_branch, git_info, slugify_branch_name};
use stencila_config::RedirectStatus;
use stencila_dirs::{closest_stencila_dir, workspace_dir};

use crate::{RouteEntry, RouteType, list};

/// A document encoded and ready for upload (but not yet uploaded)
#[derive(Debug)]
struct EncodedDocument {
    /// The source file path
    source_path: PathBuf,
    /// The computed route (e.g., "/report/")
    route: String,
    /// The storage path for HTML (e.g., "report/index.html")
    html_storage_path: String,
    /// The HTML content (not yet compressed)
    html_content: Vec<u8>,
    /// Media files collected for this document: (filename, file_path)
    /// Note: The filename already contains the SeaHash (e.g., "a1b2c3d4.png")
    /// as computed by node-media's Collector (see collect.rs:164)
    /// These paths point into the shared media directory.
    media_files: Vec<(String, PathBuf)>,
}

/// Result of a push
#[derive(Debug, Clone)]
pub struct PushResult {
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
    Complete(PushResult),
}

/// Push a path to a Stencila Site
///
/// Walks the path (file or directory), encodes documents provided via the `decode_fn` callback
/// to HTML with shared media deduplication, and uploads all files to the site.
///
/// # Arguments
/// * `path` - The path to push (file or directory within site.root)
/// * `workspace_id` - The site ID to push to
/// * `branch` - Optional branch name (defaults to current git branch or "main")
/// * `force` - Force upload all files even if unchanged (skip ETag comparison)
/// * `is_dry_run` - Whether this is a dry run (skip uploads even if no output path)
/// * `dry_run_output` - Optional path to write files for dry run inspection
/// * `progress` - Optional channel to send progress events
/// * `decode_document_fn` - Async function to decode a document from a path with optional spread arguments.
///   For regular documents, arguments will be empty. For spread routes, arguments contain the
///   parameter values for that variant.
///
/// # Error Handling
/// - **Encoding phase**: Continue on error - if one document fails, log it and continue
/// - **Upload phase**: Stop on first error - partial uploads leave site inconsistent
/// - **Reconciliation phase**: Stop on first error
#[allow(clippy::too_many_arguments)]
pub async fn push<F, Fut>(
    path: &Path,
    workspace_id: &str,
    branch: Option<&str>,
    route_filter: Option<&str>,
    path_filter: Option<&str>,
    force: bool,
    is_dry_run: bool,
    dry_run_output: Option<&Path>,
    progress: Option<mpsc::Sender<PushProgress>>,
    decode_document_fn: F,
) -> Result<PushResult>
where
    F: Fn(PathBuf, HashMap<String, String>) -> Fut,
    Fut: std::future::Future<Output = Result<Node>>,
{
    let all_routes = list(path, true, true, route_filter, path_filter).await?;

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

    // Partition routes by type.
    // Documents include File, Implied, and Spread routes (all have source_path and route).
    // Static files are uploaded as-is without HTML encoding.
    let mut document_routes: Vec<RouteEntry> = Vec::new();
    let mut static_files: Vec<PathBuf> = Vec::new();

    for entry in all_routes {
        match entry.route_type {
            RouteType::File | RouteType::Implied | RouteType::Spread => {
                if entry.source_path.is_some() {
                    document_routes.push(entry);
                }
            }
            RouteType::Static => {
                if let Some(path) = entry.source_path {
                    static_files.push(path);
                }
            }
            RouteType::Redirect => {
                // Redirects are handled separately below
            }
        }
    }

    send_progress!(PushProgress::FilesFound {
        documents: document_routes.len(),
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
        format!("https://{workspace_id}.stencila.site")
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

    // Process all document routes (File, Implied, and Spread) in a single loop.
    // list() already computed the route for each entry, so we use entry.route as the override.
    for (index, entry) in document_routes.iter().enumerate() {
        let source_path = entry.source_path.as_ref().expect("filtered above");

        // Skip if source file doesn't exist
        if !source_path.exists() {
            let error_msg = format!("Source file not found: {}", entry.target);
            tracing::warn!("{}", error_msg);
            documents_failed.push((source_path.clone(), error_msg));
            continue;
        }

        send_progress!(PushProgress::EncodingDocument {
            path: source_path.clone(),
            index,
            total: document_routes.len(),
        });

        // Convert spread arguments from IndexMap to HashMap (empty for non-spread routes)
        let arguments: HashMap<String, String> = entry
            .spread_arguments
            .as_ref()
            .map(|args| args.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();

        let result = async {
            let node = decode_document_fn(source_path.clone(), arguments.clone()).await?;
            encode_document(
                &node,
                Some(source_path),
                &base_url,
                site_temp_root.path(),
                &entry.route,
            )
            .await
        }
        .await;

        match result {
            Ok(encoded) => {
                total_media_created += encoded.media_files.len();
                send_progress!(PushProgress::DocumentEncoded {
                    path: source_path.clone(),
                    route: encoded.route.clone(),
                });
                encoded_docs.push(encoded);
            }
            Err(e) => {
                let error_msg = if arguments.is_empty() {
                    e.to_string()
                } else {
                    format!("Spread variant {arguments:?}: {e}")
                };
                send_progress!(PushProgress::DocumentFailed {
                    path: source_path.clone(),
                    error: error_msg.clone(),
                });
                documents_failed.push((source_path.clone(), error_msg));
            }
        }
    }

    // Add site-level redirects from config (redirect routes not tied to files)
    // This mirrors the logic in single-file push (handle_redirect_route loop)
    if let Some(site) = &config.site
        && let Some(routes) = &site.routes
    {
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
                    workspace_id, branch_slug, doc.html_storage_path
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
                    dry_run_path.join(format!("{workspace_id}/{branch_slug}/{storage_path}"));
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
            let media_dest = dry_run_path.join(format!("{workspace_id}/{branch_slug}/media"));
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
                    "{workspace_id}/{branch_slug}/{}",
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
            get_etags(workspace_id, &branch_slug, paths)
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

                    upload_file(workspace_id, &branch_slug, html_path, &temp_html).await?;
                } else if let Some(source) = &file.source_path {
                    // Upload directly from source file
                    upload_file(workspace_id, &branch_slug, &file.storage_path, source).await?;
                } else {
                    // Write content to temp file and upload
                    let temp_dir = TempDir::new()?;
                    let temp_path = temp_dir.path().join("content");
                    tokio::fs::write(&temp_path, &file.content).await?;
                    upload_file(workspace_id, &branch_slug, &file.storage_path, &temp_path).await?;
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

        // Only reconcile when pushing the full site (no filters applied).
        // When filters are used, we're intentionally pushing a subset and don't
        // want to delete the rest of the site.
        let is_filtered_push = route_filter.is_some() || path_filter.is_some();

        if !is_filtered_push {
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
                workspace_id,
                &repo_url,
                &branch_name,
                &branch_slug,
                "",
                all_uploaded_files,
            )
            .await?;
        }
    }

    // Calculate how many duplicate media files were eliminated
    // total_media_created tracks media reported by each document
    // media_to_upload.len() is the unique files in the shared directory
    // The difference is the number of duplicates eliminated by SeaHash deduplication
    let duplicate_count = total_media_created.saturating_sub(media_to_upload.len());

    // Build result
    let result = PushResult {
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

/// Encode a document to HTML without uploading.
///
/// This function extracts the encoding logic from `push()` to allow:
/// 1. Encoding multiple documents with a shared media directory (for deduplication)
/// 2. Generating HTML with correct media paths upfront (no post-hoc rewriting needed)
///
/// # Arguments
/// * `node` - The decoded document node
/// * `path` - Original source file path (used for media resolution)
/// * `base_url` - Base URL for the site (e.g., "https://mysite.stencila.site")
/// * `site_temp_root` - Temporary directory that mirrors the site structure. Media is placed
///   at `{site_temp_root}/media/` and HTML at `{site_temp_root}/{route}/index.html` so that
///   relative paths in the generated HTML correctly point to `/media/{hash}.ext`.
/// * `route` - The route for this document (e.g., "/docs/report/")
///
/// # Returns
/// The encoded document with HTML content and list of media files collected.
async fn encode_document(
    node: &Node,
    path: Option<&Path>,
    base_url: &str,
    site_temp_root: &Path,
    route: &str,
) -> Result<EncodedDocument> {
    // Ensure route ends with /
    let route = if route.ends_with('/') {
        route.to_string()
    } else {
        format!("{route}/")
    };

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

    Ok(EncodedDocument {
        source_path: path.map(|p| p.to_path_buf()).unwrap_or_default(),
        route,
        html_storage_path,
        html_content: html_content.to_vec(),
        media_files,
    })
}

/// Normalize a path to use forward slashes for cloud storage keys.
///
/// On Windows, `Path::to_string_lossy()` produces backslashes which are
/// invalid for cloud storage keys and break URL routing.
///
/// # Examples
/// - `"assets/style.css"` -> `"assets/style.css"` (unchanged on Unix)
/// - `"assets\\style.css"` -> `"assets/style.css"` (normalized on Windows)
fn normalize_storage_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Calculate an ETag for content (MD5 hash in quoted hex format).
///
/// This must match how Stencila Cloud calculates ETags for uploaded files.
/// Format: `"<hex-encoded-md5>"` (quotes included as per HTTP ETag spec)
fn calculate_etag(content: &[u8]) -> String {
    let hash = Md5::digest(content);
    format!("\"{:x}\"", hash)
}

/// Convert a route to redirect storage path
///
/// # Examples
/// - `"/"` -> `"redirect.json"`
/// - `"/old-page/"` -> `"old-page/redirect.json"`
/// - `"/docs/old/"` -> `"docs/old/redirect.json"`
fn route_to_redirect_storage_path(route: &str) -> String {
    let trimmed = route.trim_matches('/');
    if trimmed.is_empty() {
        "redirect.json".to_string()
    } else {
        format!("{trimmed}/redirect.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
