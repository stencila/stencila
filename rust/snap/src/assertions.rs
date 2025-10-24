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

        // For count/exists, no property needed
        if domain == "count" || domain == "exists" {
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

        let actual_value = match self.domain.as_str() {
            "css" => {
                // Safe: parse() ensures property is present for css domain
                let prop = self
                    .property
                    .as_ref()
                    .expect("property required for css domain");

                if element_count == 0 {
                    format!(
                        "element not found (selector '{}' matched 0 elements)",
                        self.selector
                    )
                } else {
                    measurements
                        .css
                        .get(&self.selector)
                        .and_then(|css| get_css_property(css, prop))
                        .unwrap_or_else(|| format!("property '{}' not available", prop))
                }
            }
            "box" => {
                // Safe: parse() ensures property is present for box domain
                let prop = self
                    .property
                    .as_ref()
                    .expect("property required for box domain");

                if element_count == 0 {
                    format!(
                        "element not found (selector '{}' matched 0 elements)",
                        self.selector
                    )
                } else {
                    measurements
                        .box_info
                        .get(&self.selector)
                        .and_then(|box_info| get_box_property(box_info, prop))
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| format!("property '{}' not available", prop))
                }
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
            "text" => {
                if element_count == 0 {
                    format!(
                        "element not found (selector '{}' matched 0 elements)",
                        self.selector
                    )
                } else {
                    measurements
                        .text
                        .get(&self.selector)
                        .cloned()
                        .unwrap_or_else(|| "text not available".to_string())
                }
            }
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
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AssertionResults {
    pub passed: bool,
    pub failures: Vec<AssertionResult>,
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

    let close_paren = input[open_paren..]
        .find(')')
        .ok_or_else(|| eyre::eyre!("Missing ')'"))?;
    let selector = input[open_paren + 1..open_paren + close_paren]
        .trim()
        .to_string();

    let rest = &input[open_paren + close_paren + 1..];

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
        "backgroundImage" => css.background_image.clone(),
        "backgroundSize" => css.background_size.clone(),
        "backgroundPosition" => css.background_position.clone(),
        // Borders
        "borderWidth" => css.border_width.clone(),
        "borderColor" => css.border_color.clone(),
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
            let actual_num = actual
                .trim_end_matches("px")
                .trim()
                .parse::<f64>()
                .unwrap_or(0.0);

            match operator {
                Operator::Equals => (actual_num - expected_num).abs() < 0.5,
                Operator::NotEquals => (actual_num - expected_num).abs() >= 0.5,
                Operator::GreaterThan => actual_num > *expected_num,
                Operator::GreaterThanOrEqual => actual_num >= *expected_num,
                Operator::LessThan => actual_num < *expected_num,
                Operator::LessThanOrEqual => actual_num <= *expected_num,
            }
        }
        Value::String(expected_str) => match operator {
            Operator::Equals => actual == expected_str,
            Operator::NotEquals => actual != expected_str,
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
}
