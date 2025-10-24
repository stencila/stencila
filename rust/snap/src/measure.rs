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
}

/// Common CSS properties
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CssProperties {
    pub padding_top: Option<String>,

    pub padding_bottom: Option<String>,

    pub padding_left: Option<String>,

    pub padding_right: Option<String>,

    pub margin_top: Option<String>,

    pub margin_bottom: Option<String>,

    pub margin_left: Option<String>,

    pub margin_right: Option<String>,

    pub font_size: Option<String>,

    pub line_height: Option<String>,

    pub color: Option<String>,

    pub font_family: Option<String>,

    pub font_weight: Option<String>,

    pub display: Option<String>,

    pub visibility: Option<String>,

    pub opacity: Option<String>,
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
    const result = {
        css: {},
        box_info: {},
        counts: {},
        text: {}
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

        if (elements.length === 0) continue;

        // Get first element for measurements
        const el = elements[0];

        // Computed styles
        const cs = getComputedStyle(el);
        result.css[sel] = {
            paddingTop: cs.paddingTop,
            paddingBottom: cs.paddingBottom,
            paddingLeft: cs.paddingLeft,
            paddingRight: cs.paddingRight,
            marginTop: cs.marginTop,
            marginBottom: cs.marginBottom,
            marginLeft: cs.marginLeft,
            marginRight: cs.marginRight,
            fontSize: cs.fontSize,
            lineHeight: cs.lineHeight,
            color: cs.color,
            fontFamily: cs.fontFamily,
            fontWeight: cs.fontWeight,
            display: cs.display,
            visibility: cs.visibility,
            opacity: cs.opacity
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
        let json = r#"{
            "css": {
                ".title": {
                    "paddingTop": "24px",
                    "fontSize": "28px",
                    "color": "rgb(0, 0, 0)"
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
            }
        }"#;

        let result = parse_measurements(json).expect("Failed to parse");
        assert_eq!(result.css.len(), 1);
        assert_eq!(result.box_info.len(), 1);
        assert_eq!(result.counts.get(".title"), Some(&1));
        assert_eq!(
            result.text.get(".title"),
            Some(&"Document Title".to_string())
        );
    }
}
