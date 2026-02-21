//! Stylesheet parser for the model stylesheet grammar (§8.2).
//!
//! Parses CSS-like rules that configure LLM properties on pipeline nodes.
//! Extracted into its own module so the `stylesheet_syntax` lint rule (§7.2)
//! can validate stylesheets before the full stylesheet application module exists.
//!
//! # Grammar (EBNF from §8.2)
//!
//! ```text
//! Stylesheet    ::= Rule+
//! Rule          ::= Selector '{' Declaration ( ';' Declaration )* ';'? '}'
//! Selector      ::= '*' | '#' Identifier | '.' ClassName
//! ClassName     ::= [a-z0-9-]+
//! Declaration   ::= Property ':' PropertyValue
//! Property      ::= 'llm_model' | 'llm_provider' | 'reasoning_effort'
//! PropertyValue ::= QuotedString | Identifier
//! ```

use crate::error::{AttractorError, AttractorResult};

/// The set of property names allowed by the §8.2 grammar.
///
/// Shared by the parser (for syntax validation) and `stylesheet.rs` (for application).
pub const ALLOWED_PROPERTIES: &[&str] = &["llm_model", "llm_provider", "reasoning_effort"];

/// Valid values for `reasoning_effort` per §8.4.
const REASONING_EFFORT_VALUES: &[&str] = &["low", "medium", "high"];

/// A parsed selector from the stylesheet grammar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selector {
    /// `*` — matches all nodes.
    Universal,
    /// `.class-name` — matches nodes with the given class.
    Class(String),
    /// `#node-id` — matches a specific node by ID.
    Id(String),
}

impl Selector {
    /// The specificity of this selector (§8.3).
    ///
    /// Universal = 0, Class = 1, Id = 2.
    #[must_use]
    pub fn specificity(&self) -> u8 {
        match self {
            Self::Universal => 0,
            Self::Class(_) => 1,
            Self::Id(_) => 2,
        }
    }
}

/// A single property declaration within a rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration {
    /// The property name (e.g., `llm_model`, `llm_provider`, `reasoning_effort`).
    pub property: String,
    /// The property value.
    pub value: String,
}

/// A parsed stylesheet rule: selector + declarations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StylesheetRule {
    /// The selector that determines which nodes this rule applies to.
    pub selector: Selector,
    /// The property declarations within this rule.
    pub declarations: Vec<Declaration>,
}

/// A parsed stylesheet consisting of one or more rules.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedStylesheet {
    /// The rules in source order.
    pub rules: Vec<StylesheetRule>,
}

/// Parse a model stylesheet string into a [`ParsedStylesheet`].
///
/// # Errors
///
/// Returns [`AttractorError::InvalidPipeline`] if the stylesheet is malformed.
pub fn parse_stylesheet(input: &str) -> AttractorResult<ParsedStylesheet> {
    let input = input.trim();
    if input.is_empty() {
        return Ok(ParsedStylesheet { rules: Vec::new() });
    }

    let mut rules = Vec::new();
    let mut pos = 0;
    let bytes = input.as_bytes();

    while pos < bytes.len() {
        // Skip whitespace
        pos = skip_whitespace(input, pos);
        if pos >= bytes.len() {
            break;
        }

        // Parse selector
        let (selector, new_pos) = parse_selector(input, pos)?;
        pos = skip_whitespace(input, new_pos);

        // Expect '{'
        if pos >= bytes.len() || bytes[pos] != b'{' {
            return Err(stylesheet_error(input, pos, "expected '{' after selector"));
        }
        pos += 1;

        // Parse declarations until '}'
        let mut declarations = Vec::new();
        loop {
            pos = skip_whitespace(input, pos);
            if pos >= bytes.len() {
                return Err(stylesheet_error(
                    input,
                    pos,
                    "unexpected end of input, expected '}'",
                ));
            }
            if bytes[pos] == b'}' {
                pos += 1;
                break;
            }

            // Parse declaration
            let (decl, new_pos) = parse_declaration(input, pos)?;
            declarations.push(decl);
            pos = skip_whitespace(input, new_pos);

            // Optional semicolon
            if pos < bytes.len() && bytes[pos] == b';' {
                pos += 1;
            }
        }

        rules.push(StylesheetRule {
            selector,
            declarations,
        });
    }

    Ok(ParsedStylesheet { rules })
}

fn stylesheet_error(input: &str, pos: usize, message: &str) -> AttractorError {
    let context = if pos < input.len() {
        let end = (pos + 20).min(input.len());
        format!(" near `{}`", &input[pos..end])
    } else {
        String::new()
    };
    AttractorError::InvalidPipeline {
        reason: format!("stylesheet parse error at position {pos}: {message}{context}"),
    }
}

fn skip_whitespace(input: &str, mut pos: usize) -> usize {
    let bytes = input.as_bytes();
    while pos < bytes.len() && bytes[pos].is_ascii_whitespace() {
        pos += 1;
    }
    pos
}

fn parse_selector(input: &str, pos: usize) -> AttractorResult<(Selector, usize)> {
    let bytes = input.as_bytes();
    if pos >= bytes.len() {
        return Err(stylesheet_error(input, pos, "expected selector"));
    }

    match bytes[pos] {
        b'*' => Ok((Selector::Universal, pos + 1)),
        b'#' => {
            let start = pos + 1;
            let end = scan_identifier(input, start);
            if end == start {
                return Err(stylesheet_error(
                    input,
                    pos,
                    "expected identifier after '#'",
                ));
            }
            Ok((Selector::Id(input[start..end].to_string()), end))
        }
        b'.' => {
            let start = pos + 1;
            let end = scan_class_name(input, start);
            if end == start {
                return Err(stylesheet_error(
                    input,
                    pos,
                    "expected class name after '.'",
                ));
            }
            Ok((Selector::Class(input[start..end].to_string()), end))
        }
        _ => Err(stylesheet_error(
            input,
            pos,
            "expected '*', '#', or '.' to start a selector",
        )),
    }
}

/// Scan an identifier: `[A-Za-z_][A-Za-z0-9_]*`
fn scan_identifier(input: &str, start: usize) -> usize {
    let bytes = input.as_bytes();
    if start >= bytes.len() {
        return start;
    }
    let first = bytes[start];
    if !first.is_ascii_alphabetic() && first != b'_' {
        return start;
    }
    let mut pos = start + 1;
    while pos < bytes.len() && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_') {
        pos += 1;
    }
    pos
}

/// Scan a class name: `[a-z0-9-]+`
fn scan_class_name(input: &str, start: usize) -> usize {
    let bytes = input.as_bytes();
    let mut pos = start;
    while pos < bytes.len()
        && (bytes[pos].is_ascii_lowercase() || bytes[pos].is_ascii_digit() || bytes[pos] == b'-')
    {
        pos += 1;
    }
    pos
}

fn parse_declaration(input: &str, pos: usize) -> AttractorResult<(Declaration, usize)> {
    let bytes = input.as_bytes();

    // Parse property name
    let prop_start = pos;
    let prop_end = scan_identifier(input, prop_start);
    if prop_end == prop_start {
        return Err(stylesheet_error(input, pos, "expected property name"));
    }
    let property = input[prop_start..prop_end].to_string();

    // Validate property name against §8.2 grammar
    if !ALLOWED_PROPERTIES.contains(&property.as_str()) {
        return Err(stylesheet_error(
            input,
            prop_start,
            &format!(
                "unknown property `{property}` (allowed: {})",
                ALLOWED_PROPERTIES.join(", ")
            ),
        ));
    }

    // Skip whitespace, expect ':'
    let colon_pos = skip_whitespace(input, prop_end);
    if colon_pos >= bytes.len() || bytes[colon_pos] != b':' {
        return Err(stylesheet_error(
            input,
            colon_pos,
            "expected ':' after property name",
        ));
    }
    let value_start = skip_whitespace(input, colon_pos + 1);

    // Parse value: either quoted string or unquoted token
    let (value, end_pos) = if value_start < bytes.len() && bytes[value_start] == b'"' {
        parse_quoted_string(input, value_start)?
    } else {
        parse_unquoted_value(input, value_start)?
    };

    // Validate reasoning_effort values per §8.4
    if property == "reasoning_effort" && !REASONING_EFFORT_VALUES.contains(&value.as_str()) {
        return Err(stylesheet_error(
            input,
            value_start,
            &format!(
                "invalid reasoning_effort value `{value}` (allowed: {})",
                REASONING_EFFORT_VALUES.join(", ")
            ),
        ));
    }

    Ok((Declaration { property, value }, end_pos))
}

fn parse_quoted_string(input: &str, pos: usize) -> AttractorResult<(String, usize)> {
    let bytes = input.as_bytes();
    debug_assert!(bytes[pos] == b'"');

    let mut result = String::new();
    let mut i = pos + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => return Ok((result, i + 1)),
            b'\\' if i + 1 < bytes.len() => {
                match bytes[i + 1] {
                    b'"' => result.push('"'),
                    b'\\' => result.push('\\'),
                    b'n' => result.push('\n'),
                    b't' => result.push('\t'),
                    other => {
                        result.push('\\');
                        result.push(char::from(other));
                    }
                }
                i += 2;
            }
            _ => {
                result.push(char::from(bytes[i]));
                i += 1;
            }
        }
    }
    Err(stylesheet_error(input, pos, "unterminated quoted string"))
}

/// Parse an unquoted value token: `[A-Za-z0-9_.-]+` per §8.2 grammar.
///
/// Unquoted values are identifier-like tokens (model IDs, provider keys,
/// enum keywords). Values with spaces or special characters must be quoted.
fn parse_unquoted_value(input: &str, pos: usize) -> AttractorResult<(String, usize)> {
    let bytes = input.as_bytes();
    let mut end = pos;
    // Value is an identifier-like token: alphanumeric, underscore, hyphen, dot
    while end < bytes.len()
        && (bytes[end].is_ascii_alphanumeric()
            || bytes[end] == b'_'
            || bytes[end] == b'-'
            || bytes[end] == b'.'
            || bytes[end] == b'/')
    {
        end += 1;
    }
    let value = &input[pos..end];
    if value.is_empty() {
        return Err(stylesheet_error(input, pos, "expected property value"));
    }
    Ok((value.to_string(), end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_stylesheet() -> AttractorResult<()> {
        let s = parse_stylesheet("")?;
        assert!(s.rules.is_empty());
        Ok(())
    }

    #[test]
    fn parse_universal_rule() -> AttractorResult<()> {
        let s = parse_stylesheet("* { llm_model: claude-sonnet-4-5; }")?;
        assert_eq!(s.rules.len(), 1);
        assert_eq!(s.rules[0].selector, Selector::Universal);
        assert_eq!(s.rules[0].declarations.len(), 1);
        assert_eq!(s.rules[0].declarations[0].property, "llm_model");
        assert_eq!(s.rules[0].declarations[0].value, "claude-sonnet-4-5");
        Ok(())
    }

    #[test]
    fn parse_class_rule() -> AttractorResult<()> {
        let s = parse_stylesheet(".code { llm_model: claude-opus-4-6; llm_provider: anthropic; }")?;
        assert_eq!(s.rules.len(), 1);
        assert_eq!(s.rules[0].selector, Selector::Class("code".into()));
        assert_eq!(s.rules[0].declarations.len(), 2);
        Ok(())
    }

    #[test]
    fn parse_id_rule() -> AttractorResult<()> {
        let s = parse_stylesheet("#review { reasoning_effort: high; }")?;
        assert_eq!(s.rules.len(), 1);
        assert_eq!(s.rules[0].selector, Selector::Id("review".into()));
        assert_eq!(s.rules[0].declarations[0].property, "reasoning_effort");
        assert_eq!(s.rules[0].declarations[0].value, "high");
        Ok(())
    }

    #[test]
    fn parse_multiple_rules() -> AttractorResult<()> {
        let s = parse_stylesheet(
            "* { llm_model: gpt-4; } .code { llm_model: claude-opus-4-6; } #x { reasoning_effort: low; }",
        )?;
        assert_eq!(s.rules.len(), 3);
        assert_eq!(s.rules[0].selector, Selector::Universal);
        assert_eq!(s.rules[1].selector, Selector::Class("code".into()));
        assert_eq!(s.rules[2].selector, Selector::Id("x".into()));
        Ok(())
    }

    #[test]
    fn specificity_ordering() {
        assert!(Selector::Universal.specificity() < Selector::Class("x".into()).specificity());
        assert!(Selector::Class("x".into()).specificity() < Selector::Id("x".into()).specificity());
    }

    #[test]
    fn parse_error_missing_brace() {
        let result = parse_stylesheet("* llm_model: foo; }");
        assert!(result.is_err());
    }

    #[test]
    fn parse_error_empty_selector() {
        let result = parse_stylesheet("{ llm_model: foo; }");
        assert!(result.is_err());
    }

    #[test]
    fn parse_trailing_semicolons_optional() -> AttractorResult<()> {
        let s = parse_stylesheet("* { llm_model: foo }")?;
        assert_eq!(s.rules[0].declarations[0].value, "foo");
        Ok(())
    }

    #[test]
    fn parse_rejects_invalid_reasoning_effort() {
        let result = parse_stylesheet("* { reasoning_effort: turbo; }");
        assert!(result.is_err());
        let err = format!(
            "{}",
            result.expect_err("should reject invalid reasoning_effort")
        );
        assert!(err.contains("invalid reasoning_effort value"), "{err}");
    }

    #[test]
    fn parse_rejects_value_with_spaces() {
        let result = parse_stylesheet("* { llm_model: has spaces; }");
        assert!(result.is_err());
    }

    #[test]
    fn parse_accepts_quoted_value_with_spaces() -> AttractorResult<()> {
        let s = parse_stylesheet(r#"* { llm_model: "model with spaces"; }"#)?;
        assert_eq!(s.rules[0].declarations[0].value, "model with spaces");
        Ok(())
    }

    #[test]
    fn parse_accepts_model_id_with_dots_and_slashes() -> AttractorResult<()> {
        let s = parse_stylesheet("* { llm_model: org/model-v2.1; }")?;
        assert_eq!(s.rules[0].declarations[0].value, "org/model-v2.1");
        Ok(())
    }

    #[test]
    fn parse_multiline_stylesheet() -> AttractorResult<()> {
        let input = r"
            * {
                llm_model: claude-sonnet-4-5;
                llm_provider: anthropic;
            }
            .code {
                llm_model: claude-opus-4-6;
            }
        ";
        let s = parse_stylesheet(input)?;
        assert_eq!(s.rules.len(), 2);
        assert_eq!(s.rules[0].declarations.len(), 2);
        assert_eq!(s.rules[1].declarations.len(), 1);
        Ok(())
    }
}
