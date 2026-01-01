//! Site upload module
//!
//! Uploads a rendered site directory to Stencila Cloud with parallel uploads
//! and ETag-based incremental updates.

use std::{
    collections::HashMap,
    io::Write as IoWrite,
    path::{Path, PathBuf},
    sync::Arc,
};

use eyre::Result;
use flate2::{Compression, write::GzEncoder};
use futures::stream::{self, StreamExt};
use md5::{Digest, Md5};
use tokio::{
    fs::read,
    sync::{Mutex, mpsc},
};
use walkdir::WalkDir;

use stencila_cloud::sites::{get_etags, reconcile_prefix, upload_file};
use stencila_codec_utils::{get_current_branch, git_info, slugify_branch_name};

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

/// A file to be uploaded
#[derive(Debug)]
struct FileToUpload {
    /// Path to the local file
    local_path: PathBuf,
    /// Storage path on the cloud (relative)
    storage_path: String,
    /// Pre-calculated ETag for incremental upload
    etag: String,
}

/// Upload a rendered site directory to Stencila Cloud
///
/// Walks the rendered directory, compresses HTML files, and uploads
/// all files with ETag-based incremental updates.
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

    // Collect all files to upload
    let mut files: Vec<FileToUpload> = Vec::new();

    for entry in WalkDir::new(source_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let local_path = entry.path().to_path_buf();
        let relative = local_path
            .strip_prefix(source_dir)
            .map_err(|_| eyre::eyre!("Failed to strip prefix"))?;
        let storage_path = normalize_storage_path(&relative.to_string_lossy());

        // Calculate ETag based on what will actually be uploaded
        // For HTML files, upload_file compresses them, so we need to calculate
        // ETag on the compressed content
        let content = read(&local_path).await?;
        let etag = if local_path
            .extension()
            .map(|ext| ext == "html")
            .unwrap_or(false)
        {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&content)?;
            let compressed = encoder.finish()?;
            calculate_etag(&compressed)
        } else {
            calculate_etag(&content)
        };

        // For HTML files, the storage path will have .gz appended by upload_file
        let storage_path = if local_path
            .extension()
            .map(|ext| ext == "html")
            .unwrap_or(false)
        {
            format!("{storage_path}.gz")
        } else {
            storage_path
        };

        files.push(FileToUpload {
            local_path,
            storage_path,
            etag,
        });
    }

    let total = files.len();
    send_progress!(UploadProgress::Starting { total });

    // Get server ETags for incremental upload (unless --force)
    let server_etags: HashMap<String, String> = if force {
        HashMap::new()
    } else {
        let paths: Vec<String> = files.iter().map(|f| f.storage_path.clone()).collect();
        get_etags(workspace_id, &branch_slug, paths)
            .await
            .unwrap_or_default()
    };

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

            async move {
                // Check if file is unchanged (ETag matches)
                let should_skip = !force
                    && server_etags
                        .get(&file.storage_path)
                        .map(|server_etag| server_etag == &file.etag)
                        .unwrap_or(false);

                // Track for reconciliation
                {
                    let mut paths = all_storage_paths.lock().await;
                    paths.push(file.storage_path.clone());
                }

                if should_skip {
                    let mut s = skipped.lock().await;
                    *s += 1;
                } else {
                    // Upload the file
                    // For HTML files, strip the .gz suffix as upload_file will add it
                    let upload_path = if file.storage_path.ends_with(".html.gz") {
                        file.storage_path.trim_end_matches(".gz").to_string()
                    } else {
                        file.storage_path.clone()
                    };

                    upload_file(&workspace_id, &branch_slug, &upload_path, &file.local_path)
                        .await?;

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
        let repo_url = git_info(source_dir)
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
}
