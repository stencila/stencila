//! Main content area configuration
//!
//! Properties that control how the main content area is formatted,
//! independent of the structural layout (regions, sidebars, etc.).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Content width constraint
///
/// Controls the maximum width applied to content elements within the
/// main content area. The default (`65ch`) provides optimal line length
/// for reading. Set to `none` for full-width content where the page
/// author manages width using styled blocks or utility classes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ContentWidth {
    /// No max-width constraint; content fills available space
    None,
    /// Custom CSS width value (e.g., "80ch", "900px")
    #[serde(untagged)]
    Custom(String),
}

/// Content padding
///
/// Controls the padding around the `#main-content` element.
/// The default uses the theme's content spacing. Set to `none`
/// for full-bleed content.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ContentPadding {
    /// No padding (full-bleed content)
    None,
    /// Custom CSS padding value (e.g., "3rem", "24px")
    #[serde(untagged)]
    Custom(String),
}

/// Main content area configuration
///
/// Controls formatting of the main content area, including content width
/// constraints, padding, and title visibility. These properties are
/// orthogonal to the structural layout (regions, sidebars, etc.) and
/// can be set independently on any page regardless of preset.
///
/// Example:
/// ```toml
/// [site.layout.main]
/// width = "none"
/// padding = "none"
/// title = false
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct MainConfig {
    /// Maximum width for content elements
    ///
    /// Controls `max-width` on content children (paragraphs, headings, etc.).
    /// Defaults to `65ch` for optimal reading line length.
    /// Set to `"none"` for full-width content.
    pub width: Option<ContentWidth>,

    /// Padding around the main content area
    ///
    /// Controls padding on the `#main-content` element. Defaults to theme
    /// content spacing. Set to `"none"` for full-bleed content.
    pub padding: Option<ContentPadding>,

    /// Whether to display the document title slot
    ///
    /// When `false`, the `[slot="title"]` section is hidden via CSS.
    /// The HTML `<title>` element is unaffected (preserving SEO).
    /// Defaults to `true`.
    pub title: Option<bool>,
}

/// Merge two main configs with field-level granularity
///
/// For each field, if the override has a value it wins; otherwise the
/// base value is kept. This mirrors the same pattern used by `merge_region`
/// for layout regions.
pub fn merge_main(base: &Option<MainConfig>, override_: &Option<MainConfig>) -> Option<MainConfig> {
    match (base, override_) {
        (_, Some(o)) if o.width.is_some() || o.padding.is_some() || o.title.is_some() => {
            let b = base.as_ref().cloned().unwrap_or_default();
            Some(MainConfig {
                width: o.width.clone().or(b.width),
                padding: o.padding.clone().or(b.padding),
                title: o.title.or(b.title),
            })
        }
        // Override is None or all-None fields: keep base
        (Some(_), _) => base.clone(),
        _ => None,
    }
}
