//! Sitemap file generation

use std::{fs::Metadata, path::Path};

use eyre::Result;
use html_escape::encode_safe;
use stencila_config::{AccessLevel, SiteConfig, SitemapFormat, SitemapVisibility};
use tokio::fs::write;
use url::Url;

use super::entry::{SitemapEntry, SitemapRouteType};
use crate::{RouteEntry, RouteType, specimen::SPECIMEN_ROUTE};

/// Statistics from sitemap generation
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SitemapStats {
    /// Number of sitemap entries emitted
    pub entries: usize,

    /// Number of sitemap files written
    pub files: usize,
}

/// Generate sitemap files for a rendered site
///
/// Builds sitemap entries from the full route inventory so incremental renders
/// still regenerate complete sitemap outputs. Routes are filtered according to
/// the configured sitemap visibility and exclusion rules.
pub async fn generate_sitemaps(
    output_dir: &Path,
    base_url: &str,
    site_config: &SiteConfig,
    routes: &[RouteEntry],
) -> Result<SitemapStats> {
    let Some(spec) = site_config.sitemap.as_ref() else {
        return Ok(SitemapStats::default());
    };

    let config = spec.to_config();
    if !config.is_enabled() {
        return Ok(SitemapStats::default());
    }

    let base_url = Url::parse(base_url)?;
    let mut entries = Vec::new();

    for route in routes {
        if !is_sitemap_route(route) || config.is_route_excluded(&route.route) {
            continue;
        }

        let access_level = site_config
            .access
            .as_ref()
            .map(|access| access.get_access_level(&route.route))
            .unwrap_or(AccessLevel::Public);
        if matches!(config.visibility(), SitemapVisibility::PublicOnly)
            && access_level != AccessLevel::Public
        {
            continue;
        }

        entries.push(build_entry(&base_url, route, access_level, config.include_lastmod()).await?);
    }

    if matches!(config.visibility(), SitemapVisibility::All)
        && !config.is_route_excluded(SPECIMEN_ROUTE)
    {
        entries.push(SitemapEntry {
            route: SPECIMEN_ROUTE.to_string(),
            url: canonicalize_route(&base_url, SPECIMEN_ROUTE)?.to_string(),
            title: "Specimen".to_string(),
            route_type: SitemapRouteType::Specimen,
            access_level: AccessLevel::Public,
            is_auto_index: false,
            is_specimen: true,
            source_path: None,
            spread_arguments: None,
            lastmod: None,
        });
    }

    entries.sort_by(|a, b| a.route.cmp(&b.route));

    let mut stats = SitemapStats {
        entries: entries.len(),
        files: 0,
    };

    for format in config.formats() {
        match format {
            SitemapFormat::Xml => {
                write(&output_dir.join("sitemap.xml"), render_xml(&entries)).await?;
                stats.files += 1;
            }
            SitemapFormat::Txt => {
                write(&output_dir.join("sitemap.txt"), render_txt(&entries)).await?;
                stats.files += 1;
            }
        }
    }

    Ok(stats)
}

/// Check whether a route type should be considered for sitemap generation
fn is_sitemap_route(route: &RouteEntry) -> bool {
    matches!(
        route.route_type,
        RouteType::File | RouteType::Spread | RouteType::Implied | RouteType::AutoIndex
    )
}

/// Build a sitemap entry from a route entry
async fn build_entry(
    base_url: &Url,
    route: &RouteEntry,
    access_level: AccessLevel,
    include_lastmod: bool,
) -> Result<SitemapEntry> {
    let lastmod = if include_lastmod {
        match route.route_type {
            RouteType::File | RouteType::Spread | RouteType::Implied => {
                source_lastmod(route.source_path.as_deref()).await?
            }
            RouteType::AutoIndex | RouteType::Redirect | RouteType::Static => None,
        }
    } else {
        None
    };

    Ok(SitemapEntry::new(
        route.route.clone(),
        canonicalize_route(base_url, &route.route)?.to_string(),
        route.title(),
        route.route_type,
        access_level,
    )
    .with_source_path(route.source_path.as_ref())
    .with_spread_arguments(route.spread_arguments.as_ref())
    .with_lastmod(lastmod))
}

/// Convert a route to a canonical absolute URL
fn canonicalize_route(base_url: &Url, route: &str) -> Result<Url> {
    Ok(base_url.join(route.trim_start_matches('/'))?)
}

/// Read the modification time for a source path
async fn source_lastmod(source_path: Option<&Path>) -> Result<Option<String>> {
    let Some(path) = source_path else {
        return Ok(None);
    };

    let metadata = tokio::fs::metadata(path).await.ok();
    Ok(metadata.and_then(last_modified_string))
}

/// Convert a file metadata timestamp to an RFC 3339 UTC string
fn last_modified_string(metadata: Metadata) -> Option<String> {
    metadata.modified().ok().map(|time| {
        chrono::DateTime::<chrono::Utc>::from(time)
            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
    })
}

/// Render XML sitemap content
fn render_xml(entries: &[SitemapEntry]) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
"#,
    );

    for entry in entries {
        xml.push_str("  <url>\n");
        xml.push_str("    <loc>");
        xml.push_str(&encode_safe(&entry.url));
        xml.push_str("</loc>\n");
        if let Some(lastmod) = &entry.lastmod {
            xml.push_str("    <lastmod>");
            xml.push_str(&encode_safe(lastmod));
            xml.push_str("</lastmod>\n");
        }
        xml.push_str("  </url>\n");
    }

    xml.push_str("</urlset>\n");
    xml
}

/// Render text sitemap content
///
/// Each route is emitted as a single canonical URL line.
fn render_txt(entries: &[SitemapEntry]) -> String {
    if entries.is_empty() {
        return String::new();
    }

    let lines = entries
        .iter()
        .map(|entry| entry.url.clone())
        .collect::<Vec<_>>();
    format!("{}\n", lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use stencila_config::{SiteAccessConfig, SitemapConfig, SitemapSpec};
    use tempfile::TempDir;

    use super::*;

    fn site_config_with_sitemap() -> SiteConfig {
        SiteConfig {
            sitemap: Some(SitemapSpec::Enabled(true)),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_generate_sitemaps_public_only_filters_private_and_specimen() -> Result<()> {
        let output_dir = TempDir::new()?;
        let source_dir = TempDir::new()?;
        let public_path = source_dir.path().join("public.md");
        let private_path = source_dir.path().join("private.md");
        tokio::fs::write(&public_path, "public").await?;
        tokio::fs::write(&private_path, "private").await?;

        let mut site_config = site_config_with_sitemap();
        site_config.access = Some(SiteAccessConfig {
            default: AccessLevel::Public,
            routes: std::iter::once(("/private/".to_string(), AccessLevel::Password)).collect(),
        });

        let routes = vec![
            RouteEntry {
                route: "/public/".to_string(),
                route_type: RouteType::Implied,
                target: "public.md".to_string(),
                source_path: Some(public_path),
                spread_count: None,
                spread_arguments: None,
            },
            RouteEntry {
                route: "/private/".to_string(),
                route_type: RouteType::Implied,
                target: "private.md".to_string(),
                source_path: Some(private_path),
                spread_count: None,
                spread_arguments: None,
            },
        ];

        let stats = generate_sitemaps(
            output_dir.path(),
            "https://example.com/",
            &site_config,
            &routes,
        )
        .await?;

        assert_eq!(stats.entries, 1);
        assert!(output_dir.path().join("sitemap.xml").exists());
        assert!(!output_dir.path().join("sitemap.txt").exists());
        Ok(())
    }

    #[tokio::test]
    async fn test_generate_sitemaps_all_includes_specimen_in_txt() -> Result<()> {
        let output_dir = TempDir::new()?;
        let source_dir = TempDir::new()?;
        let spread_path = source_dir.path().join("report.smd");
        tokio::fs::write(&spread_path, "report").await?;

        let mut args = IndexMap::new();
        args.insert("region".to_string(), "north".to_string());

        let site_config = SiteConfig {
            sitemap: Some(SitemapSpec::Config(SitemapConfig {
                enabled: Some(true),
                visibility: Some(SitemapVisibility::All),
                formats: Some(vec![SitemapFormat::Txt]),
                ..Default::default()
            })),
            ..Default::default()
        };

        let routes = vec![RouteEntry {
            route: "/north/".to_string(),
            route_type: RouteType::Spread,
            target: "report.smd".to_string(),
            source_path: Some(spread_path.clone()),
            spread_count: None,
            spread_arguments: Some(args),
        }];

        let stats = generate_sitemaps(
            output_dir.path(),
            "https://example.com/",
            &site_config,
            &routes,
        )
        .await?;

        assert_eq!(stats.entries, 2);
        let txt = tokio::fs::read_to_string(output_dir.path().join("sitemap.txt")).await?;
        assert!(txt.contains("https://example.com/north/"));
        assert!(txt.contains("https://example.com/_specimen/"));
        Ok(())
    }

    #[tokio::test]
    async fn test_generate_sitemaps_txt_emits_only_urls() -> Result<()> {
        let output_dir = TempDir::new()?;
        let site_config = SiteConfig {
            sitemap: Some(SitemapSpec::Config(SitemapConfig {
                enabled: Some(true),
                formats: Some(vec![SitemapFormat::Txt]),
                ..Default::default()
            })),
            ..Default::default()
        };

        let routes = vec![RouteEntry {
            route: "/docs/".to_string(),
            route_type: RouteType::AutoIndex,
            target: "[auto-index]".to_string(),
            source_path: None,
            spread_count: None,
            spread_arguments: None,
        }];

        generate_sitemaps(
            output_dir.path(),
            "https://example.com/",
            &site_config,
            &routes,
        )
        .await?;

        let txt = tokio::fs::read_to_string(output_dir.path().join("sitemap.txt")).await?;
        assert_eq!(txt, "https://example.com/docs/\n");
        Ok(())
    }

    #[test]
    fn test_canonicalize_route_root() -> Result<()> {
        let base_url = Url::parse("https://example.com/")?;
        let root = canonicalize_route(&base_url, "/")?;
        assert_eq!(root.as_str(), "https://example.com/");
        Ok(())
    }

    #[test]
    fn test_render_txt_empty_entries() {
        assert_eq!(render_txt(&[]), "");
    }
}
