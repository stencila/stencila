//! CSS and layout measurement

use std::collections::HashMap;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::devices::ViewportConfig;

/// Context for deriving post-measurement diagnostics.
#[derive(Debug, Clone, Copy, Default)]
pub struct MeasurementContext {
    pub viewport_only_capture: bool,
}

/// Measurement preset determining which selectors to measure
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum MeasurePreset {
    /// Document content selectors (stencila-article, headings, etc.)
    Document,
    /// Site chrome selectors (layout, header, nav, logo, sidebar, footer)
    Site,
    /// Both document and site selectors
    All,
    /// Header-focused selectors
    Header,
    /// Navigation-focused selectors
    Nav,
    /// Main content selectors
    Main,
    /// Footer-focused selectors
    Footer,
    /// Theme review selectors spanning key regions
    Theme,
}

/// Return default selectors for document content measurement
pub fn document_selectors() -> Vec<String> {
    [
        "stencila-article",
        "stencila-paragraph",
        "stencila-heading[level=\"1\"]",
        "stencila-heading[level=\"2\"]",
        "stencila-heading[level=\"3\"]",
        "stencila-code-block",
        "stencila-list",
        "stencila-figure",
        "stencila-table",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect()
}

/// Return default selectors for site chrome measurement
pub fn site_selectors() -> Vec<String> {
    [
        "stencila-layout",
        "stencila-layout > header",
        "stencila-layout > .layout-body > .left-sidebar",
        "stencila-layout > .layout-body > .right-sidebar",
        "stencila-layout > footer",
        "stencila-nav-tree",
        "stencila-nav-menu",
        "stencila-breadcrumbs",
        "stencila-logo",
        "main#main-content",
        ".layout-main",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect()
}

/// Return header selectors
pub fn header_selectors() -> Vec<String> {
    ["stencila-layout > header", "header", ".site-header"]
        .iter()
        .map(|s| (*s).to_string())
        .collect()
}

/// Return navigation selectors
pub fn nav_selectors() -> Vec<String> {
    [
        "stencila-nav-tree",
        "stencila-nav-menu",
        "stencila-breadcrumbs",
        "nav",
        "[role=\"navigation\"]",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect()
}

/// Return main content selectors
pub fn main_selectors() -> Vec<String> {
    [
        "main#main-content",
        ".layout-main",
        "main",
        "stencila-article",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect()
}

/// Return footer selectors
pub fn footer_selectors() -> Vec<String> {
    ["stencila-layout > footer", "footer", ".site-footer"]
        .iter()
        .map(|s| (*s).to_string())
        .collect()
}

/// Return theme review selectors
pub fn theme_selectors() -> Vec<String> {
    let mut selectors = Vec::new();
    selectors.extend(header_selectors());
    selectors.extend(nav_selectors());
    selectors.extend(main_selectors());
    selectors.extend(footer_selectors());
    selectors.extend([
        "stencila-heading[level=\"1\"]".to_string(),
        "stencila-paragraph".to_string(),
        "stencila-code-block".to_string(),
        "stencila-table".to_string(),
    ]);
    selectors
}

/// Build the selector list for a given preset
pub fn selectors_for_preset(preset: MeasurePreset) -> Vec<String> {
    match preset {
        MeasurePreset::Document => document_selectors(),
        MeasurePreset::Site => site_selectors(),
        MeasurePreset::All => {
            let mut all = document_selectors();
            all.extend(site_selectors());
            all
        }
        MeasurePreset::Header => header_selectors(),
        MeasurePreset::Nav => nav_selectors(),
        MeasurePreset::Main => main_selectors(),
        MeasurePreset::Footer => footer_selectors(),
        MeasurePreset::Theme => theme_selectors(),
    }
}

/// Result of measurements collected from the browser
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasureResult {
    /// Computed CSS properties by selector
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub css: HashMap<String, CssProperties>,

    /// Bounding box information by selector
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub box_info: HashMap<String, BoxInfo>,

    /// Element counts by selector
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub counts: HashMap<String, usize>,

    /// Text content by selector
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub text: HashMap<String, String>,

    /// Concise theme-oriented summaries by selector
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub summaries: HashMap<String, StyleSummary>,

    /// Contrast checks by selector
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub contrast: HashMap<String, ContrastCheck>,

    /// Diagnostics and warnings
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub diagnostics: Vec<String>,

    /// Diagnostic errors and warnings
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub errors: Vec<String>,
}

/// Concise summary of selector styles and likely issues.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleSummary {
    pub summary: String,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub issues: Vec<String>,
}

/// Contrast evaluation for a selector.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContrastCheck {
    pub ratio: Option<f64>,
    pub normal_text_aa: Option<bool>,
    pub large_text_aa: Option<bool>,
    pub foreground: Option<String>,
    pub background: Option<String>,
    pub reason: Option<String>,
}

/// Common CSS properties
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CssProperties {
    // Spacing
    pub padding_top: Option<String>,

    pub padding_bottom: Option<String>,

    pub padding_left: Option<String>,

    pub padding_right: Option<String>,

    pub margin_top: Option<String>,

    pub margin_bottom: Option<String>,

    pub margin_left: Option<String>,

    pub margin_right: Option<String>,

    // Typography
    pub font_size: Option<String>,

    pub line_height: Option<String>,

    pub color: Option<String>,

    pub color_hex: Option<String>,

    pub font_family: Option<String>,

    pub font_weight: Option<String>,

    pub text_align: Option<String>,

    pub text_decoration: Option<String>,

    pub letter_spacing: Option<String>,

    pub text_transform: Option<String>,

    pub white_space: Option<String>,

    // Display
    pub display: Option<String>,

    pub visibility: Option<String>,

    pub opacity: Option<String>,

    // Backgrounds
    pub background_color: Option<String>,

    pub background_color_hex: Option<String>,

    pub background_image: Option<String>,

    pub background_size: Option<String>,

    pub background_position: Option<String>,

    // Borders
    pub border_width: Option<String>,

    pub border_color: Option<String>,

    pub border_color_hex: Option<String>,

    pub border_radius: Option<String>,

    pub border_style: Option<String>,

    pub border_top_width: Option<String>,

    pub border_right_width: Option<String>,

    pub border_bottom_width: Option<String>,

    pub border_left_width: Option<String>,

    // Layout
    pub position: Option<String>,

    pub top: Option<String>,

    pub right: Option<String>,

    pub bottom: Option<String>,

    pub left: Option<String>,

    pub z_index: Option<String>,

    pub overflow: Option<String>,

    pub overflow_x: Option<String>,

    pub overflow_y: Option<String>,

    pub min_height: Option<String>,

    pub max_width: Option<String>,

    // Flexbox
    pub gap: Option<String>,

    pub justify_content: Option<String>,

    pub align_items: Option<String>,

    pub flex_direction: Option<String>,

    // Visual effects
    pub box_shadow: Option<String>,

    pub transform: Option<String>,

    pub filter: Option<String>,
}

/// Bounding box information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxInfo {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// JavaScript code to inject for measurement
///
/// This function accepts a JSON array of selectors and collects computed styles,
/// bounding boxes, counts, and text content for each, returning them as JSON
pub const MEASUREMENT_SCRIPT: &str = r#"
(function(selectors) {
    // Helper function to convert rgb()/rgba() to hex
    function rgbToHex(rgb) {
        if (!rgb || rgb === 'transparent') return null;

        // Match rgb() or rgba()
        const match = rgb.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*([\d.]+))?\)/);
        if (!match) return null;

        const r = parseInt(match[1]);
        const g = parseInt(match[2]);
        const b = parseInt(match[3]);
        const a = match[4] !== undefined ? parseFloat(match[4]) : 1;

        // If fully transparent, return null
        if (a === 0) return null;

        const hex = '#' + [r, g, b].map(x => x.toString(16).padStart(2, '0')).join('');

        // Include alpha if not fully opaque
        if (a < 1) {
            const alphaHex = Math.round(a * 255).toString(16).padStart(2, '0');
            return hex + alphaHex;
        }

        return hex;
    }

    const result = {
        css: {},
        box_info: {},
        counts: {},
        text: {},
        errors: []
    };

    for (const sel of selectors) {
        // Count elements
        const elements = document.querySelectorAll(sel);
        result.counts[sel] = elements.length;

        if (elements.length === 0) {
            continue;
        }

        // Get first element for measurements
        const el = elements[0];

        // Computed styles
        const cs = getComputedStyle(el);
        result.css[sel] = {
            // Spacing
            paddingTop: cs.paddingTop,
            paddingBottom: cs.paddingBottom,
            paddingLeft: cs.paddingLeft,
            paddingRight: cs.paddingRight,
            marginTop: cs.marginTop,
            marginBottom: cs.marginBottom,
            marginLeft: cs.marginLeft,
            marginRight: cs.marginRight,
            // Typography
            fontSize: cs.fontSize,
            lineHeight: cs.lineHeight,
            color: cs.color,
            colorHex: rgbToHex(cs.color),
            fontFamily: cs.fontFamily,
            fontWeight: cs.fontWeight,
            textAlign: cs.textAlign,
            textDecoration: cs.textDecoration,
            letterSpacing: cs.letterSpacing,
            textTransform: cs.textTransform,
            whiteSpace: cs.whiteSpace,
            // Display
            display: cs.display,
            visibility: cs.visibility,
            opacity: cs.opacity,
            // Backgrounds
            backgroundColor: cs.backgroundColor,
            backgroundColorHex: rgbToHex(cs.backgroundColor),
            backgroundImage: cs.backgroundImage,
            backgroundSize: cs.backgroundSize,
            backgroundPosition: cs.backgroundPosition,
            // Borders
            borderWidth: cs.borderWidth,
            borderColor: cs.borderColor,
            borderColorHex: rgbToHex(cs.borderColor),
            borderRadius: cs.borderRadius,
            borderStyle: cs.borderStyle,
            borderTopWidth: cs.borderTopWidth,
            borderRightWidth: cs.borderRightWidth,
            borderBottomWidth: cs.borderBottomWidth,
            borderLeftWidth: cs.borderLeftWidth,
            // Layout
            position: cs.position,
            top: cs.top,
            right: cs.right,
            bottom: cs.bottom,
            left: cs.left,
            zIndex: cs.zIndex,
            overflow: cs.overflow,
            overflowX: cs.overflowX,
            overflowY: cs.overflowY,
            minHeight: cs.minHeight,
            maxWidth: cs.maxWidth,
            // Flexbox
            gap: cs.gap,
            justifyContent: cs.justifyContent,
            alignItems: cs.alignItems,
            flexDirection: cs.flexDirection,
            // Visual effects
            boxShadow: cs.boxShadow,
            transform: cs.transform,
            filter: cs.filter
        };

        // Bounding box
        const rect = el.getBoundingClientRect();
        result.box_info[sel] = {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height
        };

        // Text content (truncate if very long)
        const text = el.textContent || '';
        result.text[sel] = text.length > 200 ? text.substring(0, 200) + '...' : text;
    }

    return result;
})
"#;

/// JavaScript code to extract resolved CSS custom property (token) values
///
/// Reads all `--*` custom properties from stylesheets and returns their
/// computed values from `:root`
pub const TOKENS_SCRIPT: &str = r#"
(function() {
    const root = document.documentElement;
    const styles = getComputedStyle(root);
    const tokens = {};
    for (const sheet of document.styleSheets) {
        try {
            for (const rule of sheet.cssRules) {
                if (rule.style) {
                    for (let i = 0; i < rule.style.length; i++) {
                        const prop = rule.style[i];
                        if (prop.startsWith('--')) {
                            tokens[prop] = styles.getPropertyValue(prop).trim();
                        }
                    }
                }
            }
        } catch (e) { /* cross-origin sheet, skip */ }
    }
    return tokens;
})()
"#;

/// JavaScript code to extract the page's color palette
///
/// Samples computed color, backgroundColor, and borderColor from all visible
/// elements, deduplicates, and returns sorted by usage count
pub const PALETTE_SCRIPT: &str = r#"
(function() {
    function rgbToHex(rgb) {
        if (!rgb || rgb === 'transparent') return null;
        const match = rgb.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*([\d.]+))?\)/);
        if (!match) return null;
        const r = parseInt(match[1]);
        const g = parseInt(match[2]);
        const b = parseInt(match[3]);
        const a = match[4] !== undefined ? parseFloat(match[4]) : 1;
        if (a === 0) return null;
        const hex = '#' + [r, g, b].map(x => x.toString(16).padStart(2, '0')).join('');
        if (a < 1) {
            const alphaHex = Math.round(a * 255).toString(16).padStart(2, '0');
            return hex + alphaHex;
        }
        return hex;
    }

    const colors = {};
    const elements = document.querySelectorAll('*');
    for (const el of elements) {
        if (el.offsetWidth === 0 && el.offsetHeight === 0) continue;
        const cs = getComputedStyle(el);
        for (const prop of ['color', 'backgroundColor', 'borderColor']) {
            const val = cs[prop];
            const hex = rgbToHex(val);
            if (hex) colors[hex] = (colors[hex] || 0) + 1;
        }
    }
    return Object.entries(colors)
        .sort((a, b) => b[1] - a[1])
        .map(([hex, count]) => ({ hex, count }));
})()
"#;

/// Parse measurement results from browser
pub fn parse_measurements(json: &str) -> eyre::Result<MeasureResult> {
    Ok(serde_json::from_str(json)?)
}

/// Add concise summaries, diagnostics, and contrast checks to measurements.
pub fn enrich_measurements(
    measurements: &mut MeasureResult,
    viewport: &ViewportConfig,
    context: MeasurementContext,
) {
    let selectors: Vec<String> = measurements.counts.keys().cloned().collect();

    for selector in selectors {
        let count = measurements
            .counts
            .get(&selector)
            .copied()
            .unwrap_or_default();
        if count == 0 {
            measurements
                .diagnostics
                .push(format!("Selector `{selector}` matched no elements."));
            continue;
        }

        let css = measurements.css.get(&selector);
        let box_info = measurements.box_info.get(&selector);
        let text = measurements
            .text
            .get(&selector)
            .map(|value| value.trim())
            .unwrap_or_default();

        let mut issues = Vec::new();

        if let Some(css) = css {
            if css.display.as_deref() == Some("none") {
                issues.push("display is none".to_string());
            }
            if css.visibility.as_deref() == Some("hidden") {
                issues.push("visibility is hidden".to_string());
            }
            if css.opacity.as_deref() == Some("0") {
                issues.push("opacity is 0".to_string());
            }
            if !text.is_empty() && css.color_hex.is_none() {
                issues.push("text is present but text color could not be resolved".to_string());
            }
            if !text.is_empty()
                && css.color_hex.is_some()
                && css.color_hex == css.background_color_hex
            {
                issues.push("text and background resolve to the same color".to_string());
            }
            if let Some(font_family) = &css.font_family {
                let lower = font_family.to_lowercase();
                if ["serif", "sans-serif", "monospace"]
                    .iter()
                    .any(|generic| lower.trim() == *generic)
                {
                    issues.push("font family appears to be a generic fallback".to_string());
                }
            }
        }

        if let Some(box_info) = box_info {
            if box_info.width <= 1.0 || box_info.height <= 1.0 {
                issues.push("element has near-zero dimensions".to_string());
            }
            if context.viewport_only_capture && box_info.y >= viewport.height as f64 {
                issues.push("element is below the initial viewport".to_string());
            }
            if context.viewport_only_capture && box_info.y + box_info.height <= 0.0 {
                issues.push("element is above the initial viewport".to_string());
            }
        }

        if !issues.is_empty() {
            measurements
                .diagnostics
                .push(format!("Selector `{selector}`: {}.", issues.join("; ")));
        }

        let summary = build_style_summary(css, box_info, text, &issues);
        measurements.summaries.insert(selector.clone(), summary);

        let contrast = build_contrast_check(css);
        if let Some(contrast) = &contrast
            && contrast.normal_text_aa == Some(false)
        {
            measurements.diagnostics.push(format!(
                "Selector `{selector}` has contrast ratio {:.2}, below WCAG AA for normal text.",
                contrast.ratio.unwrap_or_default()
            ));
        }
        if let Some(contrast) = contrast {
            measurements.contrast.insert(selector, contrast);
        }
    }
}

fn build_style_summary(
    css: Option<&CssProperties>,
    box_info: Option<&BoxInfo>,
    text: &str,
    issues: &[String],
) -> StyleSummary {
    let mut parts = Vec::new();

    if let Some(box_info) = box_info {
        parts.push(format!(
            "box {:.0}x{:.0}px",
            box_info.width, box_info.height
        ));
    }

    if let Some(css) = css {
        let mut typography = Vec::new();
        if let Some(font_size) = &css.font_size {
            typography.push(font_size.clone());
        }
        if let Some(line_height) = &css.line_height {
            typography.push(format!("lh {line_height}"));
        }
        if let Some(font_family) = &css.font_family {
            typography.push(font_family.clone());
        }
        if !typography.is_empty() {
            parts.push(format!("font {}", typography.join(" / ")));
        }

        if css.color_hex.is_some() || css.background_color_hex.is_some() {
            parts.push(format!(
                "colors {} on {}",
                css.color_hex.as_deref().unwrap_or("unknown"),
                css.background_color_hex.as_deref().unwrap_or("transparent")
            ));
        }

        let mut visibility = Vec::new();
        if let Some(display) = &css.display {
            visibility.push(display.clone());
        }
        if let Some(visibility_value) = &css.visibility {
            visibility.push(visibility_value.clone());
        }
        if let Some(opacity) = &css.opacity {
            visibility.push(format!("opacity {opacity}"));
        }
        if !visibility.is_empty() {
            parts.push(format!("visibility {}", visibility.join(", ")));
        }
    }

    if !text.is_empty() {
        parts.push(format!("text {} chars", text.chars().count()));
    }

    StyleSummary {
        summary: parts.join("; "),
        issues: issues.to_vec(),
    }
}

fn build_contrast_check(css: Option<&CssProperties>) -> Option<ContrastCheck> {
    let css = css?;

    if css
        .background_image
        .as_deref()
        .is_some_and(|value| value != "none")
    {
        return Some(ContrastCheck {
            ratio: None,
            normal_text_aa: None,
            large_text_aa: None,
            foreground: css.color_hex.clone(),
            background: css.background_color_hex.clone(),
            reason: Some("background image prevents a reliable solid-color contrast check".into()),
        });
    }

    let Some(foreground) = css.color_hex.as_deref().and_then(parse_hex_color) else {
        return Some(ContrastCheck {
            ratio: None,
            normal_text_aa: None,
            large_text_aa: None,
            foreground: css.color_hex.clone(),
            background: css.background_color_hex.clone(),
            reason: Some("foreground color is unavailable or non-solid".into()),
        });
    };

    let Some(background) = css
        .background_color_hex
        .as_deref()
        .and_then(parse_hex_color)
    else {
        return Some(ContrastCheck {
            ratio: None,
            normal_text_aa: None,
            large_text_aa: None,
            foreground: css.color_hex.clone(),
            background: css.background_color_hex.clone(),
            reason: Some("background color is unavailable or non-solid".into()),
        });
    };

    let ratio = contrast_ratio(foreground, background);

    Some(ContrastCheck {
        ratio: Some((ratio * 100.0).round() / 100.0),
        normal_text_aa: Some(ratio >= 4.5),
        large_text_aa: Some(ratio >= 3.0),
        foreground: css.color_hex.clone(),
        background: css.background_color_hex.clone(),
        reason: None,
    })
}

fn parse_hex_color(value: &str) -> Option<(u8, u8, u8)> {
    if value.len() != 7 || !value.starts_with('#') {
        return None;
    }

    let r = u8::from_str_radix(&value[1..3], 16).ok()?;
    let g = u8::from_str_radix(&value[3..5], 16).ok()?;
    let b = u8::from_str_radix(&value[5..7], 16).ok()?;

    Some((r, g, b))
}

fn contrast_ratio(foreground: (u8, u8, u8), background: (u8, u8, u8)) -> f64 {
    let fg = relative_luminance(foreground);
    let bg = relative_luminance(background);

    if fg > bg {
        (fg + 0.05) / (bg + 0.05)
    } else {
        (bg + 0.05) / (fg + 0.05)
    }
}

fn relative_luminance((r, g, b): (u8, u8, u8)) -> f64 {
    let convert = |value: u8| {
        let channel = value as f64 / 255.0;
        if channel <= 0.03928 {
            channel / 12.92
        } else {
            ((channel + 0.055) / 1.055).powf(2.4)
        }
    };

    0.2126 * convert(r) + 0.7152 * convert(g) + 0.0722 * convert(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_measurements() {
        let json = r##"{
            "css": {
                ".title": {
                    "paddingTop": "24px",
                    "fontSize": "28px",
                    "color": "rgb(0, 0, 0)",
                    "colorHex": "#000000",
                    "backgroundColor": "rgb(255, 255, 255)",
                    "backgroundColorHex": "#ffffff"
                }
            },
            "box_info": {
                ".title": {
                    "x": 100.0,
                    "y": 200.0,
                    "width": 800.0,
                    "height": 60.0
                }
            },
            "counts": {
                ".title": 1
            },
            "text": {
                ".title": "Document Title"
            },
            "errors": []
        }"##;

        let result = parse_measurements(json).expect("Failed to parse");
        assert_eq!(result.css.len(), 1);
        assert_eq!(result.box_info.len(), 1);
        assert_eq!(result.counts.get(".title"), Some(&1));
        assert_eq!(
            result.text.get(".title"),
            Some(&"Document Title".to_string())
        );
        assert_eq!(result.errors.len(), 0);

        // Check hex color conversion
        let title_css = result.css.get(".title").expect("Title CSS not found");
        assert_eq!(title_css.color_hex.as_deref(), Some("#000000"));
        assert_eq!(title_css.background_color_hex.as_deref(), Some("#ffffff"));
    }

    #[test]
    fn test_selectors_for_preset() {
        let doc = selectors_for_preset(MeasurePreset::Document);
        assert!(doc.contains(&"stencila-article".to_string()));
        assert!(!doc.contains(&"stencila-layout".to_string()));

        let site = selectors_for_preset(MeasurePreset::Site);
        assert!(site.contains(&"stencila-layout".to_string()));
        assert!(!site.contains(&"stencila-article".to_string()));

        let all = selectors_for_preset(MeasurePreset::All);
        assert!(all.contains(&"stencila-article".to_string()));
        assert!(all.contains(&"stencila-layout".to_string()));
        assert_eq!(all.len(), doc.len() + site.len());

        let theme = selectors_for_preset(MeasurePreset::Theme);
        assert!(theme.contains(&"header".to_string()));
        assert!(theme.contains(&"main".to_string()));
        assert!(theme.contains(&"footer".to_string()));
    }

    #[test]
    fn test_enrich_measurements_adds_summary_and_contrast() {
        let mut result = MeasureResult {
            css: HashMap::from([(
                ".title".to_string(),
                CssProperties {
                    font_size: Some("28px".into()),
                    line_height: Some("36px".into()),
                    color_hex: Some("#000000".into()),
                    background_color_hex: Some("#ffffff".into()),
                    display: Some("block".into()),
                    visibility: Some("visible".into()),
                    opacity: Some("1".into()),
                    ..Default::default()
                },
            )]),
            box_info: HashMap::from([(
                ".title".to_string(),
                BoxInfo {
                    x: 0.0,
                    y: 0.0,
                    width: 640.0,
                    height: 48.0,
                },
            )]),
            counts: HashMap::from([(".title".to_string(), 1)]),
            text: HashMap::from([(".title".to_string(), "Hello".to_string())]),
            summaries: HashMap::new(),
            contrast: HashMap::new(),
            diagnostics: Vec::new(),
            errors: Vec::new(),
        };

        enrich_measurements(
            &mut result,
            &ViewportConfig::default(),
            MeasurementContext {
                viewport_only_capture: true,
            },
        );

        assert!(result.summaries.contains_key(".title"));
        assert_eq!(result.contrast[".title"].normal_text_aa, Some(true));
    }

    #[test]
    fn test_enrich_measurements_skips_viewport_position_diagnostics_when_not_viewport_only() {
        let mut result = MeasureResult {
            css: HashMap::new(),
            box_info: HashMap::from([(
                ".footer".to_string(),
                BoxInfo {
                    x: 0.0,
                    y: 2000.0,
                    width: 640.0,
                    height: 48.0,
                },
            )]),
            counts: HashMap::from([(".footer".to_string(), 1)]),
            text: HashMap::new(),
            summaries: HashMap::new(),
            contrast: HashMap::new(),
            diagnostics: Vec::new(),
            errors: Vec::new(),
        };

        enrich_measurements(
            &mut result,
            &ViewportConfig::default(),
            MeasurementContext {
                viewport_only_capture: false,
            },
        );

        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn test_parse_hex_color_rejects_alpha() {
        assert_eq!(parse_hex_color("#ffffff"), Some((255, 255, 255)));
        assert_eq!(parse_hex_color("#ffffffff"), None);
    }
}
