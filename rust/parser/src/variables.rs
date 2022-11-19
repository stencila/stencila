use std::{collections::HashMap, path::Path};

use common::{
    once_cell::sync::Lazy,
    regex::{Captures, Regex},
};
use stencila_schema::{CodeError, ExecutionDependency};

use crate::utils::uses_variable;

/// Regex for detecting variable interpolations within code
///
/// Allows for $var and ${var} patterns
pub static VAR_INTERP_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?:\$([a-zA-Z][a-zA-Z_0-9]*)\b)|(?:\$\{\s*([a-zA-Z][a-zA-Z_0-9]*)\s*\})")
        .expect("Unable to create regex")
});

/// Parse variable interpolations in code into a vector of relations
///
/// Used by parsers to define the relations between some code and other resources.
pub fn parse_var_interps(code: &str, path: Option<&Path>) -> Vec<ExecutionDependency> {
    VAR_INTERP_REGEX
        .captures_iter(code)
        .map(|captures| {
            let name = captures
                .get(1)
                .or_else(|| captures.get(2))
                .expect("Should always have one group");
            uses_variable(
                name.as_str(),
                path,
                None,
                Some([0, name.start(), 0, name.end()]),
            )
        })
        .collect()
}

/// Perform variable interpolations in code using a map of symbols to strings
///
/// Used by kernels before executing code to perform interpolation. Returns
/// an error message for each variable that is not in the map.
pub fn perform_var_interps(
    code: &str,
    symbols: &HashMap<String, String>,
) -> (String, Vec<CodeError>) {
    let mut messages = Vec::new();
    let interpolated = VAR_INTERP_REGEX.replace_all(code, |captures: &Captures| {
        let symbol = captures
            .get(1)
            .or_else(|| captures.get(2))
            .expect("Should always have one group")
            .as_str();
        match symbols.get(symbol) {
            Some(value) => value.to_owned(),
            None => {
                messages.push(CodeError {
                    error_type: Some(Box::new("UnknownSymbol".to_string())),
                    error_message: format!("Symbol `{}` is not available", symbol),
                    ..Default::default()
                });
                captures[0].to_string()
            }
        }
    });
    (interpolated.to_string(), messages)
}
