//! Site upload module
//!
//! Uploads a rendered site directory to Stencila Cloud with parallel uploads
//! and ETag-based incremental updates.

use std::{
    collections::HashMap,
    io::Write as _,
    path::{Path, PathBuf},
    sync::Arc,
};

use brotli::enc::writer::CompressorWriter;
use eyre::{Result, eyre};
use futures::stream::{self, StreamExt};
use md5::{Digest, Md5};
use reqwest::Client;
use tokio::{
    fs::read,
    sync::{Mutex, mpsc},
};
use walkdir::WalkDir;

use stencila_cloud::sites::{get_etags, reconcile_prefix};
use stencila_cloud::{api_token, base_url, check_response};
use stencila_codec_utils::{get_current_branch, git_repo_info, slugify_branch_name};

/// Result of an upload operation
#[derive(Debug, Clone)]
pub struct UploadResult {
    /// Number of files uploaded
    pub files_uploaded: usize,
    /// Number of files skipped (ETag match)
    pub files_skipped: usize,
    /// Total files processed
    pub total_files: usize,
}

/// Progress events during upload
#[derive(Debug, Clone)]
pub enum UploadProgress {
    /// Collecting files and fetching server ETags (in parallel)
    CollectingFiles,
    /// Starting upload
    Starting { total: usize },
    /// Processing files
    Processing {
        processed: usize,
        uploaded: usize,
        total: usize,
    },
    /// Reconciling stale files
    Reconciling,
    /// Upload complete
    Complete(UploadResult),
}

/// A file to be uploaded (metadata only, content read lazily)
#[derive(Debug, Clone)]
struct FileToUpload {
    /// Path to the local file
    local_path: PathBuf,
    /// Storage path on the cloud (relative, with .br suffix for compressible files)
    storage_path: String,
    /// Whether this file should be compressed (for Brotli compression and ETag calculation)
    compress: bool,
}

/// Determine if a file should be compressed based on its extension.
///
/// Returns true for text-based and other compressible formats,
/// false for already-compressed media files.
fn should_compress(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return false;
    };

    // Extensions that should NOT be compressed (already compressed or binary)
    let skip_extensions = [
        // Images
        "png", "jpg", "jpeg", "gif", "webp", "avif", "ico", "bmp", "tiff",
        // Audio/Video
        "mp3", "mp4", "webm", "wav", "ogg", "flac", "aac", "m4a", "avi", "mkv", "mov", "ogv", "wmv",
        // Archives
        "zip", "gz", "br", "zst", "xz", "bz2", "rar", "7z", "tar", // Other binary
        "wasm", "pdf",
    ];

    !skip_extensions.contains(&ext.to_lowercase().as_str())
}

/// Upload a rendered site directory to Stencila Cloud
///
/// Walks the rendered directory, compresses compressible files with Brotli,
/// and uploads all files with ETag-based incremental updates.
///
/// # Arguments
/// * `source_dir` - The rendered site directory (from render())
/// * `workspace_id` - The workspace/site ID to upload to
/// * `branch` - Optional branch name (defaults to current git branch or "main")
/// * `force` - Force upload all files even if unchanged (skip ETag comparison)
/// * `reconcile` - Whether to reconcile (delete stale files). Set to false for filtered uploads.
/// * `progress` - Optional channel for progress events
///
/// # Error Handling
/// - **Stop on first error**: Partial uploads leave site inconsistent
pub async fn upload(
    source_dir: &Path,
    workspace_id: &str,
    branch: Option<&str>,
    force: bool,
    reconcile: bool,
    progress: Option<mpsc::Sender<UploadProgress>>,
) -> Result<UploadResult> {
    // Helper macro to send progress events
    macro_rules! send_progress {
        ($event:expr) => {
            if let Some(tx) = &progress {
                let _ = tx.send($event).await;
            }
        };
    }

    // Get branch info
    let branch_name = branch.map(String::from).unwrap_or_else(|| {
        get_current_branch(Some(source_dir)).unwrap_or_else(|| "main".to_string())
    });
    let branch_slug = slugify_branch_name(&branch_name);

    // Fetch server ETags and collect local files in parallel (they're independent)
    send_progress!(UploadProgress::CollectingFiles);

    let etags_future = async {
        if force {
            HashMap::new()
        } else {
            get_etags(workspace_id, &branch_slug)
                .await
                .unwrap_or_default()
        }
    };

    let files_future = async {
        let mut files: Vec<FileToUpload> = Vec::new();
        for entry in WalkDir::new(source_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let local_path = entry.path().to_path_buf();
            let relative = match local_path.strip_prefix(source_dir) {
                Ok(r) => r,
                Err(_) => continue,
            };
            let storage_path = normalize_storage_path(&relative.to_string_lossy());

            let compress = should_compress(&local_path);

            // For compressible files, the storage path will have .br appended
            let storage_path = if compress {
                format!("{storage_path}.br")
            } else {
                storage_path
            };

            files.push(FileToUpload {
                local_path,
                storage_path,
                compress,
            });
        }
        files
    };

    let (server_etags, files) = tokio::join!(etags_future, files_future);
    let total = files.len();

    // Now that prep work is done, signal we're starting actual uploads
    send_progress!(UploadProgress::Starting { total });

    // Get API token once for all uploads
    let token = api_token().ok_or_else(|| eyre!("No API token. Run `stencila signin` first."))?;

    // Create a single HTTP client for connection reuse
    let client = Client::new();
    let upload_base_url = base_url();

    // Track progress counters
    let processed = Arc::new(Mutex::new(0usize));
    let uploaded = Arc::new(Mutex::new(0usize));
    let skipped = Arc::new(Mutex::new(0usize));
    let all_storage_paths = Arc::new(Mutex::new(Vec::<String>::new()));

    // Upload files in parallel (10 concurrent)
    let workspace_id = workspace_id.to_string();
    let branch_slug_clone = branch_slug.clone();
    let progress_clone = progress.clone();

    let results: Vec<Result<(), eyre::Error>> = stream::iter(files)
        .map(|file| {
            let workspace_id = workspace_id.clone();
            let branch_slug = branch_slug_clone.clone();
            let server_etags = server_etags.clone();
            let processed = Arc::clone(&processed);
            let uploaded = Arc::clone(&uploaded);
            let skipped = Arc::clone(&skipped);
            let all_storage_paths = Arc::clone(&all_storage_paths);
            let progress = progress_clone.clone();
            let client = client.clone();
            let token = token.clone();
            let upload_base_url = upload_base_url.clone();

            async move {
                // Track for reconciliation
                {
                    let mut paths = all_storage_paths.lock().await;
                    paths.push(file.storage_path.clone());
                }

                // Read file content
                let content = read(&file.local_path).await?;

                // Compress if needed (also used for ETag calculation)
                let (body, local_etag) = if file.compress {
                    let mut compressed = Vec::new();
                    {
                        // Quality 6, buffer 4096, lgwin 22
                        let mut encoder = CompressorWriter::new(&mut compressed, 4096, 6, 22);
                        encoder.write_all(&content)?;
                    }
                    let etag = calculate_etag(&compressed);
                    (compressed, etag)
                } else {
                    let etag = calculate_etag(&content);
                    (content, etag)
                };

                // Check if file is unchanged (ETag matches)
                let should_skip = !force
                    && server_etags
                        .get(&file.storage_path)
                        .map(|server_etag| server_etag == &local_etag)
                        .unwrap_or(false);

                if should_skip {
                    let mut s = skipped.lock().await;
                    *s += 1;
                } else {
                    // Upload the file directly
                    let response = client
                        .put(format!(
                            "{}/workspaces/{}/site/branches/{}/{}",
                            upload_base_url, workspace_id, branch_slug, file.storage_path
                        ))
                        .bearer_auth(&token)
                        .body(body)
                        .send()
                        .await?;

                    check_response(response).await?;

                    let mut u = uploaded.lock().await;
                    *u += 1;
                }

                // Update processed count and send progress
                let current_processed = {
                    let mut p = processed.lock().await;
                    *p += 1;
                    *p
                };

                if let Some(tx) = &progress {
                    let u = *uploaded.lock().await;
                    let _ = tx
                        .send(UploadProgress::Processing {
                            processed: current_processed,
                            uploaded: u,
                            total,
                        })
                        .await;
                }

                Ok(())
            }
        })
        .buffer_unordered(10) // 10 concurrent uploads
        .collect()
        .await;

    // Check for any upload errors
    for result in results {
        result?;
    }

    // Reconcile if requested
    if reconcile {
        send_progress!(UploadProgress::Reconciling);

        // Get repo URL for PR comments
        let repo_url = git_repo_info(source_dir)
            .ok()
            .and_then(|info| info.origin)
            .unwrap_or_default();

        let all_files = all_storage_paths.lock().await.clone();

        reconcile_prefix(
            &workspace_id,
            &repo_url,
            &branch_name,
            &branch_slug,
            "",
            all_files,
        )
        .await?;
    }

    let files_uploaded = *uploaded.lock().await;
    let files_skipped = *skipped.lock().await;

    let result = UploadResult {
        files_uploaded,
        files_skipped,
        total_files: total,
    };

    send_progress!(UploadProgress::Complete(result.clone()));

    Ok(result)
}

/// Normalize a path to use forward slashes for storage paths.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_etag() {
        let content = b"Hello, World!";
        let etag = calculate_etag(content);
        // MD5 of "Hello, World!" is 65a8e27d8879283831b664bd8b7f0ad4
        assert_eq!(etag, "\"65a8e27d8879283831b664bd8b7f0ad4\"");
    }

    #[test]
    fn test_normalize_storage_path() {
        assert_eq!(normalize_storage_path("a/b/c.html"), "a/b/c.html");
        assert_eq!(normalize_storage_path("a\\b\\c.html"), "a/b/c.html");
    }

    #[test]
    fn test_should_compress() {
        // Compressible file types
        assert!(should_compress(Path::new("index.html")));
        assert!(should_compress(Path::new("styles.css")));
        assert!(should_compress(Path::new("app.js")));
        assert!(should_compress(Path::new("data.json")));
        assert!(should_compress(Path::new("icon.svg")));
        assert!(should_compress(Path::new("feed.xml")));
        assert!(should_compress(Path::new("readme.txt")));
        assert!(should_compress(Path::new("doc.md")));

        // Non-compressible file types (images)
        assert!(!should_compress(Path::new("photo.png")));
        assert!(!should_compress(Path::new("image.jpg")));
        assert!(!should_compress(Path::new("image.jpeg")));
        assert!(!should_compress(Path::new("animation.gif")));
        assert!(!should_compress(Path::new("modern.webp")));
        assert!(!should_compress(Path::new("next-gen.avif")));
        assert!(!should_compress(Path::new("favicon.ico")));

        // Non-compressible file types (media)
        assert!(!should_compress(Path::new("song.mp3")));
        assert!(!should_compress(Path::new("video.mp4")));

        // Non-compressible file types (archives)
        assert!(!should_compress(Path::new("archive.zip")));
        assert!(!should_compress(Path::new("already.gz")));
        assert!(!should_compress(Path::new("already.br")));

        // Non-compressible file types (other binary)
        assert!(!should_compress(Path::new("doc.pdf")));
        assert!(!should_compress(Path::new("module.wasm")));

        // No extension - should not compress
        assert!(!should_compress(Path::new("Makefile")));
    }
}
