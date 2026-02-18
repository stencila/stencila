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
use serde::Deserialize;
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
use stencila_config::{AccessLevel, NavItem, RedirectStatus, SiteConfig, SiteFormat};
use stencila_format::Format;
use stencila_node_stabilize::stabilize;

use crate::{
    RouteEntry, RouteType,
    auto_index::{find_nav_group_children, generate_auto_index_article, get_child_pages_from_nav},
    glide::render_glide,
    layout::render_layout,
    links::{build_routes_set, rewrite_links},
    list::{list, update_nav_items_with_auto_index},
    logo::resolve_logo,
    nav_common::auto_generate_nav,
    search::{
        Breadcrumb, SearchEntry, SearchIndexBuilder, build_breadcrumbs_map,
        extract_entries_with_config, get_breadcrumbs,
    },
};

/// A document rendered to HTML
#[derive(Debug)]
struct RenderedDocument {
    /// The source file path
    source_path: PathBuf,

    /// The computed route (e.g., "/report/")
    route: String,

    /// Search entries extracted from this document (after stabilization)
    search_entries: Vec<SearchEntry>,
}

/// User-defined redirect from a `_redirect.json` file
#[derive(Debug, Deserialize)]
struct RedirectFile {
    /// Target URL or path for the redirect
    location: String,

    /// HTTP status code (301, 302, 303, 307, 308)
    /// Defaults to 302 (TemporaryRedirect) if not specified
    #[serde(default)]
    status: Option<u16>,
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
    web_base: Option<&str>,
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
    // Auto-index routes are now included from list() which applies filters
    let mut document_routes_all: Vec<RouteEntry> = Vec::new();
    let mut document_routes_render: Vec<RouteEntry> = Vec::new();
    let mut auto_index_routes_all: Vec<RouteEntry> = Vec::new();
    let mut auto_index_routes_render: Vec<RouteEntry> = Vec::new();
    let mut static_files: Vec<PathBuf> = Vec::new();
    let mut redirects: Vec<(String, String, RedirectStatus)> = Vec::new();

    for entry in all_routes {
        match entry.route_type {
            _ if entry.route_type.is_document() && entry.source_path.is_some() => {
                document_routes_all.push(entry);
            }
            RouteType::AutoIndex => {
                auto_index_routes_all.push(entry);
            }
            _ => {
                // Static and Redirect are handled separately
            }
        }
    }

    for entry in render_routes {
        match entry.route_type {
            _ if entry.route_type.is_document() && entry.source_path.is_some() => {
                document_routes_render.push(entry);
            }
            RouteType::Static => {
                if let Some(path) = entry.source_path {
                    static_files.push(path);
                }
            }
            RouteType::AutoIndex => {
                auto_index_routes_render.push(entry);
            }
            _ => {
                // Redirect handled separately
            }
        }
    }

    // Discover user-defined redirect files (these take precedence over config)
    let user_redirects = discover_user_redirects(&site_root, &config).await?;
    let user_redirect_routes: HashSet<String> = user_redirects
        .iter()
        .map(|(route, _, _)| route.clone())
        .collect();

    // Add site-level redirects from config (skipping routes with user-defined redirects)
    if let Some(site) = &config.site
        && let Some(routes) = &site.routes
    {
        for (route_path, target) in routes {
            if let Some(redirect_config) = target.redirect() {
                // Normalize route for comparison (ensures trailing slash consistency)
                let normalized_route = normalize_route(route_path);
                // Skip if user has defined a redirect for this route
                if !user_redirect_routes.contains(&normalized_route) {
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

    // Add user-defined redirects
    redirects.extend(user_redirects);

    send_progress!(RenderProgress::FilesFound {
        documents: document_routes_render.len() + auto_index_routes_render.len(),
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
    let mut nav_items: Vec<NavItem> = if let Some(ref nav) = site_config.nav {
        nav.clone()
    } else {
        auto_generate_nav(&document_routes_all, &None, Some(&site_root))
    };

    // Auto-index routes come from list() which already applies filters
    // auto_index_routes_all: for navigation (all auto-index routes matching route_filter/path_filter)
    // auto_index_routes_render: for rendering (filtered by source_files too, if specified)

    // Add auto-index routes to the document routes and routes_set for navigation
    let mut routes_set = routes_set;
    for entry in &auto_index_routes_all {
        document_routes_all.push(entry.clone());
        routes_set.insert(entry.route.clone());
    }

    // Build set of auto-index routes for updating nav items
    let auto_index_route_set: HashSet<String> = auto_index_routes_all
        .iter()
        .map(|e| e.route.clone())
        .collect();

    // Update nav items to include routes for auto-index pages
    // This makes auto-index routes clickable in nav-tree, breadcrumbs, etc.
    if !auto_index_routes_all.is_empty() {
        update_nav_items_with_auto_index(&mut nav_items, &auto_index_route_set);
    }

    // Build breadcrumbs map once for search indexing (after nav items are updated)
    let breadcrumbs_map = build_breadcrumbs_map(&nav_items);

    // Resolve logo once (avoid per-document filesystem scanning)
    let resolved_logo = resolve_logo(None, site_config.logo.as_ref(), Some(&site_root));

    // Wrap shared data in Arc for parallel access
    let decode_fn = Arc::new(decode_document_fn);
    let progress = Arc::new(progress);
    let processed = Arc::new(AtomicUsize::new(0));
    let total = document_routes_render.len() + auto_index_routes_render.len();
    let source_dir = Arc::new(source_dir.to_path_buf());
    let base_url = Arc::new(base_url.to_string());
    let web_base = Arc::new(web_base.map(|s| s.to_string()));
    let glide_attrs = Arc::new(glide_attrs);
    let site_config = Arc::new(site_config);
    let output_dir = Arc::new(output_dir.to_path_buf());
    let document_routes = Arc::new(document_routes_all);
    let routes_set = Arc::new(routes_set);
    let nav_items = Arc::new(nav_items);
    let breadcrumbs_map = Arc::new(breadcrumbs_map);
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
        let web_base = Arc::clone(&web_base);
        let glide_attrs = Arc::clone(&glide_attrs);
        let site_config = Arc::clone(&site_config);
        let output_dir = Arc::clone(&output_dir);
        let document_routes = Arc::clone(&document_routes);
        let routes_set = Arc::clone(&routes_set);
        let nav_items = Arc::clone(&nav_items);
        let breadcrumbs_map = Arc::clone(&breadcrumbs_map);
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
                    web_base.as_deref(),
                    &glide_attrs,
                    &site_config,
                    &source_path,
                    &output_dir,
                    &document_routes,
                    &routes_set,
                    &nav_items,
                    breadcrumbs_map.as_ref(),
                    resolved_logo.as_ref().as_ref(),
                    workspace_id.as_deref(),
                    git_repo_root.as_deref(),
                    git_origin.as_deref(),
                    git_branch.as_deref(),
                    &arguments,
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

    // Render auto-index routes (directories without content files)
    // Uses auto_index_routes_render which respects filters (route_filter, path_filter, source_files)
    for entry in &auto_index_routes_render {
        // Update progress for auto-index routes (use route as synthetic path)
        let index = processed.fetch_add(1, Ordering::SeqCst);
        if let Some(tx) = progress.as_ref() {
            let _ = tx.try_send(RenderProgress::EncodingDocument {
                path: PathBuf::from(&entry.route),
                relative_path: format!("[auto-index] {}", entry.route),
                index,
                total,
            });
        }

        let result = render_auto_index_route(
            &entry.route,
            &base_url,
            web_base.as_deref(),
            &glide_attrs,
            &site_config,
            &output_dir,
            &document_routes,
            &routes_set,
            &nav_items,
            &breadcrumbs_map,
            resolved_logo.as_ref().as_ref(),
            workspace_id.as_deref(),
            git_repo_root.as_deref(),
            git_origin.as_deref(),
            git_branch.as_deref(),
        )
        .await;

        match result {
            Ok(rendered) => {
                if let Some(tx) = progress.as_ref() {
                    let _ = tx.try_send(RenderProgress::DocumentEncoded {
                        path: PathBuf::from(&entry.route),
                        route: rendered.route.clone(),
                    });
                }
                docs_rendered.push(rendered);
            }
            Err(e) => {
                tracing::warn!("Failed to render auto-index route {}: {}", entry.route, e);
                if let Some(tx) = progress.as_ref() {
                    let _ = tx.try_send(RenderProgress::DocumentFailed {
                        path: PathBuf::from(&entry.route),
                        error: e.to_string(),
                    });
                }
            }
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

    // Generate search index if enabled in config
    // Entries were extracted from each document after stabilization (node IDs assigned)
    if let Some(search_spec) = site_config.search.as_ref()
        && search_spec.is_enabled()
    {
        let mut builder = SearchIndexBuilder::new().with_fuzzy(search_spec.is_fuzzy_enabled());
        for doc in &docs_rendered {
            builder.add_entries(doc.search_entries.clone());
        }
        if let Err(e) = builder.write(&output_dir).await {
            tracing::warn!("Failed to write search index: {}", e);
        }
    }

    // Generate files index if uploads are enabled
    if let Some(uploads_spec) = site_config.uploads.as_ref()
        && uploads_spec.is_enabled()
    {
        let uploads_config = uploads_spec.to_config();
        if let Err(e) = crate::files::generate_files_index(
            &config.workspace_dir,
            &site_root,
            &output_dir,
            uploads_config.extensions.as_deref(),
        )
        .await
        {
            tracing::warn!("Failed to write files index: {}", e);
        }
    }

    // Generate access index if access restrictions are configured
    if let Some(access_config) = site_config.access.as_ref()
        && let Err(e) = crate::access::generate_access_index(access_config, &output_dir).await
    {
        tracing::warn!("Failed to write access index: {}", e);
    }

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
    web_base: Option<&str>,
    glide_attrs: &str,
    site_config: &SiteConfig,
    source_file: &Path,
    output_dir: &Path,
    routes: &[RouteEntry],
    routes_set: &HashSet<String>,
    nav_items: &Vec<NavItem>,
    breadcrumbs_map: &HashMap<String, Vec<Breadcrumb>>,
    resolved_logo: Option<&stencila_config::LogoConfig>,
    workspace_id: Option<&str>,
    git_repo_root: Option<&Path>,
    git_origin: Option<&str>,
    git_branch: Option<&str>,
    spread_arguments: &HashMap<String, String>,
) -> Result<RenderedDocument> {
    // Normalize route to ensure trailing slash
    let route = normalize_route(route);

    // Convert route to HTML file path
    let html_file = route_to_html_path(&route, output_dir);

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

    // Extract search entries from the stabilized node (node IDs are now assigned)
    let search_entries = extract_search_entries(&node, &route, site_config, breadcrumbs_map);

    // Render layout for the route
    let layout_html = render_layout(
        site_config,
        &route,
        routes,
        routes_set,
        nav_items,
        breadcrumbs_map,
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
            web_base: web_base.map(|s| s.to_string()),
            view: Some("site".to_string()),
            ..Default::default()
        }),
        Some(site),
    )
    .await?;

    // Inject spread-args attribute on root element if this is a spread route.
    // The root element has a boolean `root` attribute (e.g., `<stencila-article root ...>`).
    // We match ` root` followed by a space or `>` to avoid matching content text.
    let html = if !spread_arguments.is_empty() {
        let spread_json = serde_json::to_string(spread_arguments)?;
        let escaped = html_escape::encode_double_quoted_attribute(&spread_json);
        let spread_attr = format!(" root spread-args=\"{escaped}\"");

        // Try matching " root " (followed by another attribute) first, then " root>" (end of tag)
        if html.contains(" root ") {
            html.replacen(" root ", &format!("{spread_attr} "), 1)
        } else if html.contains(" root>") {
            html.replacen(" root>", &format!("{spread_attr}>"), 1)
        } else {
            // Fallback: root attribute not found in expected format, skip injection
            tracing::warn!(
                "Could not inject spread-args: root attribute not found in expected format"
            );
            html
        }
    } else {
        html
    };

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
    let is_spread = !spread_arguments.is_empty();
    let should_generate_nodemap = site_config
        .reviews
        .as_ref()
        .map(|reviews| {
            reviews.is_enabled()
                && crate::layout::should_show_reviews_for_route(
                    &route,
                    &reviews.to_config(),
                    is_spread,
                )
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
        search_entries,
    })
}

/// Render an auto-generated index page for a directory without content
///
/// Creates an Article with a list of child page links and renders it to HTML.
#[allow(clippy::too_many_arguments)]
async fn render_auto_index_route(
    route: &str,
    base_url: &str,
    web_base: Option<&str>,
    glide_attrs: &str,
    site_config: &SiteConfig,
    output_dir: &Path,
    routes: &[RouteEntry],
    routes_set: &HashSet<String>,
    nav_items: &Vec<NavItem>,
    breadcrumbs_map: &HashMap<String, Vec<Breadcrumb>>,
    resolved_logo: Option<&stencila_config::LogoConfig>,
    workspace_id: Option<&str>,
    git_repo_root: Option<&Path>,
    git_origin: Option<&str>,
    git_branch: Option<&str>,
) -> Result<RenderedDocument> {
    // Normalize route to ensure trailing slash
    let route = normalize_route(route);

    // Find children for this route from the nav tree
    let children = if let Some(child_items) = find_nav_group_children(nav_items, &route) {
        get_child_pages_from_nav(child_items)
    } else {
        Vec::new()
    };

    // Generate the Article node
    let mut node = generate_auto_index_article(&route, children);

    // Stabilize node UIDs for deterministic rendering
    stabilize(&mut node);

    // Extract search entries from the stabilized node
    let search_entries = extract_search_entries(&node, &route, site_config, breadcrumbs_map);

    // Render layout for the route
    let layout_html = render_layout(
        site_config,
        &route,
        routes,
        routes_set,
        nav_items,
        breadcrumbs_map,
        resolved_logo,
        workspace_id,
        git_repo_root,
        git_origin,
        git_branch,
    );

    // Generate site body
    let site = format!("<body{glide_attrs}>\n{layout_html}\n</body>");

    // Generate standalone html with "site" view
    let (html, ..) = stencila_codec_dom::encode(
        &node,
        Some(EncodeOptions {
            base_url: Some(base_url.to_string()),
            web_base: web_base.map(|s| s.to_string()),
            view: Some("site".to_string()),
            ..Default::default()
        }),
        Some(site),
    )
    .await?;

    // Convert route to HTML file path
    let html_file = route_to_html_path(&route, output_dir);

    // Write to output HTML file
    if let Some(parent) = html_file.parent() {
        create_dir_all(parent).await?;
    }
    write(&html_file, html).await?;

    // Use a synthetic source path for auto-index routes
    let source_path = PathBuf::from(format!("[auto-index:{route}]"));

    Ok(RenderedDocument {
        source_path,
        route,
        search_entries,
    })
}

/// Normalize a route to always have a trailing slash
///
/// This ensures consistent route comparison between user-defined redirects
/// (which always have trailing slashes) and config-defined routes (which may not).
fn normalize_route(route: &str) -> String {
    if route == "/" || route.ends_with('/') {
        route.to_string()
    } else {
        format!("{route}/")
    }
}

/// Convert a route to an HTML output file path
///
/// For example:
/// - "/" -> "index.html"
/// - "/docs/" -> "docs/index.html"
/// - "/docs/guide/" -> "docs/guide/index.html"
fn route_to_html_path(route: &str, output_dir: &Path) -> PathBuf {
    let trimmed = route.trim_start_matches('/').trim_end_matches('/');
    let relative_path = if trimmed.is_empty() {
        "index.html".to_string()
    } else {
        format!("{trimmed}/index.html")
    };
    output_dir.join(relative_path)
}

/// Extract search entries from a node with access level tagging
///
/// Handles the common pattern of:
/// 1. Checking if search is enabled
/// 2. Getting breadcrumbs for the route
/// 3. Extracting search entries
/// 4. Tagging with access level
fn extract_search_entries(
    node: &Node,
    route: &str,
    site_config: &SiteConfig,
    breadcrumbs_map: &HashMap<String, Vec<Breadcrumb>>,
) -> Vec<SearchEntry> {
    let Some(search_spec) = site_config.search.as_ref() else {
        return Vec::new();
    };

    if !search_spec.is_enabled() {
        return Vec::new();
    }

    let breadcrumbs = get_breadcrumbs(route, breadcrumbs_map);
    let entries = extract_entries_with_config(node, route, breadcrumbs, &search_spec.to_config());

    // Determine access level for this route
    let access_level = site_config
        .access
        .as_ref()
        .map(|config| config.get_access_level(route))
        .unwrap_or(AccessLevel::Public);

    // Tag all entries with the route's access level
    entries
        .into_iter()
        .map(|e| e.with_access_level(access_level))
        .collect()
}

/// Load a user-defined redirect file from a directory if one exists
///
/// Looks for `_redirect.json` in the specified directory and parses it.
/// Returns None if the file doesn't exist or fails to parse.
async fn load_user_redirect(dir: &Path) -> Option<(String, RedirectStatus)> {
    let redirect_file = dir.join("_redirect.json");
    let content = read_to_string(&redirect_file).await.ok()?;

    match serde_json::from_str::<RedirectFile>(&content) {
        Ok(redirect) => {
            let status = redirect
                .status
                .and_then(|s| RedirectStatus::try_from(s).ok())
                .unwrap_or(RedirectStatus::TemporaryRedirect);
            Some((redirect.location, status))
        }
        Err(err) => {
            tracing::warn!(
                "Failed to parse redirect file {}: {}",
                redirect_file.display(),
                err
            );
            None
        }
    }
}

/// Discover user-defined redirect files in the site directory
///
/// Walks the site root and finds all `_redirect.json` files,
/// converting their paths to routes. Respects site.exclude patterns
/// and other standard exclusions.
async fn discover_user_redirects(
    site_root: &Path,
    config: &stencila_config::Config,
) -> Result<Vec<(String, String, RedirectStatus)>> {
    use ignore::{WalkBuilder, overrides::OverrideBuilder};

    let site_root = site_root.to_path_buf();
    let excludes: Vec<String> = config
        .site
        .as_ref()
        .and_then(|s| s.exclude.clone())
        .unwrap_or_default();

    // Walk directory in blocking task since WalkBuilder is sync
    let redirect_paths: Vec<PathBuf> = tokio::task::spawn_blocking(move || {
        let mut builder = WalkBuilder::new(&site_root);
        builder
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true);

        // Build overrides to exclude sensitive directories (same as list.rs)
        let mut overrides = OverrideBuilder::new(&site_root);
        const SENSITIVE_PATTERNS: &[&str] = &[
            "!.git/",
            "!.stencila/",
            "!.env",
            "!.env.*",
            "!node_modules/",
        ];
        for pattern in SENSITIVE_PATTERNS {
            if overrides.add(pattern).is_err() {
                continue;
            }
        }

        // Add user-configured exclude patterns
        for pattern in &excludes {
            let exclude_pattern = format!("!{pattern}");
            if overrides.add(&exclude_pattern).is_err() {
                continue;
            }
        }

        if let Ok(built) = overrides.build() {
            builder.overrides(built);
        }

        let mut paths = Vec::new();
        for entry in builder.build().flatten() {
            if let Some(file_name) = entry.file_name().to_str()
                && file_name == "_redirect.json"
            {
                paths.push(entry.into_path());
            }
        }
        paths
    })
    .await?;

    // Load each redirect file asynchronously
    let site_root = config
        .site
        .as_ref()
        .and_then(|s| s.root.as_ref())
        .map(|r| config.workspace_dir.join(r))
        .unwrap_or_else(|| config.workspace_dir.clone());

    let mut redirects = Vec::new();
    for path in redirect_paths {
        let dir = path.parent().unwrap_or(&site_root);

        // Convert directory path to route
        let route = if dir == site_root {
            "/".to_string()
        } else if let Ok(rel) = dir.strip_prefix(&site_root) {
            format!("/{}/", rel.to_string_lossy().replace('\\', "/"))
        } else {
            continue;
        };

        if let Some((target, status)) = load_user_redirect(dir).await {
            redirects.push((route, target, status));
        }
    }

    Ok(redirects)
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
        "_redirect.json".to_string()
    } else {
        format!("{trimmed}/_redirect.json")
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
    use std::fs;
    use tempfile::TempDir;

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
    fn test_normalize_route() {
        assert_eq!(normalize_route("/"), "/");
        assert_eq!(normalize_route("/path"), "/path/");
        assert_eq!(normalize_route("/path/"), "/path/");
        assert_eq!(normalize_route("/deep/nested/path"), "/deep/nested/path/");
        assert_eq!(normalize_route("/deep/nested/path/"), "/deep/nested/path/");
    }

    #[tokio::test]
    async fn test_load_user_redirect_valid() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let redirect_content = r#"{"location": "/new-path/", "status": 301}"#;
        fs::write(temp_dir.path().join("_redirect.json"), redirect_content)?;

        let result = load_user_redirect(temp_dir.path()).await;
        let (location, status) = result.ok_or_else(|| eyre::eyre!("expected redirect"))?;
        assert_eq!(location, "/new-path/");
        assert_eq!(status, RedirectStatus::MovedPermanently);
        Ok(())
    }

    #[tokio::test]
    async fn test_load_user_redirect_default_status() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let redirect_content = r#"{"location": "https://example.com"}"#;
        fs::write(temp_dir.path().join("_redirect.json"), redirect_content)?;

        let result = load_user_redirect(temp_dir.path()).await;
        let (location, status) = result.ok_or_else(|| eyre::eyre!("expected redirect"))?;
        assert_eq!(location, "https://example.com");
        assert_eq!(status, RedirectStatus::TemporaryRedirect);
        Ok(())
    }

    #[tokio::test]
    async fn test_load_user_redirect_invalid_status() -> Result<()> {
        let temp_dir = TempDir::new()?;
        // 404 is not a valid redirect status, should fall back to default
        let redirect_content = r#"{"location": "/path/", "status": 404}"#;
        fs::write(temp_dir.path().join("_redirect.json"), redirect_content)?;

        let result = load_user_redirect(temp_dir.path()).await;
        let (_, status) = result.ok_or_else(|| eyre::eyre!("expected redirect"))?;
        assert_eq!(status, RedirectStatus::TemporaryRedirect);
        Ok(())
    }

    #[tokio::test]
    async fn test_load_user_redirect_missing() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let result = load_user_redirect(temp_dir.path()).await;
        assert!(result.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn test_discover_user_redirects() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create redirect at root
        fs::write(
            temp_dir.path().join("_redirect.json"),
            r#"{"location": "/home/"}"#,
        )?;

        // Create redirect in subdirectory
        let subdir = temp_dir.path().join("old-section");
        fs::create_dir_all(&subdir)?;
        fs::write(
            subdir.join("_redirect.json"),
            r#"{"location": "/new-section/", "status": 301}"#,
        )?;

        // Create a minimal config pointing to the temp directory
        let config = stencila_config::Config {
            workspace_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let redirects = discover_user_redirects(temp_dir.path(), &config).await?;
        assert_eq!(redirects.len(), 2);

        // Check routes exist (order not guaranteed)
        let routes: Vec<_> = redirects.iter().map(|(r, _, _)| r.as_str()).collect();
        assert!(routes.contains(&"/"));
        assert!(routes.contains(&"/old-section/"));
        Ok(())
    }

    #[tokio::test]
    async fn test_discover_user_redirects_respects_excludes() -> Result<()> {
        let temp_dir = TempDir::new()?;

        // Create redirect at root (should be included)
        fs::write(
            temp_dir.path().join("_redirect.json"),
            r#"{"location": "/home/"}"#,
        )?;

        // Create redirect in excluded directory (should be excluded)
        let excluded_dir = temp_dir.path().join("excluded");
        fs::create_dir_all(&excluded_dir)?;
        fs::write(
            excluded_dir.join("_redirect.json"),
            r#"{"location": "/should-not-appear/"}"#,
        )?;

        // Create config with exclude pattern
        let config = stencila_config::Config {
            workspace_dir: temp_dir.path().to_path_buf(),
            site: Some(stencila_config::SiteConfig {
                exclude: Some(vec!["excluded/".to_string()]),
                ..Default::default()
            }),
            ..Default::default()
        };

        let redirects = discover_user_redirects(temp_dir.path(), &config).await?;

        // Should only have the root redirect, not the excluded one
        assert_eq!(redirects.len(), 1);
        let routes: Vec<_> = redirects.iter().map(|(r, _, _)| r.as_str()).collect();
        assert!(routes.contains(&"/"));
        assert!(!routes.iter().any(|r| r.contains("excluded")));
        Ok(())
    }
}
