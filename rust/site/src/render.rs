//! Site rendering module
//!
//! Renders a site to a directory with HTML documents, media files, static assets,
//! and redirect metadata. The output can be inspected locally or uploaded to
//! Stencila Cloud using the upload module.

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use eyre::Result;
use serde_json::json;
use tokio::{
    fs::{create_dir_all, read_dir, write},
    sync::mpsc,
};

use stencila_codec::{Codec, EncodeOptions, stencila_schema::Node};
use stencila_codec_dom::DomCodec;
use stencila_config::{RedirectStatus, SiteLayout};
use stencila_dirs::{closest_stencila_dir, workspace_dir};

use crate::{RouteEntry, RouteType, list};

/// A document rendered to HTML
#[derive(Debug)]
struct RenderedDocument {
    /// The source file path
    source_path: PathBuf,

    /// The computed route (e.g., "/report/")
    route: String,

    /// Media files collected for this document: (filename, file_path)
    /// Note: The filename already contains the SeaHash (e.g., "a1b2c3d4.png")
    /// as computed by node-media's Collector
    media_files: Vec<(String, PathBuf)>,
}

/// Result of a render operation
#[derive(Debug, Clone)]
pub struct RenderResult {
    /// Documents successfully rendered: (source_path, route)
    pub documents_ok: Vec<(PathBuf, String)>,

    /// Documents that failed: (source_path, error_message)
    pub documents_failed: Vec<(PathBuf, String)>,

    /// Redirects processed: (route, target)
    pub redirects: Vec<(String, String)>,

    /// Static files copied
    pub static_files: Vec<PathBuf>,

    /// Total unique media files (after deduplication)
    pub media_files_count: usize,

    /// Number of duplicates eliminated
    pub media_duplicates_eliminated: usize,
}

/// Progress events during rendering
#[derive(Debug, Clone)]
pub enum RenderProgress {
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
        relative_path: String,
        index: usize,
        total: usize,
    },
    /// Document encoded successfully
    DocumentEncoded { path: PathBuf, route: String },
    /// Document encoding failed (continues with next)
    DocumentFailed { path: PathBuf, error: String },
    /// Copying static files
    CopyingStatic { copied: usize, total: usize },
    /// Render complete
    Complete(RenderResult),
}

/// Render a site to a directory
///
/// Walks the source path, encodes documents to HTML, and writes all files
/// (HTML, redirects, media, static) to the output directory.
///
/// # Arguments
/// * `source` - The source directory to render
/// * `output` - The output directory for rendered files
/// * `base_url` - Base URL for the site (used in HTML for absolute links)
/// * `route_filter` - Optional filter to only render routes matching prefix
/// * `path_filter` - Optional filter to only render files matching path prefix
/// * `progress` - Optional channel for progress events
/// * `decode_document_fn` - Async function to decode a document from a path
///
/// # Error Handling
/// - **Continue on error**: If one document fails, log it and continue with next
#[allow(clippy::too_many_arguments)]
pub async fn render<F, Fut>(
    source: &Path,
    output: &Path,
    base_url: &str,
    route_filter: Option<&str>,
    path_filter: Option<&str>,
    progress: Option<mpsc::Sender<RenderProgress>>,
    decode_document_fn: F,
) -> Result<RenderResult>
where
    F: Fn(PathBuf, HashMap<String, String>) -> Fut,
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

    send_progress!(RenderProgress::WalkingDirectory);

    // List all routes
    let all_routes = list(source, true, true, route_filter, path_filter).await?;

    // Find workspace root for config
    let stencila_dir = closest_stencila_dir(source, true).await?;
    let workspace_dir = workspace_dir(&stencila_dir)?;

    // Load config from workspace
    let config = stencila_config::config(&workspace_dir)?;

    // Resolve site root for static file paths
    let site_root = if let Some(site) = &config.site
        && let Some(root) = &site.root
    {
        root.resolve(&workspace_dir)
    } else {
        workspace_dir.clone()
    };

    // Get layout configuration
    let layout = config.site.as_ref().and_then(|s| s.layout.clone());

    // Partition routes by type
    let mut document_routes: Vec<RouteEntry> = Vec::new();
    let mut static_files: Vec<PathBuf> = Vec::new();
    let mut redirects: Vec<(String, String, RedirectStatus)> = Vec::new();

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
                // Collect redirects for later
            }
        }
    }

    // Add site-level redirects from config
    if let Some(site) = &config.site
        && let Some(routes) = &site.routes
    {
        for (route_path, target) in routes {
            if let Some(redirect_config) = target.redirect() {
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

    send_progress!(RenderProgress::FilesFound {
        documents: document_routes.len(),
        static_files: static_files.len(),
    });

    // Create output directory
    create_dir_all(output).await?;

    // Render all documents
    let mut rendered_docs: Vec<RenderedDocument> = Vec::new();
    let mut documents_failed: Vec<(PathBuf, String)> = Vec::new();
    let mut total_media_created: usize = 0;

    for (index, entry) in document_routes.iter().enumerate() {
        let source_path = entry.source_path.as_ref().expect("filtered above");

        // Skip if source file doesn't exist
        if !source_path.exists() {
            let error_msg = format!("Source file not found: {}", entry.target);
            tracing::warn!("{}", error_msg);
            documents_failed.push((source_path.clone(), error_msg));
            continue;
        }

        let relative_path = source_path
            .strip_prefix(source)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| source_path.to_string_lossy().to_string());

        send_progress!(RenderProgress::EncodingDocument {
            path: source_path.clone(),
            relative_path,
            index,
            total: document_routes.len(),
        });

        // Convert spread arguments from IndexMap to HashMap
        let arguments: HashMap<String, String> = entry
            .spread_arguments
            .as_ref()
            .map(|args| args.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();

        let result = async {
            let node = decode_document_fn(source_path.clone(), arguments.clone()).await?;
            render_document(
                &node,
                Some(source_path),
                base_url,
                output,
                &entry.route,
                layout.as_ref(),
            )
            .await
        }
        .await;

        match result {
            Ok(rendered) => {
                total_media_created += rendered.media_files.len();
                send_progress!(RenderProgress::DocumentEncoded {
                    path: source_path.clone(),
                    route: rendered.route.clone(),
                });
                rendered_docs.push(rendered);
            }
            Err(e) => {
                let error_msg = if arguments.is_empty() {
                    e.to_string()
                } else {
                    format!("Spread variant {arguments:?}: {e}")
                };
                send_progress!(RenderProgress::DocumentFailed {
                    path: source_path.clone(),
                    error: error_msg.clone(),
                });
                documents_failed.push((source_path.clone(), error_msg));
            }
        }
    }

    // Write redirect files
    for (route, target, status) in &redirects {
        let storage_path = route_to_redirect_storage_path(route);
        let redirect_path = output.join(&storage_path);
        if let Some(parent) = redirect_path.parent() {
            create_dir_all(parent).await?;
        }

        let redirect_content = serde_json::to_string(&json!({
            "location": target,
            "status": status
        }))?;
        write(&redirect_path, redirect_content).await?;
    }

    // Copy static files (preserving relative paths)
    for (index, static_path) in static_files.iter().enumerate() {
        let relative = static_path.strip_prefix(&site_root)?;
        let dest_path = output.join(normalize_storage_path(&relative.to_string_lossy()));
        if let Some(parent) = dest_path.parent() {
            create_dir_all(parent).await?;
        }
        tokio::fs::copy(static_path, &dest_path).await?;

        send_progress!(RenderProgress::CopyingStatic {
            copied: index + 1,
            total: static_files.len(),
        });
    }

    // Count unique media files
    let media_dir = output.join("media");
    let media_files_count = if media_dir.exists() {
        let mut count = 0;
        let mut entries = read_dir(&media_dir).await?;
        while entries.next_entry().await?.is_some() {
            count += 1;
        }
        count
    } else {
        0
    };

    let duplicate_count = total_media_created.saturating_sub(media_files_count);

    let result = RenderResult {
        documents_ok: rendered_docs
            .iter()
            .map(|d| (d.source_path.clone(), d.route.clone()))
            .collect(),
        documents_failed,
        redirects: redirects
            .iter()
            .map(|(r, t, _)| (r.clone(), t.clone()))
            .collect(),
        static_files: static_files.clone(),
        media_files_count,
        media_duplicates_eliminated: duplicate_count,
    };

    send_progress!(RenderProgress::Complete(result.clone()));

    Ok(result)
}

/// Render a document to HTML
///
/// # Arguments
/// * `node` - The decoded document node
/// * `path` - Original source file path (used for media resolution)
/// * `base_url` - Base URL for the site
/// * `output_root` - Output directory root
/// * `route` - The route for this document (e.g., "/docs/report/")
/// * `layout` - Optional site layout configuration for wrapping content
///
/// # Returns
/// The rendered document with path information and media files collected.
async fn render_document(
    node: &Node,
    path: Option<&Path>,
    base_url: &str,
    output_root: &Path,
    route: &str,
    layout: Option<&SiteLayout>,
) -> Result<RenderedDocument> {
    // Ensure route ends with /
    let route = if route.ends_with('/') {
        route.to_string()
    } else {
        format!("{route}/")
    };

    // Convert route to HTML path (e.g., "/docs/report/" -> "docs/report/index.html")
    let trimmed = route.trim_start_matches('/').trim_end_matches('/');
    let html_path = if trimmed.is_empty() {
        "index.html".to_string()
    } else {
        format!("{trimmed}/index.html")
    };

    // Create HTML file at its route path
    let html_file = output_root.join(&html_path);
    if let Some(parent) = html_file.parent() {
        create_dir_all(parent).await?;
    }

    // Media directory at output root for shared deduplication
    let media_dir = output_root.join("media");
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
    DomCodec
        .to_path(
            node,
            &html_file,
            Some(EncodeOptions {
                standalone: Some(true),
                base_url: Some(base_url.to_string()),
                from_path: path.map(|p| p.to_path_buf()),
                to_path: Some(html_file.clone()),
                // Collect and extract media to the shared media directory
                extract_media: Some(media_dir.clone()),
                collect_media: Some(media_dir.clone()),
                // Use static view for site publishing
                view: Some("static".into()),
                // Apply site layout wrapper if configured
                layout: layout.cloned(),
                ..Default::default()
            }),
        )
        .await?;

    // Collect NEW media files created during this document's encoding
    let mut media_files = Vec::new();
    if media_dir.exists() {
        let mut entries = read_dir(&media_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();
            if entry_path.is_file()
                && let Some(filename) = entry_path.file_name().and_then(|n| n.to_str())
                && !existing_media.contains(filename)
            {
                media_files.push((filename.to_string(), entry_path));
            }
        }
    }

    Ok(RenderedDocument {
        source_path: path.map(|p| p.to_path_buf()).unwrap_or_default(),
        route,
        media_files,
    })
}

/// Normalize a path to use forward slashes for storage paths.
///
/// On Windows, `Path::to_string_lossy()` produces backslashes which are
/// invalid for URL routing.
fn normalize_storage_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Convert a route to redirect storage path
///
/// # Examples
/// - `"/"` -> `"redirect.json"`
/// - `"/old-page/"` -> `"old-page/redirect.json"`
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
        assert_eq!(
            normalize_storage_path("assets/style.css"),
            "assets/style.css"
        );
        assert_eq!(
            normalize_storage_path("assets\\style.css"),
            "assets/style.css"
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
    }
}
