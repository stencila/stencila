//! Parsing utilities for parameter values and case strings

use indexmap::IndexMap;

use super::{Result, SpreadError};

/// A parameter value that may hold multiple values (for grid/zip modes) or a scalar.
///
/// # Examples
///
/// ```
/// use stencila_spread::ParameterValues;
///
/// // Multi-valued parameter (comma-separated)
/// let values = ParameterValues::parse("north,south,east");
/// assert_eq!(values.len(), 3);
/// assert!(values.is_multi());
///
/// // Single value
/// let single = ParameterValues::scalar("2025".to_string());
/// assert_eq!(single.len(), 1);
/// assert!(!single.is_multi());
/// ```
#[derive(Debug, Clone)]
pub struct ParameterValues(pub Vec<String>);

impl ParameterValues {
    /// Parse a raw value string into parameter values.
    ///
    /// In spread modes, commas separate values unless:
    /// - Escaped with `\,`
    /// - Within single or double quotes
    ///
    /// # Examples
    ///
    /// ```
    /// use stencila_spread::ParameterValues;
    ///
    /// let values = ParameterValues::parse("a,b,c");
    /// assert_eq!(values.values(), &["a", "b", "c"]);
    ///
    /// // Escaped comma
    /// let escaped = ParameterValues::parse(r"Smith\, John,Doe");
    /// assert_eq!(escaped.values(), &["Smith, John", "Doe"]);
    ///
    /// // Quoted values
    /// let quoted = ParameterValues::parse("'a,b',c");
    /// assert_eq!(quoted.values(), &["a,b", "c"]);
    /// ```
    #[must_use]
    pub fn parse(value: &str) -> Self {
        let values = split_comma_values(value);
        ParameterValues(values)
    }

    /// Create a scalar (single value) parameter.
    #[must_use]
    pub fn scalar(value: String) -> Self {
        ParameterValues(vec![value])
    }

    /// Check if this is a multi-valued parameter (length > 1).
    #[must_use]
    pub fn is_multi(&self) -> bool {
        self.0.len() > 1
    }

    /// Get the number of values.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the values as a slice.
    #[must_use]
    pub fn values(&self) -> &[String] {
        &self.0
    }

    /// Get the first value, or an empty string if empty.
    #[must_use]
    pub fn first_or_default(&self) -> String {
        self.0.first().cloned().unwrap_or_default()
    }
}

/// Split a value string on commas, respecting escape sequences and quotes.
///
/// - `\,` is treated as a literal comma
/// - Single or double quotes suppress comma splitting within them
fn split_comma_values(input: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while let Some(c) = chars.next() {
        match c {
            '\\' if chars.peek() == Some(&',') => {
                // Escaped comma - consume and add literal comma
                chars.next();
                current.push(',');
            }
            '\\' => {
                current.push(c);
            }
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
                // Don't include the quote in the value
            }
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
                // Don't include the quote in the value
            }
            ',' if !in_single_quote && !in_double_quote => {
                values.push(current.trim().to_string());
                current = String::new();
            }
            _ => {
                current.push(c);
            }
        }
    }

    // Don't forget the last value
    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() || !values.is_empty() {
        values.push(trimmed);
    }

    values
}

/// Parameters parsed from CLI, preserving insertion order.
///
/// This is a newtype wrapper around `IndexMap` to provide type safety
/// and prevent accidental mixing with other map types.
#[derive(Debug, Clone, Default)]
pub struct Parameters(IndexMap<String, ParameterValues>);

impl Parameters {
    /// Create a new empty parameter map.
    #[must_use]
    pub fn new() -> Self {
        Parameters(IndexMap::new())
    }

    /// Insert a parameter.
    pub fn insert(&mut self, name: String, values: ParameterValues) {
        self.0.insert(name, values);
    }

    /// Get a parameter by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&ParameterValues> {
        self.0.get(name)
    }

    /// Check if a parameter exists.
    #[must_use]
    pub fn contains_key(&self, name: &str) -> bool {
        self.0.contains_key(name)
    }

    /// Iterate over parameters.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &ParameterValues)> {
        self.0.iter()
    }

    /// Check if empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// One explicit case parameter set (raw scalars).
///
/// This is a newtype wrapper around `IndexMap` to provide type safety.
#[derive(Debug, Clone, Default)]
pub struct CaseParameters(IndexMap<String, String>);

impl CaseParameters {
    /// Create a new empty case parameter set.
    #[must_use]
    pub fn new() -> Self {
        CaseParameters(IndexMap::new())
    }

    /// Insert a parameter.
    pub fn insert(&mut self, name: String, value: String) {
        self.0.insert(name, value);
    }

    /// Get a parameter by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&String> {
        self.0.get(name)
    }

    /// Iterate over parameters.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &String)> {
        self.0.iter()
    }

    /// Get parameter names.
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.0.keys()
    }
}

/// Parse a case string like `"region=north species=ABC"` into parameter pairs.
///
/// # Errors
///
/// Returns an error if:
/// - `SpreadError::InvalidCaseSyntax`: Invalid syntax (missing `=` in a part)
/// - `SpreadError::DuplicateCaseParameter`: Duplicate parameter name
///
/// # Examples
///
/// ```
/// use stencila_spread::parse_case;
///
/// let case = parse_case("region=north species=ABC")?;
/// assert_eq!(case.get("region"), Some(&"north".to_string()));
/// assert_eq!(case.get("species"), Some(&"ABC".to_string()));
/// # Ok::<(), stencila_spread::SpreadError>(())
/// ```
pub fn parse_case(case_str: &str) -> Result<CaseParameters> {
    let mut params = CaseParameters::new();

    // Split on whitespace and parse key=value pairs
    for part in case_str.split_whitespace() {
        if let Some((key, value)) = part.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            if key.is_empty() {
                return Err(SpreadError::InvalidCaseSyntax {
                    message: format!("empty key in '{part}'"),
                });
            }

            if params.0.contains_key(key) {
                return Err(SpreadError::DuplicateCaseParameter {
                    name: key.to_string(),
                });
            }

            params.insert(key.to_string(), value.to_string());
        } else {
            return Err(SpreadError::InvalidCaseSyntax {
                message: format!("expected 'key=value' pairs, found '{part}'"),
            });
        }
    }

    Ok(params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_comma_values_simple() {
        assert_eq!(split_comma_values("a,b,c"), vec!["a", "b", "c"]);
        assert_eq!(split_comma_values("north,south"), vec!["north", "south"]);
    }

    #[test]
    fn test_split_comma_values_escaped() {
        assert_eq!(split_comma_values(r"a\,b,c"), vec!["a,b", "c"]);
        assert_eq!(split_comma_values(r"Smith\, John"), vec!["Smith, John"]);
    }

    #[test]
    fn test_split_comma_values_quoted() {
        assert_eq!(split_comma_values("'a,b',c"), vec!["a,b", "c"]);
        assert_eq!(split_comma_values("\"1,000\",2000"), vec!["1,000", "2000"]);
    }

    #[test]
    fn test_split_comma_values_single() {
        assert_eq!(split_comma_values("single"), vec!["single"]);
        assert_eq!(split_comma_values("2025"), vec!["2025"]);
    }

    #[test]
    fn test_parameter_values_is_multi() {
        assert!(ParameterValues::parse("a,b").is_multi());
        assert!(!ParameterValues::parse("single").is_multi());
    }

    #[test]
    fn test_parse_case() -> Result<()> {
        let case = parse_case("region=north species=ABC")?;
        assert_eq!(case.get("region"), Some(&"north".to_string()));
        assert_eq!(case.get("species"), Some(&"ABC".to_string()));
        Ok(())
    }

    #[test]
    fn test_parse_case_error_duplicate() {
        let result = parse_case("region=north region=south");
        assert!(matches!(
            result,
            Err(SpreadError::DuplicateCaseParameter { name }) if name == "region"
        ));
    }
}
