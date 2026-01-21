//! Site push module
//!
//! Combines rendering and uploading into a single operation.
//! Renders a site to a temporary directory, then uploads to Stencila Cloud.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use eyre::Result;
use tempfile::TempDir;
use tokio::sync::mpsc;

use stencila_codec::stencila_schema::Node;
use stencila_codec_utils::get_current_branch;

use crate::{
    render::{self, RenderProgress, RenderResult},
    upload::{self, UploadProgress, UploadResult},
};

/// Combined result of push (render + upload)
#[derive(Debug, Clone)]
pub struct PushResult {
    /// Render results
    pub render: RenderResult,
    /// Upload results
    pub upload: UploadResult,
}

/// Progress events emitted during push
#[derive(Debug, Clone)]
pub enum PushProgress {
    // Render phase
    /// Starting to walk the directory
    WalkingDirectory,
    /// Found files to process
    FilesFound {
        documents: usize,
        static_files: usize,
    },
    /// Encoding a document
    EncodingDocument {
        path: std::path::PathBuf,
        relative_path: String,
        index: usize,
        total: usize,
    },
    /// Document encoded successfully
    DocumentEncoded {
        path: std::path::PathBuf,
        route: String,
    },
    /// Document encoding failed (continues with next)
    DocumentFailed {
        path: std::path::PathBuf,
        error: String,
    },

    // Upload phase
    /// Collecting files and fetching server ETags (in parallel)
    CollectingFiles,
    /// Starting upload
    UploadStarting { total: usize },
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

/// Push a site to Stencila Cloud
///
/// This is a convenience function that renders to a temporary directory,
/// then uploads all files to the cloud.
///
/// # Arguments
/// * `path` - The source path to push
/// * `workspace_id` - The workspace/site ID to push to
/// * `branch` - Optional branch name (defaults to current git branch or "main")
/// * `route_filter` - Optional filter to only push routes matching prefix
/// * `path_filter` - Optional filter by source file path prefix
/// * `source_files` - Optional list of exact source file paths to push
/// * `force` - Force upload all files even if unchanged
/// * `progress` - Optional channel for progress events
/// * `decode_document_fn` - Async function to decode a document from a path
///
/// # Error Handling
/// - **Render phase**: Continue on error - if one document fails, log it and continue
/// - **Upload phase**: Stop on first error - partial uploads leave site inconsistent
#[allow(clippy::too_many_arguments)]
pub async fn push<F, Fut>(
    path: &Path,
    workspace_id: &str,
    branch: Option<&str>,
    route_filter: Option<&str>,
    path_filter: Option<&str>,
    source_files: Option<&[PathBuf]>,
    force: bool,
    progress: Option<mpsc::Sender<PushProgress>>,
    decode_document_fn: F,
) -> Result<PushResult>
where
    F: Fn(std::path::PathBuf, HashMap<String, String>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<Node>> + Send + 'static,
{
    // Helper macro to send progress events
    macro_rules! send_progress {
        ($event:expr) => {
            if let Some(tx) = &progress {
                let _ = tx.send($event).await;
            }
        };
    }

    // Create temp directory for rendered output
    let temp_dir = TempDir::new()?;

    // Resolve base URL for HTML generation
    let base_url = resolve_base_url(path, workspace_id).await?;

    // Get branch info for upload
    let branch_name = branch
        .map(String::from)
        .unwrap_or_else(|| get_current_branch(Some(path)).unwrap_or_else(|| "main".to_string()));

    // Set up render progress channel with adapter
    let (render_tx, mut render_rx) = mpsc::channel::<RenderProgress>(100);

    // Forward render progress to combined progress
    let progress_clone = progress.clone();
    let render_forward = tokio::spawn(async move {
        while let Some(event) = render_rx.recv().await {
            if let Some(tx) = &progress_clone {
                let push_event = match event {
                    RenderProgress::WalkingDirectory => PushProgress::WalkingDirectory,
                    RenderProgress::FilesFound {
                        documents,
                        static_files,
                    } => PushProgress::FilesFound {
                        documents,
                        static_files,
                    },
                    RenderProgress::EncodingDocument {
                        path,
                        relative_path,
                        index,
                        total,
                    } => PushProgress::EncodingDocument {
                        path,
                        relative_path,
                        index,
                        total,
                    },
                    RenderProgress::DocumentEncoded { path, route } => {
                        PushProgress::DocumentEncoded { path, route }
                    }
                    RenderProgress::DocumentFailed { path, error } => {
                        PushProgress::DocumentFailed { path, error }
                    }
                    RenderProgress::CopyingStatic { .. } => continue, // Skip static copy progress
                    RenderProgress::Complete(_) => continue, // Handle completion separately
                };
                let _ = tx.send(push_event).await;
            }
        }
    });

    // Phase 1: Render to temp directory
    let render_result = render::render(
        path,
        temp_dir.path(),
        &base_url,
        route_filter,
        path_filter,
        source_files,
        Some(render_tx),
        decode_document_fn,
    )
    .await?;

    // Wait for render progress forwarding to complete
    let _ = render_forward.await;

    // Set up upload progress channel with adapter
    let (upload_tx, mut upload_rx) = mpsc::channel::<UploadProgress>(100);

    // Forward upload progress to combined progress
    let progress_clone = progress.clone();
    let upload_forward = tokio::spawn(async move {
        while let Some(event) = upload_rx.recv().await {
            if let Some(tx) = &progress_clone {
                let push_event = match event {
                    UploadProgress::CollectingFiles => PushProgress::CollectingFiles,
                    UploadProgress::Starting { total } => PushProgress::UploadStarting { total },
                    UploadProgress::Processing {
                        processed,
                        uploaded,
                        total,
                    } => PushProgress::Processing {
                        processed,
                        uploaded,
                        total,
                    },
                    UploadProgress::Reconciling => PushProgress::Reconciling,
                    UploadProgress::Complete(_) => continue, // Handle completion separately
                };
                let _ = tx.send(push_event).await;
            }
        }
    });

    // Only reconcile when pushing the full site (no filters applied)
    let is_filtered_push =
        route_filter.is_some() || path_filter.is_some() || source_files.is_some();

    // Phase 2: Upload to cloud
    let upload_result = upload::upload(
        temp_dir.path(),
        workspace_id,
        Some(&branch_name),
        force,
        !is_filtered_push, // reconcile only for full pushes
        Some(upload_tx),
    )
    .await?;

    // Wait for upload progress forwarding to complete
    let _ = upload_forward.await;

    let result = PushResult {
        render: render_result,
        upload: upload_result,
    };

    send_progress!(PushProgress::Complete(result.clone()));

    Ok(result)
}

/// Resolve the base URL for a site
///
/// Prefers custom domain if configured, otherwise uses default site URL.
async fn resolve_base_url(_path: &Path, workspace_id: &str) -> Result<String> {
    let config = stencila_config::get()?;

    let base_url = if let Some(site) = &config.site
        && let Some(domain) = &site.domain
    {
        format!("https://{domain}")
    } else {
        format!("https://{workspace_id}.stencila.site")
    };

    Ok(base_url)
}
