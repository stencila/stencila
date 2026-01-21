//! Link rewriting for static site generation
//!
//! This module transforms file-based link targets (e.g., `other.md`) into
//! route-based links (e.g., `/other/`) that work on the generated static site.

use std::{collections::HashSet, path::Path};

use stencila_codec::stencila_schema::{Inline, VisitorMut, WalkControl, WalkNode};

use crate::RouteEntry;

/// Build a HashSet of route strings from route entries.
///
/// This should be called once and the result reused across multiple documents
/// to avoid repeated allocations.
pub fn build_routes_set(routes: &[RouteEntry]) -> HashSet<String> {
    routes.iter().map(|r| r.route.clone()).collect()
}

/// Rewrites file-based link targets to route-based links for static site generation
pub struct LinkRewriter<'a> {
    /// Current document's route (e.g., "/docs/guide/")
    current_route: String,
    /// Set of valid routes in the site (borrowed to avoid cloning)
    routes: &'a HashSet<String>,
    /// Whether the current route represents an index file (index.md, main.md, README.md)
    /// This affects how relative links are resolved.
    is_index: bool,
}

impl<'a> LinkRewriter<'a> {
    pub fn new(current_route: &str, routes: &'a HashSet<String>, is_index: bool) -> Self {
        Self {
            current_route: current_route.to_string(),
            routes,
            is_index,
        }
    }

    /// Transform a link target from file path to route
    fn transform_target(&self, target: &str) -> Option<String> {
        // Skip internal anchors
        if target.starts_with('#') {
            return None;
        }

        // Skip protocol-relative URLs (//cdn.example.com/...)
        if target.starts_with("//") {
            return None;
        }

        // Skip any URI scheme (http:, https:, ftp:, ws:, wss:, data:, mailto:, tel:, file:, javascript:, etc.)
        // A URI scheme is letters followed by : before the first / (if any)
        if let Some(colon_pos) = target.find(':') {
            let before_colon = &target[..colon_pos];
            // Check if everything before : is a valid scheme (letters, digits, +, -, .)
            // and there's no / before the : (to avoid matching Windows paths like C:\)
            if !before_colon.contains('/')
                && before_colon
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
                && before_colon
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_alphabetic())
            {
                return None;
            }
        }

        // Split anchor from path (e.g., "file.md#section" -> ("file.md", Some("#section")))
        let (path, anchor) = match target.find('#') {
            Some(pos) => (&target[..pos], Some(&target[pos..])),
            None => (target, None),
        };

        // Split query string (e.g., "file.md?v=1" -> ("file.md", Some("?v=1")))
        let (path, query) = match path.find('?') {
            Some(pos) => (&path[..pos], Some(&path[pos..])),
            None => (path, None),
        };

        // Skip if path is empty (pure anchor or query)
        if path.is_empty() {
            return None;
        }

        // Convert file path to route
        let route = self.file_path_to_route(path);

        // Only rewrite if the route exists in the site
        // This prevents breaking links to static assets (PDFs, images, etc.)
        if !self.routes.contains(&route) {
            return None;
        }

        // Reconstruct with query and anchor
        let mut result = route;
        if let Some(q) = query {
            result.push_str(q);
        }
        if let Some(a) = anchor {
            result.push_str(a);
        }

        Some(result)
    }

    /// Convert a file path to a route
    fn file_path_to_route(&self, path: &str) -> String {
        // Handle absolute paths (start with /)
        let (is_absolute, path) = if let Some(stripped) = path.strip_prefix('/') {
            (true, stripped)
        } else {
            (false, path)
        };

        // Get path without extension
        let path_obj = Path::new(path);
        let without_ext = path_obj
            .file_stem()
            .map(|s| {
                let parent = path_obj.parent().and_then(|p| p.to_str()).unwrap_or("");
                if parent.is_empty() {
                    s.to_string_lossy().to_string()
                } else {
                    format!("{}/{}", parent, s.to_string_lossy())
                }
            })
            .unwrap_or_else(|| path.to_string());

        // Check for index files
        let stem = path_obj.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let is_index = matches!(stem, "index" | "main" | "README");

        // Build route
        let route = if is_absolute {
            if is_index {
                let parent = Path::new(&without_ext)
                    .parent()
                    .and_then(|p| p.to_str())
                    .unwrap_or("");
                format!("/{}/", parent.trim_end_matches('/'))
            } else {
                format!("/{}/", without_ext)
            }
        } else {
            // Resolve relative to current route's directory
            // For index files (index.md, main.md, README.md), the route IS the directory
            // For non-index files (article.md -> /docs/article/), go up one level
            let route = self.current_route.trim_matches('/');
            let base = if self.is_index {
                route
            } else {
                // For non-index files, strip the last segment to get the parent directory
                // e.g., "docs/schema/article" -> "docs/schema"
                route
                    .rsplit_once('/')
                    .map(|(parent, _)| parent)
                    .unwrap_or("")
            };

            // Handle ../ navigation
            let resolved = resolve_relative_path(base, &without_ext);

            if is_index {
                let parent = Path::new(&resolved)
                    .parent()
                    .and_then(|p| p.to_str())
                    .unwrap_or("");
                format!("/{}/", parent.trim_matches('/'))
            } else {
                format!("/{}/", resolved.trim_matches('/'))
            }
        };

        // Normalize double slashes
        route.replace("//", "/")
    }
}

impl VisitorMut for LinkRewriter<'_> {
    fn visit_inline(&mut self, inline: &mut Inline) -> WalkControl {
        if let Inline::Link(link) = inline
            && let Some(new_target) = self.transform_target(&link.target)
        {
            link.target = new_target;
        }
        WalkControl::Continue
    }
}

/// Resolve a relative path against a base path, handling ../ segments
fn resolve_relative_path(base: &str, relative: &str) -> String {
    let mut segments: Vec<&str> = base.split('/').filter(|s| !s.is_empty()).collect();

    for part in relative.split('/') {
        match part {
            ".." => {
                segments.pop();
            }
            "." | "" => {}
            _ => segments.push(part),
        }
    }

    segments.join("/")
}

/// Apply link rewriting to a document node
///
/// The `routes_set` should be created once using `build_routes_set` and reused
/// across multiple documents for better performance.
///
/// The `is_index` parameter indicates whether the source file is an index file
/// (index.md, main.md, README.md). This affects how relative links are resolved:
/// - For index files: relative links resolve from the current route
/// - For non-index files: relative links resolve from the parent directory
pub fn rewrite_links<T: WalkNode>(
    node: &mut T,
    current_route: &str,
    routes_set: &HashSet<String>,
    is_index: bool,
) {
    let mut rewriter = LinkRewriter::new(current_route, routes_set, is_index);
    node.walk_mut(&mut rewriter);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_routes_set(routes: &[&str]) -> HashSet<String> {
        routes.iter().map(|r| r.to_string()).collect()
    }

    #[test]
    fn test_transform_target_skips_urls() {
        let routes = make_routes_set(&["/"]);
        let rewriter = LinkRewriter::new("/", &routes, true);

        // Common URL schemes
        assert_eq!(rewriter.transform_target("https://example.com"), None);
        assert_eq!(rewriter.transform_target("http://example.com"), None);
        assert_eq!(rewriter.transform_target("data:image/png;base64,abc"), None);
        assert_eq!(rewriter.transform_target("mailto:test@example.com"), None);
        assert_eq!(rewriter.transform_target("tel:+1234567890"), None);

        // Other URI schemes
        assert_eq!(rewriter.transform_target("ftp://files.example.com"), None);
        assert_eq!(rewriter.transform_target("ws://socket.example.com"), None);
        assert_eq!(rewriter.transform_target("wss://socket.example.com"), None);
        assert_eq!(rewriter.transform_target("file:///path/to/file"), None);
        assert_eq!(rewriter.transform_target("javascript:void(0)"), None);

        // Protocol-relative URLs
        assert_eq!(rewriter.transform_target("//cdn.example.com/lib.js"), None);

        // Internal anchors
        assert_eq!(rewriter.transform_target("#section"), None);
    }

    #[test]
    fn test_transform_skips_static_assets() {
        // Static assets that don't map to routes should be left unchanged
        let routes = make_routes_set(&["/", "/docs/"]);
        let rewriter = LinkRewriter::new("/", &routes, true);

        // These files don't have corresponding routes, so should return None
        assert_eq!(rewriter.transform_target("document.pdf"), None);
        assert_eq!(rewriter.transform_target("image.png"), None);
        assert_eq!(rewriter.transform_target("assets/logo.svg"), None);
        assert_eq!(rewriter.transform_target("downloads/report.xlsx"), None);
    }

    #[test]
    fn test_transform_simple_relative() {
        let routes = make_routes_set(&["/", "/other/"]);
        let rewriter = LinkRewriter::new("/", &routes, true);

        assert_eq!(
            rewriter.transform_target("other.md"),
            Some("/other/".to_string())
        );
    }

    #[test]
    fn test_transform_with_anchor() {
        let routes = make_routes_set(&["/", "/other/"]);
        let rewriter = LinkRewriter::new("/", &routes, true);

        assert_eq!(
            rewriter.transform_target("other.md#section"),
            Some("/other/#section".to_string())
        );
    }

    #[test]
    fn test_transform_with_query() {
        let routes = make_routes_set(&["/", "/other/"]);
        let rewriter = LinkRewriter::new("/", &routes, true);

        assert_eq!(
            rewriter.transform_target("other.md?v=1"),
            Some("/other/?v=1".to_string())
        );
    }

    #[test]
    fn test_transform_with_query_and_anchor() {
        let routes = make_routes_set(&["/", "/other/"]);
        let rewriter = LinkRewriter::new("/", &routes, true);

        assert_eq!(
            rewriter.transform_target("other.md?v=1#section"),
            Some("/other/?v=1#section".to_string())
        );
    }

    #[test]
    fn test_transform_nested_relative() {
        let routes = make_routes_set(&["/", "/docs/", "/docs/guide/"]);
        let rewriter = LinkRewriter::new("/docs/", &routes, true);

        assert_eq!(
            rewriter.transform_target("guide.md"),
            Some("/docs/guide/".to_string())
        );
    }

    #[test]
    fn test_transform_parent_relative() {
        // Include /docs/other/ as a valid route for the parent-relative link to resolve to
        let routes = make_routes_set(&["/", "/docs/", "/docs/guide/", "/docs/other/"]);
        let rewriter = LinkRewriter::new("/docs/guide/", &routes, true);

        assert_eq!(
            rewriter.transform_target("../other.md"),
            Some("/docs/other/".to_string())
        );
    }

    #[test]
    fn test_transform_parent_relative_nonexistent() {
        // If the parent-relative path doesn't map to a known route, leave unchanged
        let routes = make_routes_set(&["/", "/docs/", "/docs/guide/"]);
        let rewriter = LinkRewriter::new("/docs/guide/", &routes, true);

        // ../nonexistent.md would resolve to /docs/nonexistent/ which doesn't exist
        assert_eq!(rewriter.transform_target("../nonexistent.md"), None);
    }

    #[test]
    fn test_transform_absolute() {
        let routes = make_routes_set(&["/", "/docs/", "/docs/guide/"]);
        let rewriter = LinkRewriter::new("/other/", &routes, true);

        assert_eq!(
            rewriter.transform_target("/docs/guide.md"),
            Some("/docs/guide/".to_string())
        );
    }

    #[test]
    fn test_transform_index_file() {
        let routes = make_routes_set(&["/", "/docs/"]);
        let rewriter = LinkRewriter::new("/", &routes, true);

        assert_eq!(
            rewriter.transform_target("docs/index.md"),
            Some("/docs/".to_string())
        );
        assert_eq!(
            rewriter.transform_target("docs/README.md"),
            Some("/docs/".to_string())
        );
        assert_eq!(
            rewriter.transform_target("docs/main.md"),
            Some("/docs/".to_string())
        );
    }

    #[test]
    fn test_resolve_relative_path() {
        assert_eq!(resolve_relative_path("docs", "guide"), "docs/guide");
        assert_eq!(
            resolve_relative_path("docs/guide", "../other"),
            "docs/other"
        );
        assert_eq!(resolve_relative_path("docs/guide", "../../root"), "root");
        assert_eq!(resolve_relative_path("", "guide"), "guide");
        assert_eq!(resolve_relative_path("docs", "./guide"), "docs/guide");
    }

    #[test]
    fn test_transform_relative_from_non_index_file() {
        // For a non-index file like article.md with route /docs/schema/article/,
        // relative links should resolve from /docs/schema/ (parent), not /docs/schema/article/
        let routes = make_routes_set(&[
            "/",
            "/docs/",
            "/docs/schema/",
            "/docs/schema/article/",
            "/docs/schema/string/",
            "/docs/schema/creative-work/",
        ]);

        // From article.md (route /docs/schema/article/, is_index=false)
        let rewriter = LinkRewriter::new("/docs/schema/article/", &routes, false);

        // ./string.md should resolve to /docs/schema/string/ (sibling)
        assert_eq!(
            rewriter.transform_target("./string.md"),
            Some("/docs/schema/string/".to_string())
        );

        // ./creative-work.md should resolve to /docs/schema/creative-work/ (sibling)
        assert_eq!(
            rewriter.transform_target("./creative-work.md"),
            Some("/docs/schema/creative-work/".to_string())
        );
    }

    #[test]
    fn test_transform_relative_from_index_file() {
        // For an index file like index.md with route /docs/schema/,
        // relative links should resolve from /docs/schema/ (same as route)
        let routes = make_routes_set(&[
            "/",
            "/docs/",
            "/docs/schema/",
            "/docs/schema/article/",
            "/docs/schema/string/",
        ]);

        // From index.md (route /docs/schema/, is_index=true)
        let rewriter = LinkRewriter::new("/docs/schema/", &routes, true);

        // ./article.md should resolve to /docs/schema/article/ (child)
        assert_eq!(
            rewriter.transform_target("./article.md"),
            Some("/docs/schema/article/".to_string())
        );

        // ./string.md should resolve to /docs/schema/string/ (child)
        assert_eq!(
            rewriter.transform_target("./string.md"),
            Some("/docs/schema/string/".to_string())
        );
    }
}
