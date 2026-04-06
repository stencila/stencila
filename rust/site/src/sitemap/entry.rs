//! Sitemap entry types

use std::path::PathBuf;

use indexmap::IndexMap;
use serde::Serialize;
use stencila_config::AccessLevel;

use crate::RouteType;

/// A route entry prepared for sitemap serialization
///
/// Carries canonical URL information together with route metadata that may be
/// useful for richer sitemap formats and debugging.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SitemapEntry {
    /// The site route, e.g. `/docs/guide/`
    pub route: String,

    /// The canonical absolute URL for the route
    pub url: String,

    /// The display title for the route
    pub title: String,

    /// The source type of the route
    pub route_type: SitemapRouteType,

    /// The access level for the route
    pub access_level: AccessLevel,

    /// Whether the route is an auto-generated index page
    pub is_auto_index: bool,

    /// Whether the route is the generated specimen page
    pub is_specimen: bool,

    /// The source file path, when the route originated from a file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,

    /// Spread arguments for spread variants
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spread_arguments: Option<IndexMap<String, String>>,

    /// Last-modified timestamp in UTC RFC 3339 form
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lastmod: Option<String>,
}

impl SitemapEntry {
    /// Create a new sitemap entry
    pub fn new(
        route: String,
        url: String,
        title: String,
        route_type: RouteType,
        access_level: AccessLevel,
    ) -> Self {
        let is_auto_index = matches!(route_type, RouteType::AutoIndex);
        Self {
            route,
            url,
            title,
            route_type: route_type.into(),
            access_level,
            is_auto_index,
            is_specimen: false,
            source_path: None,
            spread_arguments: None,
            lastmod: None,
        }
    }

    /// Set the source file path for the entry
    pub fn with_source_path(mut self, source_path: Option<&PathBuf>) -> Self {
        self.source_path = source_path.map(|path| path.to_string_lossy().replace('\\', "/"));
        self
    }

    /// Set the spread arguments for the entry
    pub fn with_spread_arguments(
        mut self,
        spread_arguments: Option<&IndexMap<String, String>>,
    ) -> Self {
        self.spread_arguments = spread_arguments.cloned();
        self
    }

    /// Set the last-modified timestamp for the entry
    pub fn with_lastmod(mut self, lastmod: Option<String>) -> Self {
        self.lastmod = lastmod;
        self
    }

    /// Mark the entry as the generated specimen route
    pub fn with_specimen(mut self) -> Self {
        self.is_specimen = true;
        self
    }
}

/// Route source kinds for sitemap metadata
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum SitemapRouteType {
    /// Explicit file route from config
    File,
    /// Redirect route from config
    Redirect,
    /// Spread route variant
    Spread,
    /// Route implied from a file path
    Implied,
    /// Static asset route
    Static,
    /// Auto-generated index route
    AutoIndex,
    /// Generated specimen route
    Specimen,
}

impl From<RouteType> for SitemapRouteType {
    fn from(value: RouteType) -> Self {
        match value {
            RouteType::File => Self::File,
            RouteType::Redirect => Self::Redirect,
            RouteType::Spread => Self::Spread,
            RouteType::Implied => Self::Implied,
            RouteType::Static => Self::Static,
            RouteType::AutoIndex => Self::AutoIndex,
        }
    }
}
