//! Route listing for Stencila Sites
//!
//! This module provides functionality to list all routes for a site,
//! including configured routes (file, redirect, spread) and implied routes
//! computed from files in the site directory.

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use eyre::{Result, bail, eyre};
use ignore::WalkBuilder;
use indexmap::IndexMap;

use stencila_config::Config;
use stencila_dirs::{closest_stencila_dir, workspace_dir};
use stencila_format::Format;
use stencila_spread::{ParameterValues, Parameters, Run, SpreadMode, apply_template};

/// Category of a file for site listing and push
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileCategory {
    /// A document that should be decoded and encoded to HTML
    Document,
    /// A media file (image/audio/video)
    Media,
    /// A static asset (CSS, JS, fonts, etc.)
    Static,
}

/// Categorize a file based on its format
fn categorize_file(path: &Path) -> FileCategory {
    let format = Format::from_path(path);

    // Media files
    if format.is_media() {
        return FileCategory::Media;
    }

    // Document formats that can be decoded
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
            | Format::M365Docx
            | Format::Odt
            // Data serialization formats
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

    // Everything else is a static asset
    FileCategory::Static
}

/// Determine the URL route for a document file
///
/// First checks route overrides in config, then falls back to file-based routing.
///
/// # File-based routing rules:
/// - Extensions are stripped: `report.ipynb` → `/report/`
/// - Index files (`index.*`, `main.*`, `README.*`) → `/`
/// - Subdirectories are preserved: `docs/report.md` → `/docs/report/`
/// - All routes end with trailing slash
fn determine_route(file_path: &Path, workspace_dir: &Path, config: &Config) -> Result<String> {
    // Determine the base directory for route calculation
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

    // Normalize path separators to forward slashes
    let rel_path_str = rel_path.to_string_lossy().replace('\\', "/");

    // If the file path equals the base directory (site root), route to /
    if rel_path_str.is_empty() {
        return Ok("/".to_string());
    }

    // Check route overrides first
    if let Some(site) = &config.site
        && let Some(routes) = &site.routes
    {
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

/// Walk a directory and categorize files
///
/// Respects `.gitignore` and config exclude patterns.
///
/// Returns a tuple of (document_paths, static_file_paths)
async fn walk_directory(path: &Path) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
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

    // Validate that the requested path is within the site root
    let canonical_path = path.canonicalize()?;
    let canonical_root = site_root.canonicalize()?;
    if !canonical_path.starts_with(&canonical_root) {
        bail!(
            "Path must be within site root. Got: {}\nSite root is: {}",
            path.display(),
            site_root.display()
        );
    }

    // If path is a file, categorize and return it directly
    if path.is_file() {
        let file_path = path.to_path_buf();
        return match categorize_file(&file_path) {
            FileCategory::Document => Ok((vec![file_path], vec![])),
            FileCategory::Static | FileCategory::Media => Ok((vec![], vec![file_path])),
        };
    }

    // Build walker using ignore crate
    let mut builder = WalkBuilder::new(path);
    builder
        .hidden(false) // Don't skip hidden files (allows .htaccess, etc.)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true);

    // Build overrides to exclude sensitive directories
    let mut overrides = ignore::overrides::OverrideBuilder::new(path);

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

    // Add user-configured exclude patterns
    if let Some(site) = &config.site
        && let Some(excludes) = &site.exclude
    {
        for pattern in excludes {
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
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let file_path = entry.path().to_path_buf();

        match categorize_file(&file_path) {
            FileCategory::Document => documents.push(file_path),
            FileCategory::Static | FileCategory::Media => static_files.push(file_path),
        }
    }

    Ok((documents, static_files))
}

/// Generate spread runs from config arguments
///
/// Converts from config's HashMap<String, Vec<String>> to spread crate's Parameters
fn generate_spread_runs(
    mode: stencila_config::SpreadMode,
    arguments: &std::collections::HashMap<String, Vec<String>>,
) -> Result<Vec<IndexMap<String, String>>> {
    // Convert config arguments to spread crate Parameters
    let mut params = Parameters::new();
    for (name, values) in arguments {
        // Join values with comma and parse (spread crate will split them)
        let value_str = values.join(",");
        params.insert(name.clone(), ParameterValues::parse(&value_str));
    }

    // Convert config SpreadMode to spread crate SpreadMode
    let spread_mode = match mode {
        stencila_config::SpreadMode::Grid => SpreadMode::Grid,
        stencila_config::SpreadMode::Zip => SpreadMode::Zip,
    };

    // Generate runs using spread crate
    let runs = match spread_mode {
        SpreadMode::Grid => stencila_spread::generate_runs_grid(&params),
        SpreadMode::Zip => {
            stencila_spread::generate_runs_zip(&params).map_err(|e| eyre!("Spread error: {e}"))?
        }
        SpreadMode::Cases => {
            // Cases mode not supported for site routes
            bail!("Cases mode is not supported for site routes");
        }
    };

    // Convert Run to IndexMap<String, String>
    Ok(runs.into_iter().map(|run| run.values).collect())
}

/// Apply a spread template to generate a route
///
/// Replaces {placeholder} in the template with values from the run
fn apply_spread_template(template: &str, values: &IndexMap<String, String>) -> Result<String> {
    // Create a Run from the values (index doesn't matter for route templates)
    let run = Run::new(1, values.clone());

    // Use spread crate's apply_template
    let result = apply_template(template, &run).map_err(|e| eyre!("Template error: {e}"))?;

    // Ensure route ends with /
    if result.ends_with('/') {
        Ok(result)
    } else {
        Ok(format!("{result}/"))
    }
}

/// The type/source of a route
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteType {
    /// Explicit file route from config
    File,
    /// Redirect route from config
    Redirect,
    /// Spread route (template or expanded variant)
    Spread,
    /// Computed from file path
    Implied,
}

/// A route entry for display and processing
#[derive(Debug, Clone)]
pub struct RouteEntry {
    /// The route path (e.g., "/docs/report/")
    pub route: String,

    /// The type/source of the route
    pub route_type: RouteType,

    /// The target (file path, redirect URL, or spread template file)
    pub target: String,

    /// Whether this is a static file (vs document)
    pub is_static: bool,

    /// The source file path (absolute)
    pub source_path: Option<PathBuf>,

    /// Number of spread variants (for unexpanded spread routes only)
    pub spread_count: Option<usize>,

    /// Spread arguments for this variant (when expanded)
    pub spread_arguments: Option<IndexMap<String, String>>,
}

/// List routes for a site, including both configured and file-implied routes
///
/// # Arguments
/// * `path` - The path to search (typically the workspace or site root)
/// * `expanded` - Whether to expand spread routes into individual variants
/// * `route_filter` - Optional filter by route prefix (e.g., "/docs/")
/// * `path_filter` - Optional filter by source file path prefix (e.g., "docs/")
///
/// # Returns
/// A list of route entries sorted by route path
pub async fn list_routes(
    path: &Path,
    expanded: bool,
    route_filter: Option<&str>,
    path_filter: Option<&str>,
) -> Result<Vec<RouteEntry>> {
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

    let mut routes: Vec<RouteEntry> = Vec::new();
    let mut seen_routes: HashSet<String> = HashSet::new();
    let mut spread_source_files: HashSet<String> = HashSet::new();

    // Collect configured routes
    if let Some(site) = &config.site
        && let Some(configured_routes) = &site.routes
    {
        for (route_path, target) in configured_routes {
            if let Some(file) = target.file() {
                let source_path = site_root.join(file.as_str());
                routes.push(RouteEntry {
                    route: route_path.clone(),
                    route_type: RouteType::File,
                    target: file.as_str().to_string(),
                    spread_count: None,
                    spread_arguments: None,
                    source_path: Some(source_path),
                    is_static: false,
                });
                seen_routes.insert(route_path.clone());
            } else if let Some(redirect) = target.redirect() {
                let status = redirect
                    .status
                    .map(|s| format!(" ({})", s as u16))
                    .unwrap_or_default();
                routes.push(RouteEntry {
                    route: route_path.clone(),
                    route_type: RouteType::Redirect,
                    target: format!("{}{}", redirect.redirect, status),
                    spread_count: None,
                    spread_arguments: None,
                    source_path: None,
                    is_static: false,
                });
                seen_routes.insert(route_path.clone());
            } else if let Some(spread) = target.spread() {
                // Generate spread variants
                let mode = spread.spread.unwrap_or_default();
                let runs = generate_spread_runs(mode, &spread.arguments)?;
                let variant_count = runs.len();
                let source_path = site_root.join(&spread.file);

                if expanded {
                    // Add each variant as a spread route with its arguments
                    for run in runs {
                        let route = apply_spread_template(route_path, &run)?;
                        routes.push(RouteEntry {
                            route: route.clone(),
                            route_type: RouteType::Spread,
                            target: spread.file.clone(),
                            spread_count: None,
                            spread_arguments: Some(run),
                            source_path: Some(source_path.clone()),
                            is_static: false,
                        });
                        seen_routes.insert(route);
                    }
                } else {
                    // Add spread template with count
                    routes.push(RouteEntry {
                        route: route_path.clone(),
                        route_type: RouteType::Spread,
                        target: spread.file.clone(),
                        spread_count: Some(variant_count),
                        spread_arguments: None,
                        source_path: Some(source_path),
                        is_static: false,
                    });
                    // Also add expanded routes to seen set so they don't show as implied
                    for run in runs {
                        let route = apply_spread_template(route_path, &run)?;
                        seen_routes.insert(route);
                    }
                }

                // Track the spread source file so it doesn't appear as an implied route
                spread_source_files.insert(spread.file.clone());
            }
        }
    }

    // Walk directory to find document files and compute implied routes
    if site_root.exists() {
        let (documents, _static_files) = walk_directory(&site_root).await?;

        for doc_path in documents {
            // Get relative path for display
            let rel_path = doc_path
                .strip_prefix(&site_root)
                .unwrap_or(&doc_path)
                .to_string_lossy()
                .replace('\\', "/");

            // Skip files that are spread sources (they're handled by spread routes)
            if spread_source_files.contains(&rel_path) {
                continue;
            }

            // Compute the route for this document
            let route = match determine_route(&doc_path, &workspace_dir, &config) {
                Ok(r) => r,
                Err(_) => continue, // Skip files that can't be routed
            };

            // Skip if this route is already covered by a configured route
            if seen_routes.contains(&route) {
                continue;
            }

            routes.push(RouteEntry {
                route: route.clone(),
                route_type: RouteType::Implied,
                target: rel_path,
                spread_count: None,
                spread_arguments: None,
                source_path: Some(doc_path),
                is_static: false,
            });
            seen_routes.insert(route);
        }
    }

    // Apply filters
    if route_filter.is_some() || path_filter.is_some() {
        routes.retain(|entry| {
            // Check route filter
            if let Some(filter) = route_filter
                && !entry.route.starts_with(filter)
            {
                return false;
            }

            // Check path filter
            if let Some(filter) = path_filter {
                // Match against target (relative path) or source_path
                let matches_target = entry.target.starts_with(filter);
                let matches_source = entry.source_path.as_ref().is_some_and(|p| {
                    p.strip_prefix(&site_root)
                        .ok()
                        .and_then(|rel| rel.to_str())
                        .is_some_and(|s| {
                            s.starts_with(filter) || s.replace('\\', "/").starts_with(filter)
                        })
                });
                if !matches_target && !matches_source {
                    return false;
                }
            }

            true
        });
    }

    // Sort by route path
    routes.sort_by(|a, b| a.route.cmp(&b.route));

    Ok(routes)
}
