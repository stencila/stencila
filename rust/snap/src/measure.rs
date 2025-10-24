//! CSS and layout measurement

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

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

    /// Diagnostic errors and warnings
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub errors: Vec<String>,
}

/// Common CSS properties
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
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
/// This function collects computed styles, bounding boxes, counts, and text content
/// for specified selectors and returns them as JSON
pub const MEASUREMENT_SCRIPT: &str = r#"
(function(selector) {
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

    // If selector provided, measure just that element
    // Otherwise measure common selectors
    const selectors = selector ? [selector] : [
        '[slot=title]',
        'stencila-article',
        'section',
        'figure',
        'h1',
        'h2'
    ];

    for (const sel of selectors) {
        // Count elements
        const elements = document.querySelectorAll(sel);
        result.counts[sel] = elements.length;

        if (elements.length === 0) {
            if (selector) {
                // Only report error for user-specified selectors
                result.errors.push(`Selector '${sel}' matched 0 elements`);
            }
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

/// Parse measurement results from browser
pub fn parse_measurements(json: &str) -> eyre::Result<MeasureResult> {
    Ok(serde_json::from_str(json)?)
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
}
