//! Site rendering module
//!
//! Renders a site to a directory with HTML documents, media files, static assets,
//! and redirect metadata. The output can be inspected locally or uploaded to
//! Stencila Cloud using the upload module.

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use eyre::Result;
use futures::future::{join_all, try_join_all};
use serde_json::json;
use stencila_node_media::collect_media;
use tokio::{
    fs::{copy, create_dir_all, read_dir, read_to_string, write},
    sync::mpsc,
};

use stencila_codec::{EncodeOptions, stencila_schema::Node};
use stencila_codec_info::Shifter;
use stencila_codec_markdown::to_markdown;
use stencila_codec_utils::git_repo_info;
use stencila_codecs::to_string_with_info;
use stencila_config::{NavItem, RedirectStatus, SiteConfig, SiteFormat};
use stencila_format::Format;
use stencila_node_stabilize::stabilize;

use crate::{
    RouteEntry, RouteType,
    glide::render_glide,
    layout::render_layout,
    links::{build_routes_set, rewrite_links},
    list,
    logo::resolve_logo,
    nav_common::auto_generate_nav,
};

/// A document rendered to HTML
#[derive(Debug)]
struct RenderedDocument {
    /// The source file path
    source_path: PathBuf,

    /// The computed route (e.g., "/report/")
    route: String,
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
/// * `path_filter` - Optional filter by source file path prefix
/// * `source_files` - Optional list of exact source file paths to render
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
    source_files: Option<&[PathBuf]>,
    progress: Option<mpsc::Sender<RenderProgress>>,
    decode_document_fn: F,
) -> Result<RenderResult>
where
    F: Fn(PathBuf, HashMap<String, String>) -> Fut + Send + Sync + 'static,
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

    send_progress!(RenderProgress::WalkingDirectory);

    // List routes, keeping a full set for navigation when doing incremental renders.
    let (all_routes, render_routes) = if source_files.is_some() {
        let all_routes = list(true, true, route_filter, path_filter, None).await?;
        let render_routes = list(true, true, route_filter, path_filter, source_files).await?;
        (all_routes, render_routes)
    } else {
        let all_routes = list(true, true, route_filter, path_filter, source_files).await?;
        let render_routes = all_routes.clone();
        (all_routes, render_routes)
    };

    // Find workspace root for config
    // Load config from workspace
    let config = stencila_config::get()?;

    // Resolve site root for static file paths
    let site_root = if let Some(site) = &config.site
        && let Some(root) = &site.root
    {
        config.workspace_dir.join(root)
    } else {
        config.workspace_dir.clone()
    };

    // Validate site configuration paths (warns about missing images, etc.)
    if let Some(site) = &config.site {
        site.validate_paths(&site_root);
    }

    // Partition routes by type
    let mut document_routes_all: Vec<RouteEntry> = Vec::new();
    let mut document_routes_render: Vec<RouteEntry> = Vec::new();
    let mut static_files: Vec<PathBuf> = Vec::new();
    let mut redirects: Vec<(String, String, RedirectStatus)> = Vec::new();

    for entry in all_routes {
        match entry.route_type {
            RouteType::File | RouteType::Implied | RouteType::Spread => {
                if entry.source_path.is_some() {
                    document_routes_all.push(entry);
                }
            }
            RouteType::Static | RouteType::Redirect => {
                // Ignored for nav/routes_set
            }
        }
    }

    for entry in render_routes {
        match entry.route_type {
            RouteType::File | RouteType::Implied | RouteType::Spread => {
                if entry.source_path.is_some() {
                    document_routes_render.push(entry);
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
        documents: document_routes_render.len(),
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

    // Compute git repo info once (used for edit-source/edit-on components)
    let git_info = git_repo_info(&site_root)?;
    let git_repo_root = git_info.root.as_deref();
    let git_origin = git_info.origin.as_deref();
    let git_branch = git_info.branch.as_deref();

    // Build routes set once for link rewriting (avoid rebuilding for each document)
    let routes_set = build_routes_set(&document_routes_all);

    // Get or generate nav items once (avoid expensive auto-generation per document)
    // If site.nav is configured, use it; otherwise auto-generate from routes
    let nav_items: Vec<NavItem> = if let Some(ref nav) = site_config.nav {
        nav.clone()
    } else {
        auto_generate_nav(&document_routes_all, &None, Some(&site_root))
    };

    // Resolve logo once (avoid per-document filesystem scanning)
    let resolved_logo = resolve_logo(None, site_config.logo.as_ref(), Some(&site_root));

    // Wrap shared data in Arc for parallel access
    let decode_fn = Arc::new(decode_document_fn);
    let progress = Arc::new(progress);
    let processed = Arc::new(AtomicUsize::new(0));
    let total = document_routes_render.len();
    let source_dir = Arc::new(source_dir.to_path_buf());
    let base_url = Arc::new(base_url.to_string());
    let glide_attrs = Arc::new(glide_attrs);
    let site_config = Arc::new(site_config);
    let output_dir = Arc::new(output_dir.to_path_buf());
    let document_routes = Arc::new(document_routes_all);
    let routes_set = Arc::new(routes_set);
    let nav_items = Arc::new(nav_items);
    let resolved_logo = Arc::new(resolved_logo);
    let workspace_id = Arc::new(workspace_id);
    let git_repo_root = Arc::new(git_repo_root.map(|p| p.to_path_buf()));
    let git_origin = Arc::new(git_origin.map(|s| s.to_string()));
    let git_branch = Arc::new(git_branch.map(|s| s.to_string()));

    // Spawn render tasks - using tokio::spawn allows the runtime to schedule
    // blocking operations across its thread pool rather than blocking a single task
    let mut handles = Vec::with_capacity(document_routes_render.len());
    for entry in document_routes_render.iter() {
        let route = entry.route.clone();
        let target = entry.target.clone();
        let source_path = entry.source_path.clone().expect("filtered above");
        let arguments: HashMap<String, String> = entry
            .spread_arguments
            .as_ref()
            .map(|args| args.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();

        // Clone Arcs for this task
        let decode_fn = Arc::clone(&decode_fn);
        let progress = Arc::clone(&progress);
        let processed = Arc::clone(&processed);
        let source_dir = Arc::clone(&source_dir);
        let base_url = Arc::clone(&base_url);
        let glide_attrs = Arc::clone(&glide_attrs);
        let site_config = Arc::clone(&site_config);
        let output_dir = Arc::clone(&output_dir);
        let document_routes = Arc::clone(&document_routes);
        let routes_set = Arc::clone(&routes_set);
        let nav_items = Arc::clone(&nav_items);
        let resolved_logo = Arc::clone(&resolved_logo);
        let workspace_id = Arc::clone(&workspace_id);
        let git_repo_root = Arc::clone(&git_repo_root);
        let git_origin = Arc::clone(&git_origin);
        let git_branch = Arc::clone(&git_branch);

        let handle = tokio::spawn(async move {
            // Skip if source file doesn't exist
            if !source_path.exists() {
                let error_msg = format!("Source file not found: {}", target);
                tracing::warn!("{}", error_msg);
                return Err((source_path, error_msg));
            }

            // Determine relative path and send progress
            let relative_path = source_path
                .strip_prefix(source_dir.as_ref())
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| source_path.to_string_lossy().to_string());

            let index = processed.fetch_add(1, Ordering::SeqCst);
            if let Some(tx) = progress.as_ref() {
                // Use try_send to avoid blocking if channel is full
                let _ = tx.try_send(RenderProgress::EncodingDocument {
                    path: source_path.clone(),
                    relative_path,
                    index,
                    total,
                });
            }

            // Decode document and render it to route HTML
            let result = async {
                let node = decode_fn(source_path.clone(), arguments.clone()).await?;
                render_document_route(
                    &route,
                    node,
                    &base_url,
                    &glide_attrs,
                    &site_config,
                    &source_path,
                    &output_dir,
                    &document_routes,
                    &routes_set,
                    &nav_items,
                    resolved_logo.as_ref().as_ref(),
                    workspace_id.as_deref(),
                    git_repo_root.as_deref(),
                    git_origin.as_deref(),
                    git_branch.as_deref(),
                )
                .await
            }
            .await;

            match result {
                Ok(rendered) => {
                    if let Some(tx) = progress.as_ref() {
                        // Use try_send to avoid blocking if channel is full
                        let _ = tx.try_send(RenderProgress::DocumentEncoded {
                            path: source_path.clone(),
                            route: rendered.route.clone(),
                        });
                    }
                    Ok(rendered)
                }
                Err(e) => {
                    let error_msg = if arguments.is_empty() {
                        e.to_string()
                    } else {
                        format!("Spread variant {arguments:?}: {e}")
                    };
                    if let Some(tx) = progress.as_ref() {
                        // Use try_send to avoid blocking if channel is full
                        let _ = tx.try_send(RenderProgress::DocumentFailed {
                            path: source_path.clone(),
                            error: error_msg.clone(),
                        });
                    }
                    Err((source_path, error_msg))
                }
            }
        });
        handles.push(handle);
    }

    // Await all spawned tasks
    let render_results: Vec<_> = join_all(handles)
        .await
        .into_iter()
        .map(|r| r.expect("task panicked"))
        .collect();

    // Separate successful and failed renders
    let mut docs_rendered: Vec<RenderedDocument> = Vec::new();
    let mut docs_failed: Vec<(PathBuf, String)> = Vec::new();
    for result in render_results {
        match result {
            Ok(rendered) => docs_rendered.push(rendered),
            Err((path, error)) => docs_failed.push((path, error)),
        }
    }

    // Write redirect files
    for (route, target, status) in &redirects {
        render_redirect_route(route, target, status, &output_dir).await?;
    }

    // Copy static files in parallel (preserving relative paths)
    let static_total = static_files.len();
    let static_copied = Arc::new(AtomicUsize::new(0));
    let copy_futures = static_files.iter().map(|static_path| {
        let site_root = site_root.clone();
        let output_dir = Arc::clone(&output_dir);
        let static_copied = Arc::clone(&static_copied);
        let progress = Arc::clone(&progress);
        let static_path = static_path.clone();
        async move {
            let relative = static_path.strip_prefix(&site_root)?;
            let dest_path = output_dir.join(normalize_storage_path(&relative.to_string_lossy()));
            if let Some(parent) = dest_path.parent() {
                create_dir_all(parent).await?;
            }
            copy(&static_path, &dest_path).await?;

            let copied = static_copied.fetch_add(1, Ordering::SeqCst) + 1;
            if let Some(tx) = progress.as_ref() {
                // Use try_send to avoid blocking if channel is full
                let _ = tx.try_send(RenderProgress::CopyingStatic {
                    copied,
                    total: static_total,
                });
            }

            Ok::<_, eyre::Error>(static_path)
        }
    });
    let copied_files: Vec<PathBuf> = try_join_all(copy_futures).await?;

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
        static_files: copied_files,
        media_files_count,
        // Deduplication count is no longer tracked per-document; set to 0
        media_duplicates_eliminated: 0,
    };

    if let Some(tx) = progress.as_ref() {
        // Use try_send to avoid blocking if channel is full
        let _ = tx.try_send(RenderProgress::Complete(result.clone()));
    }

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
    source_file: &Path,
    output_dir: &Path,
    routes: &[RouteEntry],
    routes_set: &HashSet<String>,
    nav_items: &Vec<NavItem>,
    resolved_logo: Option<&stencila_config::LogoConfig>,
    workspace_id: Option<&str>,
    git_repo_root: Option<&Path>,
    git_origin: Option<&str>,
    git_branch: Option<&str>,
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

    // Collect media from source file to shared media directory
    // (deduplication happens automatically via hash-based filenames)
    collect_media(&mut node, Some(source_file), &html_file, &media_dir)?;

    // Determine if source is an index file (affects how relative links are resolved)
    let is_index = source_file
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| matches!(s, "index" | "main" | "README"))
        .unwrap_or(false);

    // Rewrite file-based links to route-based links
    rewrite_links(&mut node, &route, routes_set, is_index);

    // Stabilize node UIDs for deterministic rendering
    // This ensures the same source produces identical HTML/nodemap.json output,
    // enabling effective ETag-based caching and incremental uploads.
    stabilize(&mut node);

    // Render layout for the route
    let layout_html = render_layout(
        site_config,
        &route,
        routes,
        routes_set,
        nav_items,
        resolved_logo,
        workspace_id,
        git_repo_root,
        git_origin,
        git_branch,
    );

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
                let nodemap = encode_info.mapping.to_nodemap(Some(&shifter));
                let nodemap_json = serde_json::to_string(&nodemap)?;
                write(&nodemap_file, nodemap_json).await?;
            }
        }
    }

    Ok(RenderedDocument {
        source_path: source_file.to_path_buf(),
        route,
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
