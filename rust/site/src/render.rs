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
use stencila_node_media::collect_media;
use tokio::{
    fs::{copy, create_dir_all, read_dir, read_to_string, write},
    sync::mpsc,
};

use stencila_codec::{EncodeOptions, stencila_schema::Node};
use stencila_codec_info::Shifter;
use stencila_codec_markdown::to_markdown;
use stencila_codecs::to_string_with_info;
use stencila_config::{RedirectStatus, SiteConfig, SiteFormat};
use stencila_dirs::{closest_stencila_dir, workspace_dir};
use stencila_format::Format;

use crate::{
    RouteEntry, RouteType, glide::render_glide, layout::render_layout, links::rewrite_links, list,
};

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
/// Walks the source directory, encodes documents to HTML, and writes all files
/// (HTML, redirects, media, static) to the output directory.
///
/// # Arguments
/// * `source_dir` - The source directory to render
/// * `output_dir` - The output directory for rendered files
/// * `base_url` - Base URL for the site (used in HTML for absolute links)
/// * `route_filter` - Optional filter to only render routes matching prefix
/// * `path_filter` - Optional filter to only render files matching path prefix
/// * `progress` - Optional channel for progress events
/// * `decode_document_fn` - Async function to decode a document from a path
///
/// # Error Handling
/// - **Continue on error**: If one document fails, log it and continue with
///   next
#[allow(clippy::too_many_arguments)]
pub async fn render<F, Fut>(
    source_dir: &Path,
    output_dir: &Path,
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
    let all_routes = list(source_dir, true, true, route_filter, path_filter).await?;

    // Find workspace root for config
    let stencila_dir = closest_stencila_dir(source_dir, true).await?;
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

    // Validate site configuration paths (warns about missing images, etc.)
    if let Some(site) = &config.site {
        site.validate_paths(&site_root);
    }

    // Partition routes by type
    // TODO: document_routes is sorted by route path (alphabetically), which is used for
    // prev/next navigation. This may not match custom nav-tree ordering or groupings.
    // Consider allowing navigation order to be derived from nav-tree structure instead.
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
    create_dir_all(output_dir).await?;

    // Render glide attributes (used for all document routes)
    let glide_config = config.site.as_ref().and_then(|site| site.glide.as_ref());
    let glide_attrs = render_glide(glide_config);

    // Get workspace ID (for edit-on component)
    let workspace_id = config
        .workspace
        .as_ref()
        .and_then(|w| w.id.as_deref())
        .map(String::from);

    // Get site configuration (used for all document routes)
    let site_config = config.site.unwrap_or_default();

    // Render all documents
    let mut docs_rendered: Vec<RenderedDocument> = Vec::new();
    let mut docs_failed: Vec<(PathBuf, String)> = Vec::new();
    let mut media_created: usize = 0;
    for (
        index,
        RouteEntry {
            route,
            target,
            source_path,
            spread_arguments,
            ..
        },
    ) in document_routes.iter().enumerate()
    {
        let source_path = source_path.as_ref().expect("filtered above");

        // Skip if source file doesn't exist
        if !source_path.exists() {
            let error_msg = format!("Source file not found: {}", target);
            tracing::warn!("{}", error_msg);
            docs_failed.push((source_path.clone(), error_msg));
            continue;
        }

        // Determine relative path
        let relative_path = source_path
            .strip_prefix(source_dir)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| source_path.to_string_lossy().to_string());

        send_progress!(RenderProgress::EncodingDocument {
            path: source_path.clone(),
            relative_path,
            index,
            total: document_routes.len(),
        });

        // Convert spread arguments from IndexMap to HashMap
        let arguments: HashMap<String, String> = spread_arguments
            .as_ref()
            .map(|args| args.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();

        // Decode document and render it to route HTML
        let result = async {
            let node = decode_document_fn(source_path.clone(), arguments.clone()).await?;
            render_document_route(
                route,
                node,
                base_url,
                &glide_attrs,
                &site_config,
                &site_root,
                source_path,
                output_dir,
                &document_routes,
                workspace_id.as_deref(),
            )
            .await
        }
        .await;

        match result {
            Ok(rendered) => {
                media_created += rendered.media_files.len();
                send_progress!(RenderProgress::DocumentEncoded {
                    path: source_path.clone(),
                    route: rendered.route.clone(),
                });
                docs_rendered.push(rendered);
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
                docs_failed.push((source_path.clone(), error_msg));
            }
        }
    }

    // Write redirect files
    for (route, target, status) in &redirects {
        render_redirect_route(route, target, status, output_dir).await?;
    }

    // Copy static files (preserving relative paths)
    for (index, static_path) in static_files.iter().enumerate() {
        let relative = static_path.strip_prefix(&site_root)?;
        let dest_path = output_dir.join(normalize_storage_path(&relative.to_string_lossy()));
        if let Some(parent) = dest_path.parent() {
            create_dir_all(parent).await?;
        }
        copy(static_path, &dest_path).await?;

        send_progress!(RenderProgress::CopyingStatic {
            copied: index + 1,
            total: static_files.len(),
        });
    }

    // Count unique media files
    let media_dir = output_dir.join("media");
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

    let duplicate_count = media_created.saturating_sub(media_files_count);

    let result = RenderResult {
        documents_ok: docs_rendered
            .iter()
            .map(|d| (d.source_path.clone(), d.route.clone()))
            .collect(),
        documents_failed: docs_failed,
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

/// Render a file-based route to an index.html file
#[allow(clippy::too_many_arguments)]
async fn render_document_route(
    route: &str,
    mut node: Node,
    base_url: &str,
    glide_attrs: &str,
    site_config: &SiteConfig,
    site_root: &Path,
    source_file: &Path,
    output_dir: &Path,
    routes: &[RouteEntry],
    workspace_id: Option<&str>,
) -> Result<RenderedDocument> {
    // Ensure route ends with /
    let route = if route.ends_with('/') {
        route.to_string()
    } else {
        format!("{route}/")
    };

    // Convert route to HTML file path (e.g., "/docs/report/" -> "docs/report/index.html")
    let trimmed = route.trim_start_matches('/').trim_end_matches('/');
    let html_file = if trimmed.is_empty() {
        "index.html".to_string()
    } else {
        format!("{trimmed}/index.html")
    };
    let html_file = output_dir.join(&html_file);

    // Create media directory at output root for shared deduplication
    let media_dir = output_dir.join("media");
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

    // Collect media from source file to shared media directory
    collect_media(&mut node, Some(source_file), &html_file, &media_dir)?;

    // Rewrite file-based links to route-based links
    rewrite_links(&mut node, &route, routes);

    // Render layout for the route
    let layout_html = render_layout(site_config, site_root, &route, routes, workspace_id);

    // Generate site body
    let site = format!("<body{glide_attrs}>\n{layout_html}\n</body>");

    // Generate standalone html with "site" view (includes node IDs for site review)
    let (html, ..) = stencila_codec_dom::encode(
        &node,
        Some(EncodeOptions {
            base_url: Some(base_url.to_string()),
            view: Some("site".to_string()),
            ..Default::default()
        }),
        Some(site),
    )
    .await?;

    // Write to output HTML file
    if let Some(parent) = html_file.parent() {
        create_dir_all(parent).await?;
    }
    write(&html_file, html).await?;

    // Generate markdown if format is enabled
    if site_config.is_format_enabled(SiteFormat::Md) {
        let md_file = html_file.with_file_name("page.md");
        let markdown = to_markdown(&node);
        write(&md_file, markdown).await?;
    }

    // Generate nodemap.json for page review feature (only if reviews enabled for this route)
    // Re-encode to source format to get mapping, then use Shifter to translate
    // positions from generated content back to original source file positions
    let should_generate_nodemap = site_config
        .reviews
        .as_ref()
        .map(|reviews| {
            reviews.is_enabled()
                && crate::layout::should_show_reviews_for_route(&route, &reviews.to_config())
        })
        .unwrap_or(false);

    if should_generate_nodemap {
        let source_format = Format::from_path(source_file);
        if !source_format.is_binary() && !source_format.is_lossless() {
            let (generated_source, encode_info) = to_string_with_info(
                &node,
                Some(EncodeOptions {
                    format: Some(source_format),
                    ..Default::default()
                }),
            )
            .await?;

            if !encode_info.mapping.entries().is_empty() {
                // Read original source content
                let original_source = read_to_string(source_file).await?;

                // Create shifter to translate indices from generated to original source
                let shifter = Shifter::new(&original_source, &generated_source);

                let nodemap_file = html_file.with_file_name("nodemap.json");
                let nodemap = encode_info
                    .mapping
                    .to_nodemap(&original_source, Some(&shifter));
                let nodemap_json = serde_json::to_string(&nodemap)?;
                write(&nodemap_file, nodemap_json).await?;
            }
        }
    }

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
        source_path: source_file.to_path_buf(),
        route,
        media_files,
    })
}

/// Render a redirect route to a JSON file
async fn render_redirect_route(
    route: &str,
    target: &str,
    status: &RedirectStatus,
    output_dir: &Path,
) -> Result<()> {
    let trimmed = route.trim_matches('/');
    let storage_path = if trimmed.is_empty() {
        "redirect.json".to_string()
    } else {
        format!("{trimmed}/redirect.json")
    };
    let redirect_path = output_dir.join(&storage_path);
    if let Some(parent) = redirect_path.parent() {
        create_dir_all(parent).await?;
    }

    let redirect_content = serde_json::to_string(&json!({
        "location": target,
        "status": status
    }))?;
    write(&redirect_path, redirect_content).await?;

    Ok(())
}

/// Normalize a path to use forward slashes for storage paths.
///
/// On Windows, `Path::to_string_lossy()` produces backslashes which are
/// invalid for URL routing.
fn normalize_storage_path(path: &str) -> String {
    path.replace('\\', "/")
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
}
