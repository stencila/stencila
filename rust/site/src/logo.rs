//! Logo resolution and rendering
//!
//! This module handles:
//! - Auto-inference of logo files from site directory
//! - Resolution of logo config (merging site-level and component-level)
//! - Rendering logo to HTML with CSS custom properties

use std::path::Path;

use stencila_config::{LogoConfig, LogoSpec};

/// Image extensions to search for, in priority order
const IMAGE_EXTENSIONS: &[&str] = &["svg", "png", "webp", "jpg", "jpeg"];

/// Logo variants to search for
const LOGO_VARIANTS: &[(&str, &str)] = &[
    ("default", "logo"),
    ("mobile", "logo-mobile"),
    ("tablet", "logo-tablet"),
    ("dark", "logo-dark"),
    ("dark_mobile", "logo-dark-mobile"),
    ("dark_tablet", "logo-dark-tablet"),
];

/// Infer logo configuration from files in the site directory
///
/// Searches for files matching the pattern `logo-{variant}.{ext}` where
/// variant is one of: (none), mobile, tablet, dark, dark-mobile, dark-tablet
/// and ext is one of: svg, png, webp, jpg, jpeg (in priority order).
///
/// Searches the site root and up to 2 levels of subdirectories
/// (e.g., `assets/`, `images/`, `static/images/`).
pub fn infer_logo_from_directory(site_root: &Path) -> Option<LogoConfig> {
    let mut config = LogoConfig::default();
    let mut found_any = false;

    for (field, file_prefix) in LOGO_VARIANTS {
        if let Some(path) = find_logo_file(site_root, site_root, file_prefix, 0) {
            found_any = true;
            let path_str = Some(path);
            match *field {
                "default" => config.default = path_str,
                "mobile" => config.mobile = path_str,
                "tablet" => config.tablet = path_str,
                "dark" => config.dark = path_str,
                "dark_mobile" => config.dark_mobile = path_str,
                "dark_tablet" => config.dark_tablet = path_str,
                _ => {}
            }
        }
    }

    if found_any { Some(config) } else { None }
}

/// Maximum depth to search for logo files in subdirectories
const MAX_SEARCH_DEPTH: u8 = 2;

/// Find a logo file with the given prefix, searching recursively up to MAX_SEARCH_DEPTH
fn find_logo_file(site_root: &Path, dir: &Path, prefix: &str, depth: u8) -> Option<String> {
    // First, check for logo files in the current directory
    for ext in IMAGE_EXTENSIONS {
        let filename = format!("{prefix}.{ext}");
        let path = dir.join(&filename);
        if path.exists() {
            // Return path relative to site root (with leading slash for URL)
            let relative = path
                .strip_prefix(site_root)
                .ok()?
                .to_string_lossy()
                .to_string();
            return Some(format!("/{relative}"));
        }
    }

    // If not found and we haven't reached max depth, search subdirectories
    if depth < MAX_SEARCH_DEPTH
        && let Ok(entries) = std::fs::read_dir(dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip hidden directories
                if let Some(name) = path.file_name().and_then(|n| n.to_str())
                    && name.starts_with('.')
                {
                    continue;
                }
                if let Some(found) = find_logo_file(site_root, &path, prefix, depth + 1) {
                    return Some(found);
                }
            }
        }
    }

    None
}

/// Resolve logo configuration by merging component config with site config and inference
///
/// Priority (highest to lowest):
/// 1. Component-level config fields
/// 2. Site-level config (`site.logo`)
/// 3. Auto-inferred from site directory
///
/// Fields are merged individually - component fields override site-level,
/// but missing component fields inherit from site-level.
pub fn resolve_logo(
    component_config: Option<&LogoConfig>,
    site_logo: Option<&LogoSpec>,
    site_root: Option<&Path>,
) -> Option<LogoConfig> {
    // Start with auto-inferred config (lowest priority)
    let mut resolved = site_root
        .and_then(infer_logo_from_directory)
        .unwrap_or_default();

    // Merge site-level config
    if let Some(site_logo) = site_logo {
        let site_config = match site_logo {
            LogoSpec::Path(path) => LogoConfig {
                default: Some(path.clone()),
                ..Default::default()
            },
            LogoSpec::Config(config) => config.clone(),
        };
        merge_logo_config(&mut resolved, &site_config);
    }

    // Merge component-level config (highest priority)
    if let Some(component_config) = component_config {
        merge_logo_config(&mut resolved, component_config);
    }

    // Return None if no logo was found at any level
    if resolved.default.is_none()
        && resolved.mobile.is_none()
        && resolved.tablet.is_none()
        && resolved.dark.is_none()
        && resolved.dark_mobile.is_none()
        && resolved.dark_tablet.is_none()
    {
        return None;
    }

    Some(resolved)
}

/// Check if a LogoConfig has at least one image field set.
///
/// A logo must have at least one image to be rendered; link/alt alone are not sufficient.
/// This matches the guard in `resolve_logo` that returns None when no images exist.
pub fn has_any_image(config: &LogoConfig) -> bool {
    config.default.is_some()
        || config.mobile.is_some()
        || config.tablet.is_some()
        || config.dark.is_some()
        || config.dark_mobile.is_some()
        || config.dark_tablet.is_some()
}

/// Merge source config into target, overriding only fields that are Some in source
pub fn merge_logo_config(target: &mut LogoConfig, source: &LogoConfig) {
    if source.default.is_some() {
        target.default.clone_from(&source.default);
    }
    if source.mobile.is_some() {
        target.mobile.clone_from(&source.mobile);
    }
    if source.tablet.is_some() {
        target.tablet.clone_from(&source.tablet);
    }
    if source.dark.is_some() {
        target.dark.clone_from(&source.dark);
    }
    if source.dark_mobile.is_some() {
        target.dark_mobile.clone_from(&source.dark_mobile);
    }
    if source.dark_tablet.is_some() {
        target.dark_tablet.clone_from(&source.dark_tablet);
    }
    if source.link.is_some() {
        target.link.clone_from(&source.link);
    }
    if source.alt.is_some() {
        target.alt.clone_from(&source.alt);
    }
}

/// Render a resolved logo configuration to HTML
///
/// Generates a `<stencila-logo>` custom element wrapping an `<a>` that
/// contains a `<picture>` with `<source>` elements for responsive and
/// dark-mode variants.
///
/// `<picture>` handles system dark mode preference and viewport breakpoints
/// natively with zero JS. A companion JS component only intervenes when the
/// user explicitly overrides the system preference via `data-color-scheme`,
/// which is the one case `<picture>` media queries cannot express.
///
/// The variant paths are also placed as attributes on `<stencila-logo>` so
/// the JS component can read them for the override case.
///
/// Example output:
/// ```html
/// <stencila-logo default="/logo.svg" dark="/logo-dark.svg">
///   <a href="/">
///     <picture>
///       <source media="(prefers-color-scheme: dark)" srcset="/logo-dark.svg">
///       <img src="/logo.svg" alt="Company Logo">
///     </picture>
///   </a>
/// </stencila-logo>
/// ```
pub fn render_logo(config: &LogoConfig) -> String {
    let mut attrs = Vec::new();

    if let Some(ref path) = config.default {
        attrs.push(format!("default=\"{}\"", html_escape(&make_absolute(path))));
    }
    if let Some(ref path) = config.mobile {
        attrs.push(format!("mobile=\"{}\"", html_escape(&make_absolute(path))));
    }
    if let Some(ref path) = config.tablet {
        attrs.push(format!("tablet=\"{}\"", html_escape(&make_absolute(path))));
    }
    if let Some(ref path) = config.dark {
        attrs.push(format!("dark=\"{}\"", html_escape(&make_absolute(path))));
    }
    if let Some(ref path) = config.dark_mobile {
        attrs.push(format!(
            "dark-mobile=\"{}\"",
            html_escape(&make_absolute(path))
        ));
    }
    if let Some(ref path) = config.dark_tablet {
        attrs.push(format!(
            "dark-tablet=\"{}\"",
            html_escape(&make_absolute(path))
        ));
    }

    let logo_attrs = if attrs.is_empty() {
        String::new()
    } else {
        format!(" {}", attrs.join(" "))
    };

    // Build <source> elements for <picture> (most specific first)
    let mut sources = Vec::new();

    // Dark mode + mobile
    if let Some(ref path) = config.dark_mobile {
        sources.push(format!(
            "<source media=\"(prefers-color-scheme: dark) and (max-width: 639px)\" srcset=\"{}\">",
            html_escape(&make_absolute(path))
        ));
    }
    // Dark mode + tablet
    if let Some(ref path) = config.dark_tablet {
        sources.push(format!(
            "<source media=\"(prefers-color-scheme: dark) and (min-width: 640px) and (max-width: 767px)\" srcset=\"{}\">",
            html_escape(&make_absolute(path))
        ));
    }
    // Dark mode (desktop)
    if let Some(ref path) = config.dark {
        sources.push(format!(
            "<source media=\"(prefers-color-scheme: dark)\" srcset=\"{}\">",
            html_escape(&make_absolute(path))
        ));
    }
    // Light mode mobile
    if let Some(ref path) = config.mobile {
        sources.push(format!(
            "<source media=\"(max-width: 639px)\" srcset=\"{}\">",
            html_escape(&make_absolute(path))
        ));
    }
    // Light mode tablet
    if let Some(ref path) = config.tablet {
        sources.push(format!(
            "<source media=\"(min-width: 640px) and (max-width: 767px)\" srcset=\"{}\">",
            html_escape(&make_absolute(path))
        ));
    }

    let sources_html = sources.join("");

    let link = html_escape(config.link.as_deref().unwrap_or("/"));
    let alt = html_escape(config.alt.as_deref().unwrap_or("Home"));

    // The <img> src is the default logo, falling back through variants so we
    // never emit an empty src (which can trigger spurious requests in browsers).
    let fallback_path = config
        .default
        .as_ref()
        .or(config.mobile.as_ref())
        .or(config.tablet.as_ref())
        .or(config.dark.as_ref())
        .or(config.dark_mobile.as_ref())
        .or(config.dark_tablet.as_ref());
    let img_src = fallback_path
        .map(|p| html_escape(&make_absolute(p)))
        .unwrap_or_default();

    format!(
        "<stencila-logo{logo_attrs}>\
         <a href=\"{link}\">\
         <picture>{sources_html}<img src=\"{img_src}\" alt=\"{alt}\"></picture>\
         </a>\
         </stencila-logo>"
    )
}

/// Make a path absolute (prefix with / if not already absolute or a URL)
fn make_absolute(path: &str) -> String {
    if path.starts_with('/') || path.starts_with("http://") || path.starts_with("https://") {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

/// Simple HTML attribute escaping
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_logo_simple_path() {
        let site_logo = LogoSpec::Path("logo.svg".to_string());
        let resolved = resolve_logo(None, Some(&site_logo), None);

        let config = resolved.expect("should resolve logo");
        assert_eq!(config.default, Some("logo.svg".to_string()));
    }

    #[test]
    fn test_resolve_logo_responsive() {
        let site_logo = LogoSpec::Config(LogoConfig {
            default: Some("logo.svg".to_string()),
            dark: Some("logo-dark.svg".to_string()),
            mobile: Some("logo-mobile.svg".to_string()),
            ..Default::default()
        });
        let resolved = resolve_logo(None, Some(&site_logo), None);

        let config = resolved.expect("should resolve logo");
        assert_eq!(config.default, Some("logo.svg".to_string()));
        assert_eq!(config.dark, Some("logo-dark.svg".to_string()));
        assert_eq!(config.mobile, Some("logo-mobile.svg".to_string()));
    }

    #[test]
    fn test_resolve_logo_component_override() {
        let site_logo = LogoSpec::Config(LogoConfig {
            default: Some("site-logo.svg".to_string()),
            link: Some("/home/".to_string()),
            ..Default::default()
        });
        let component = LogoConfig {
            default: Some("header-logo.svg".to_string()),
            ..Default::default()
        };
        let resolved = resolve_logo(Some(&component), Some(&site_logo), None);

        let config = resolved.expect("should resolve logo");
        // Component overrides default
        assert_eq!(config.default, Some("header-logo.svg".to_string()));
        // Site-level link is inherited
        assert_eq!(config.link, Some("/home/".to_string()));
    }

    #[test]
    fn test_resolve_logo_none() {
        let resolved = resolve_logo(None, None, None);
        assert!(resolved.is_none());
    }

    #[test]
    fn test_render_logo_simple() {
        let config = LogoConfig {
            default: Some("/logo.svg".to_string()),
            link: Some("/".to_string()),
            alt: Some("My Logo".to_string()),
            ..Default::default()
        };
        let html = render_logo(&config);

        assert!(html.contains("stencila-logo"));
        assert!(html.contains("default=\"/logo.svg\""));
        assert!(html.contains("href=\"/\""));
        // Uses <picture> with <img> for native rendering
        assert!(html.contains("<img src=\"/logo.svg\" alt=\"My Logo\">"));
        // No <source> elements when only default is set
        assert!(!html.contains("<source"));
    }

    #[test]
    fn test_render_logo_responsive() {
        let config = LogoConfig {
            default: Some("/logo.svg".to_string()),
            dark: Some("/logo-dark.svg".to_string()),
            mobile: Some("/logo-mobile.svg".to_string()),
            ..Default::default()
        };
        let html = render_logo(&config);

        // Attributes on custom element for JS
        assert!(html.contains("default=\"/logo.svg\""));
        assert!(html.contains("dark=\"/logo-dark.svg\""));
        assert!(html.contains("mobile=\"/logo-mobile.svg\""));

        // <source> elements for native <picture> handling
        assert!(
            html.contains(
                "<source media=\"(prefers-color-scheme: dark)\" srcset=\"/logo-dark.svg\">"
            )
        );
        assert!(html.contains("<source media=\"(max-width: 639px)\" srcset=\"/logo-mobile.svg\">"));

        // Fallback <img>
        assert!(html.contains("<img src=\"/logo.svg\" alt=\"Home\">"));
    }

    #[test]
    fn test_make_absolute() {
        assert_eq!(make_absolute("/logo.svg"), "/logo.svg");
        assert_eq!(make_absolute("logo.svg"), "/logo.svg");
        assert_eq!(
            make_absolute("https://example.com/logo.svg"),
            "https://example.com/logo.svg"
        );
    }

    #[test]
    fn test_has_any_image() {
        // Empty config has no images
        assert!(!has_any_image(&LogoConfig::default()));

        // Config with only link/alt has no images (should not render)
        let link_only = LogoConfig {
            link: Some("/".to_string()),
            alt: Some("Logo".to_string()),
            ..Default::default()
        };
        assert!(!has_any_image(&link_only));

        // Config with default image
        let with_default = LogoConfig {
            default: Some("/logo.svg".to_string()),
            ..Default::default()
        };
        assert!(has_any_image(&with_default));

        // Config with only dark variant
        let with_dark = LogoConfig {
            dark: Some("/logo-dark.svg".to_string()),
            link: Some("/".to_string()),
            ..Default::default()
        };
        assert!(has_any_image(&with_dark));
    }
}
