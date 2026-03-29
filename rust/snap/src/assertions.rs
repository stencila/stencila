//! Assertion parsing and evaluation

use std::fmt;

use eyre::Result;
use serde::{Deserialize, Serialize};

use crate::measure::MeasureResult;

/// Parsed assertion expression
#[derive(Debug, Clone)]
pub struct Assertion {
    /// Domain: "css", "box", "count", "exists", "text"
    pub domain: String,

    /// CSS selector
    pub selector: String,

    /// Property name (for css/box domains)
    pub property: Option<String>,

    /// Comparison operator
    pub operator: Operator,

    /// Expected value
    pub value: Value,
}

/// Comparison operators
#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Contains,
}

/// Value types for assertions
#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
}

impl Assertion {
    /// Parse an assertion string like "css(.title).paddingTop >= 24px"
    pub fn parse(input: &str) -> Result<Self> {
        let input = input.trim();

        // Parse domain and selector: "css(.title)" or "count(section)"
        let (domain, selector, rest) = parse_domain_selector(input)?;

        // For count/exists/text, no property needed
        if domain == "count" || domain == "exists" || domain == "text" {
            let (operator, value) = parse_operator_value(rest)?;
            return Ok(Self {
                domain,
                selector,
                property: None,
                operator,
                value,
            });
        }

        // For css/box, parse property: ".paddingTop"
        let (property, rest) = parse_property(rest)?;
        let (operator, value) = parse_operator_value(rest)?;

        Ok(Self {
            domain,
            selector,
            property: Some(property),
            operator,
            value,
        })
    }

    /// Evaluate this assertion against measurement results
    pub fn evaluate(&self, measurements: &MeasureResult) -> AssertionResult {
        // Check if element exists first
        let element_count = measurements
            .counts
            .get(&self.selector)
            .copied()
            .unwrap_or(0);

        // Short-circuit: when the element doesn't exist and the domain
        // requires an element, fail immediately rather than comparing
        // error-description strings against numeric/string expectations.
        if element_count == 0 && self.domain != "count" && self.domain != "exists" {
            return AssertionResult {
                assertion: format!("{self}"),
                passed: false,
                expected: format!("{:?}", self.value),
                actual: format!(
                    "element not found (selector '{}' matched 0 elements)",
                    self.selector
                ),
                message: format!("Selector '{}' matched no elements", self.selector),
            };
        }

        let actual_value = match self.domain.as_str() {
            "css" => {
                // Safe: parse() ensures property is present for css domain
                let prop = self
                    .property
                    .as_ref()
                    .expect("property required for css domain");

                measurements
                    .css
                    .get(&self.selector)
                    .and_then(|css| get_css_property(css, prop))
                    .unwrap_or_else(|| format!("property '{}' not available", prop))
            }
            "box" => {
                // Safe: parse() ensures property is present for box domain
                let prop = self
                    .property
                    .as_ref()
                    .expect("property required for box domain");

                measurements
                    .box_info
                    .get(&self.selector)
                    .and_then(|box_info| get_box_property(box_info, prop))
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| format!("property '{}' not available", prop))
            }
            "count" => measurements
                .counts
                .get(&self.selector)
                .map(|c| c.to_string())
                .unwrap_or_else(|| "0".to_string()),
            "exists" => measurements
                .counts
                .get(&self.selector)
                .map(|c| (*c > 0).to_string())
                .unwrap_or_else(|| "false".to_string()),
            "text" => measurements
                .text
                .get(&self.selector)
                .cloned()
                .unwrap_or_else(|| "text not available".to_string()),
            _ => "unsupported domain".to_string(),
        };

        let passed = evaluate_comparison(&actual_value, &self.operator, &self.value);

        let message = if passed {
            String::new()
        } else {
            format!(
                "Expected {} {} {:?}, got {}",
                self.domain, self.operator, self.value, actual_value
            )
        };

        AssertionResult {
            assertion: format!("{self}"),
            passed,
            expected: format!("{:?}", self.value),
            actual: actual_value,
            message,
        }
    }
}

impl fmt::Display for Assertion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(prop) = &self.property {
            write!(
                f,
                "{}({}).{} {} {:?}",
                self.domain, self.selector, prop, self.operator, self.value
            )
        } else {
            write!(
                f,
                "{}({}) {} {:?}",
                self.domain, self.selector, self.operator, self.value
            )
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Equals => write!(f, "=="),
            Self::NotEquals => write!(f, "!="),
            Self::GreaterThan => write!(f, ">"),
            Self::GreaterThanOrEqual => write!(f, ">="),
            Self::LessThan => write!(f, "<"),
            Self::LessThanOrEqual => write!(f, "<="),
            Self::Contains => write!(f, "~="),
        }
    }
}

/// Result of evaluating a single assertion
#[derive(Debug, Serialize, Deserialize)]
pub struct AssertionResult {
    pub assertion: String,
    pub passed: bool,
    pub expected: String,
    pub actual: String,
    pub message: String,
}

/// Collection of assertion results
#[derive(Debug, Serialize, Deserialize)]
pub struct AssertionResults {
    pub passed: bool,
    pub failures: Vec<AssertionResult>,
}

impl Default for AssertionResults {
    fn default() -> Self {
        Self {
            passed: true,
            failures: vec![],
        }
    }
}

impl AssertionResults {
    /// Evaluate multiple assertions
    pub fn evaluate(assertions: &[Assertion], measurements: &MeasureResult) -> Result<Self> {
        let results: Vec<AssertionResult> = assertions
            .iter()
            .map(|a| a.evaluate(measurements))
            .collect();

        let failures: Vec<AssertionResult> = results.into_iter().filter(|r| !r.passed).collect();

        let passed = failures.is_empty();

        Ok(Self { passed, failures })
    }
}

// Helper functions

fn parse_domain_selector(input: &str) -> Result<(String, String, &str)> {
    let open_paren = input.find('(').ok_or_else(|| eyre::eyre!("Missing '('"))?;
    let domain = input[..open_paren].trim().to_string();

    // Find the matching closing parenthesis using depth tracking,
    // so that nested parens in selectors like :nth-child(2) or
    // :not(.foo) are handled correctly.
    let mut depth = 0u32;
    let mut close_pos = None;
    for (i, ch) in input[open_paren..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    close_pos = Some(open_paren + i);
                    break;
                }
            }
            _ => {}
        }
    }
    let close_paren = close_pos.ok_or_else(|| eyre::eyre!("Missing closing ')'"))?;

    let selector = input[open_paren + 1..close_paren].trim().to_string();
    let rest = &input[close_paren + 1..];

    Ok((domain, selector, rest))
}

fn parse_property(input: &str) -> Result<(String, &str)> {
    let input = input.trim_start_matches('.');
    let end = input
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(input.len());

    let property = input[..end].to_string();
    let rest = &input[end..];

    Ok((property, rest))
}

fn parse_operator_value(input: &str) -> Result<(Operator, Value)> {
    let input = input.trim();

    let (operator, value_str) = if let Some(rest) = input.strip_prefix(">=") {
        (Operator::GreaterThanOrEqual, rest)
    } else if let Some(rest) = input.strip_prefix("<=") {
        (Operator::LessThanOrEqual, rest)
    } else if let Some(rest) = input.strip_prefix("==") {
        (Operator::Equals, rest)
    } else if let Some(rest) = input.strip_prefix("!=") {
        (Operator::NotEquals, rest)
    } else if let Some(rest) = input.strip_prefix("~=") {
        (Operator::Contains, rest)
    } else if let Some(rest) = input.strip_prefix('>') {
        (Operator::GreaterThan, rest)
    } else if let Some(rest) = input.strip_prefix('<') {
        (Operator::LessThan, rest)
    } else {
        eyre::bail!("No operator found in: {}", input);
    };

    let value_str = value_str.trim();

    // Try to parse as number (strip "px" suffix if present)
    let value = if let Some(num_str) = value_str.strip_suffix("px") {
        Value::Number(num_str.trim().parse()?)
    } else if let Ok(num) = value_str.parse::<f64>() {
        Value::Number(num)
    } else if value_str == "true" {
        Value::Boolean(true)
    } else if value_str == "false" {
        Value::Boolean(false)
    } else {
        Value::String(value_str.trim_matches('"').to_string())
    };

    Ok((operator, value))
}

fn get_css_property(css: &crate::measure::CssProperties, prop: &str) -> Option<String> {
    match prop {
        // Spacing
        "paddingTop" => css.padding_top.clone(),
        "paddingBottom" => css.padding_bottom.clone(),
        "paddingLeft" => css.padding_left.clone(),
        "paddingRight" => css.padding_right.clone(),
        "marginTop" => css.margin_top.clone(),
        "marginBottom" => css.margin_bottom.clone(),
        "marginLeft" => css.margin_left.clone(),
        "marginRight" => css.margin_right.clone(),
        // Typography
        "fontSize" => css.font_size.clone(),
        "lineHeight" => css.line_height.clone(),
        "color" => css.color.clone(),
        "colorHex" => css.color_hex.clone(),
        "fontFamily" => css.font_family.clone(),
        "fontWeight" => css.font_weight.clone(),
        "textAlign" => css.text_align.clone(),
        "textDecoration" => css.text_decoration.clone(),
        "letterSpacing" => css.letter_spacing.clone(),
        "textTransform" => css.text_transform.clone(),
        "whiteSpace" => css.white_space.clone(),
        // Display
        "display" => css.display.clone(),
        "visibility" => css.visibility.clone(),
        "opacity" => css.opacity.clone(),
        // Backgrounds
        "backgroundColor" => css.background_color.clone(),
        "backgroundColorHex" => css.background_color_hex.clone(),
        "backgroundImage" => css.background_image.clone(),
        "backgroundSize" => css.background_size.clone(),
        "backgroundPosition" => css.background_position.clone(),
        // Borders
        "borderWidth" => css.border_width.clone(),
        "borderColor" => css.border_color.clone(),
        "borderColorHex" => css.border_color_hex.clone(),
        "borderRadius" => css.border_radius.clone(),
        "borderStyle" => css.border_style.clone(),
        "borderTopWidth" => css.border_top_width.clone(),
        "borderRightWidth" => css.border_right_width.clone(),
        "borderBottomWidth" => css.border_bottom_width.clone(),
        "borderLeftWidth" => css.border_left_width.clone(),
        // Layout
        "position" => css.position.clone(),
        "top" => css.top.clone(),
        "right" => css.right.clone(),
        "bottom" => css.bottom.clone(),
        "left" => css.left.clone(),
        "zIndex" => css.z_index.clone(),
        "overflow" => css.overflow.clone(),
        "overflowX" => css.overflow_x.clone(),
        "overflowY" => css.overflow_y.clone(),
        "minHeight" => css.min_height.clone(),
        "maxWidth" => css.max_width.clone(),
        // Flexbox
        "gap" => css.gap.clone(),
        "justifyContent" => css.justify_content.clone(),
        "alignItems" => css.align_items.clone(),
        "flexDirection" => css.flex_direction.clone(),
        // Visual effects
        "boxShadow" => css.box_shadow.clone(),
        "transform" => css.transform.clone(),
        "filter" => css.filter.clone(),
        _ => None,
    }
}

fn get_box_property(box_info: &crate::measure::BoxInfo, prop: &str) -> Option<f64> {
    match prop {
        "x" => Some(box_info.x),
        "y" => Some(box_info.y),
        "width" => Some(box_info.width),
        "height" => Some(box_info.height),
        _ => None,
    }
}

fn evaluate_comparison(actual: &str, operator: &Operator, expected: &Value) -> bool {
    match expected {
        Value::Number(expected_num) => {
            // Extract numeric value from actual (strip "px" if present)
            let Ok(actual_num) = actual.trim_end_matches("px").trim().parse::<f64>() else {
                return false;
            };

            match operator {
                Operator::Equals => (actual_num - expected_num).abs() < 0.5,
                Operator::NotEquals => (actual_num - expected_num).abs() >= 0.5,
                Operator::GreaterThan => actual_num > *expected_num,
                Operator::GreaterThanOrEqual => actual_num >= *expected_num,
                Operator::LessThan => actual_num < *expected_num,
                Operator::LessThanOrEqual => actual_num <= *expected_num,
                Operator::Contains => false,
            }
        }
        Value::String(expected_str) => match operator {
            Operator::Equals => actual == expected_str,
            Operator::NotEquals => actual != expected_str,
            Operator::Contains => actual.contains(expected_str.as_str()),
            _ => false,
        },
        Value::Boolean(expected_bool) => {
            let actual_bool = actual == "true";
            match operator {
                Operator::Equals => actual_bool == *expected_bool,
                Operator::NotEquals => actual_bool != *expected_bool,
                _ => false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_assertion() {
        let assertion =
            Assertion::parse("css(.title).paddingTop >= 24px").expect("Failed to parse assertion");
        assert_eq!(assertion.domain, "css");
        assert_eq!(assertion.selector, ".title");
        assert_eq!(assertion.property, Some("paddingTop".to_string()));
        assert!(matches!(assertion.operator, Operator::GreaterThanOrEqual));
        assert!(matches!(assertion.value, Value::Number(24.0)));

        let count_assertion =
            Assertion::parse("count(section) == 5").expect("Failed to parse count assertion");
        assert_eq!(count_assertion.domain, "count");
        assert_eq!(count_assertion.selector, "section");
        assert!(count_assertion.property.is_none());
    }

    #[test]
    fn test_evaluate_comparison() {
        assert!(evaluate_comparison(
            "24px",
            &Operator::Equals,
            &Value::Number(24.0)
        ));
        assert!(evaluate_comparison(
            "30px",
            &Operator::GreaterThan,
            &Value::Number(24.0)
        ));
        assert!(!evaluate_comparison(
            "20px",
            &Operator::GreaterThan,
            &Value::Number(24.0)
        ));
    }

    #[test]
    fn test_parse_contains_assertion() {
        let assertion = Assertion::parse("css(stencila-heading).fontFamily~=Source Serif")
            .expect("Failed to parse contains assertion");
        assert_eq!(assertion.domain, "css");
        assert_eq!(assertion.selector, "stencila-heading");
        assert_eq!(assertion.property, Some("fontFamily".to_string()));
        assert!(matches!(assertion.operator, Operator::Contains));
        assert!(matches!(assertion.value, Value::String(ref s) if s == "Source Serif"));
    }

    #[test]
    fn test_evaluate_contains_comparison() {
        assert!(evaluate_comparison(
            "\"Source Serif 4\", Georgia, serif",
            &Operator::Contains,
            &Value::String("Source Serif".to_string())
        ));
        assert!(!evaluate_comparison(
            "\"Source Serif 4\", Georgia, serif",
            &Operator::Contains,
            &Value::String("Roboto".to_string())
        ));
    }

    #[test]
    fn test_non_numeric_actual_value_does_not_coerce_to_zero() {
        // An error string should NOT be treated as 0.0
        assert!(!evaluate_comparison(
            "element not found (selector '.missing' matched 0 elements)",
            &Operator::LessThanOrEqual,
            &Value::Number(10.0)
        ));
        assert!(!evaluate_comparison(
            "property 'width' not available",
            &Operator::Equals,
            &Value::Number(0.0)
        ));
    }

    #[test]
    fn test_parse_nested_parentheses_selectors() {
        let assertion = Assertion::parse("css(li:nth-child(2)).fontSize >= 16px")
            .expect("Failed to parse nth-child selector");
        assert_eq!(assertion.selector, "li:nth-child(2)");
        assert_eq!(assertion.property, Some("fontSize".to_string()));

        let assertion = Assertion::parse("count(div:not(.hidden)) >= 1")
            .expect("Failed to parse :not selector");
        assert_eq!(assertion.selector, "div:not(.hidden)");

        let assertion = Assertion::parse("exists(section:has(.title)) == true")
            .expect("Failed to parse :has selector");
        assert_eq!(assertion.selector, "section:has(.title)");
    }

    #[test]
    fn test_missing_element_assertion_fails() {
        let measurements = MeasureResult {
            css: std::collections::HashMap::new(),
            box_info: std::collections::HashMap::new(),
            counts: std::collections::HashMap::new(),
            text: std::collections::HashMap::new(),
            summaries: std::collections::HashMap::new(),
            contrast: std::collections::HashMap::new(),
            diagnostics: Vec::new(),
            errors: Vec::new(),
        };

        let assertion = Assertion::parse("css(.missing).width >= 0px").expect("should parse");
        let result = assertion.evaluate(&measurements);
        assert!(!result.passed, "assertion on missing element should fail");
        assert!(
            result.actual.contains("matched 0 elements"),
            "actual should mention missing element"
        );
    }

    #[test]
    fn test_parse_text_assertion() {
        let assertion =
            Assertion::parse("text(h1) ~= Welcome").expect("Failed to parse text assertion");
        assert_eq!(assertion.domain, "text");
        assert_eq!(assertion.selector, "h1");
        assert!(assertion.property.is_none());
        assert!(matches!(assertion.operator, Operator::Contains));
        assert!(matches!(assertion.value, Value::String(ref s) if s == "Welcome"));

        let assertion = Assertion::parse("text(.title) == \"Hello World\"")
            .expect("Failed to parse text equals assertion");
        assert_eq!(assertion.domain, "text");
        assert_eq!(assertion.selector, ".title");
        assert!(assertion.property.is_none());
        assert!(matches!(assertion.operator, Operator::Equals));
        assert!(matches!(assertion.value, Value::String(ref s) if s == "Hello World"));
    }

    #[test]
    fn test_evaluate_text_assertion() {
        let measurements = MeasureResult {
            css: std::collections::HashMap::new(),
            box_info: std::collections::HashMap::new(),
            counts: std::collections::HashMap::from([("h1".to_string(), 1)]),
            text: std::collections::HashMap::from([("h1".to_string(), "Welcome Home".to_string())]),
            summaries: std::collections::HashMap::new(),
            contrast: std::collections::HashMap::new(),
            diagnostics: Vec::new(),
            errors: Vec::new(),
        };

        let assertion = Assertion::parse("text(h1) ~= Welcome").expect("should parse");
        let result = assertion.evaluate(&measurements);
        assert!(result.passed, "text contains should match");

        let assertion = Assertion::parse("text(h1) == Welcome Home").expect("should parse");
        let result = assertion.evaluate(&measurements);
        assert!(result.passed, "text equals should match");
    }

    #[test]
    fn test_get_css_property_covers_all_fields() {
        use crate::measure::CssProperties;

        let sentinel = Some("test".to_string());
        let css = CssProperties {
            padding_top: sentinel.clone(),
            padding_bottom: sentinel.clone(),
            padding_left: sentinel.clone(),
            padding_right: sentinel.clone(),
            margin_top: sentinel.clone(),
            margin_bottom: sentinel.clone(),
            margin_left: sentinel.clone(),
            margin_right: sentinel.clone(),
            font_size: sentinel.clone(),
            line_height: sentinel.clone(),
            color: sentinel.clone(),
            color_hex: sentinel.clone(),
            font_family: sentinel.clone(),
            font_weight: sentinel.clone(),
            text_align: sentinel.clone(),
            text_decoration: sentinel.clone(),
            letter_spacing: sentinel.clone(),
            text_transform: sentinel.clone(),
            white_space: sentinel.clone(),
            display: sentinel.clone(),
            visibility: sentinel.clone(),
            opacity: sentinel.clone(),
            background_color: sentinel.clone(),
            background_color_hex: sentinel.clone(),
            background_image: sentinel.clone(),
            background_size: sentinel.clone(),
            background_position: sentinel.clone(),
            border_width: sentinel.clone(),
            border_color: sentinel.clone(),
            border_color_hex: sentinel.clone(),
            border_radius: sentinel.clone(),
            border_style: sentinel.clone(),
            border_top_width: sentinel.clone(),
            border_right_width: sentinel.clone(),
            border_bottom_width: sentinel.clone(),
            border_left_width: sentinel.clone(),
            position: sentinel.clone(),
            top: sentinel.clone(),
            right: sentinel.clone(),
            bottom: sentinel.clone(),
            left: sentinel.clone(),
            z_index: sentinel.clone(),
            overflow: sentinel.clone(),
            overflow_x: sentinel.clone(),
            overflow_y: sentinel.clone(),
            min_height: sentinel.clone(),
            max_width: sentinel.clone(),
            gap: sentinel.clone(),
            justify_content: sentinel.clone(),
            align_items: sentinel.clone(),
            flex_direction: sentinel.clone(),
            box_shadow: sentinel.clone(),
            transform: sentinel.clone(),
            filter: sentinel.clone(),
        };

        let camel_case_names = [
            "paddingTop",
            "paddingBottom",
            "paddingLeft",
            "paddingRight",
            "marginTop",
            "marginBottom",
            "marginLeft",
            "marginRight",
            "fontSize",
            "lineHeight",
            "color",
            "colorHex",
            "fontFamily",
            "fontWeight",
            "textAlign",
            "textDecoration",
            "letterSpacing",
            "textTransform",
            "whiteSpace",
            "display",
            "visibility",
            "opacity",
            "backgroundColor",
            "backgroundColorHex",
            "backgroundImage",
            "backgroundSize",
            "backgroundPosition",
            "borderWidth",
            "borderColor",
            "borderColorHex",
            "borderRadius",
            "borderStyle",
            "borderTopWidth",
            "borderRightWidth",
            "borderBottomWidth",
            "borderLeftWidth",
            "position",
            "top",
            "right",
            "bottom",
            "left",
            "zIndex",
            "overflow",
            "overflowX",
            "overflowY",
            "minHeight",
            "maxWidth",
            "gap",
            "justifyContent",
            "alignItems",
            "flexDirection",
            "boxShadow",
            "transform",
            "filter",
        ];

        for name in camel_case_names {
            assert!(
                get_css_property(&css, name).is_some(),
                "get_css_property should handle '{name}'"
            );
        }
    }
}
