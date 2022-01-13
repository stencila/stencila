use super::{attr, elem, EncodeContext, ToHtml};
use chrono::{DateTime, Datelike};
use stencila_schema::*;

/// Encode a `Date` to HTML
///
/// Takes a similar approach to the encoding of `Cite` nodes in that it encodes parts
/// of the date as spans which the theme can choose to reorder and/or hide.
impl ToHtml for Date {
    fn to_html(&self, _context: &EncodeContext) -> String {
        let content = match DateTime::parse_from_rfc3339(&self.value) {
            Ok(datetime) => [
                elem("span", &[], &datetime.year().to_string()),
                elem("span", &[], &datetime.month().to_string()),
                elem("span", &[], &datetime.day().to_string()),
            ]
            .concat(),
            Err(error) => {
                tracing::warn!("While parsing date `{}`: {}", self.value, error);
                self.value.clone()
            }
        };
        elem("time", &[attr("datetime", &self.value)], &content)
    }
}

/// Encode a dependency of an executable code node
impl ToHtml for CodeExecutableCodeDependencies {
    fn to_html(&self, _context: &EncodeContext) -> String {
        let (node_type, id, execute_required, execute_status, programming_language, name) =
            match self {
                CodeExecutableCodeDependencies::CodeChunk(CodeChunk {
                    id,
                    programming_language,
                    execute_required,
                    execute_status,
                    ..
                }) => (
                    "CodeChunk",
                    id,
                    execute_required,
                    execute_status,
                    Some(programming_language),
                    None,
                ),
                CodeExecutableCodeDependencies::Parameter(Parameter { id, name, .. }) => {
                    ("Parameter", id, &None, &None, None, Some(name.as_str()))
                }
                // `CodeExpression` nodes should not assign anything so should not actually be a
                // `CodeExecutableCodeDependencies` variant; so just ignore.
                CodeExecutableCodeDependencies::CodeExpression(..) => return "".to_string(),
            };
        elem(
            "stencila-code-dependency",
            &[
                attr("node-type", node_type),
                id.as_ref()
                    .map_or("".to_string(), |value| attr("node-id", value.as_str())),
                execute_required.as_ref().map_or("".to_string(), |value| {
                    attr("execute-required", value.as_ref())
                }),
                execute_status.as_ref().map_or("".to_string(), |value| {
                    attr("execute-status", value.as_ref())
                }),
                programming_language
                    .as_ref()
                    .map_or("".to_string(), |value| attr("programming-language", value)),
                name.as_ref()
                    .map_or("".to_string(), |value| attr("name", value)),
            ],
            "",
        )
    }
}

/// Encode a dependent of an executable code node
///
/// As for dependencies, encode as a <li><a>
impl ToHtml for CodeExecutableCodeDependents {
    fn to_html(&self, _context: &EncodeContext) -> String {
        let (node_type, id, execute_required, execute_status, programming_language) = match self {
            CodeExecutableCodeDependents::CodeChunk(CodeChunk {
                id,
                programming_language,
                execute_required,
                execute_status,
                ..
            }) => (
                "CodeChunk",
                id,
                execute_required,
                execute_status,
                programming_language,
            ),
            CodeExecutableCodeDependents::CodeExpression(CodeExpression {
                id,
                programming_language,
                execute_required,
                execute_status,
                ..
            }) => (
                "CodeExpression",
                id,
                execute_required,
                execute_status,
                programming_language,
            ),
        };
        elem(
            "stencila-code-dependent",
            &[
                attr("node-type", node_type),
                id.as_ref()
                    .map_or("".to_string(), |value| attr("node-id", value.as_str())),
                execute_required.as_ref().map_or("".to_string(), |value| {
                    attr("execute-required", value.as_ref())
                }),
                execute_status.as_ref().map_or("".to_string(), |value| {
                    attr("execute-status", value.as_ref())
                }),
                attr("programming-language", programming_language),
            ],
            "",
        )
    }
}
