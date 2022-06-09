use codec::common::{
    chrono::{DateTime, Datelike},
    tracing,
};
use stencila_schema::*;

use super::{attr, elem, EncodeContext, ToHtml};

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
///
/// Note that for this type, and for `CodeExecutableCodeDependents`, the node being
/// encoded is a _partial_ copy of a node. So any properties encoded here must be copied
/// across in the `node-execute::compile` module.
impl ToHtml for CodeExecutableCodeDependencies {
    fn to_html(&self, _context: &EncodeContext) -> String {
        let (
            node_kind,
            id,
            label,
            programming_language,
            execute_auto,
            execute_required,
            execute_status,
        ) = match self {
            CodeExecutableCodeDependencies::CodeChunk(CodeChunk {
                id,
                label,
                programming_language,
                execute_auto,
                execute_required,
                execute_status,
                ..
            }) => (
                "CodeChunk",
                id,
                label.as_ref().map(|label| label.as_str()),
                Some(programming_language),
                execute_auto
                    .as_ref()
                    .map_or("Needed", |value| value.as_ref()),
                execute_required,
                execute_status,
            ),
            CodeExecutableCodeDependencies::Parameter(Parameter { id, name, .. }) => (
                "Parameter",
                id,
                Some(name.as_str()),
                None,
                "Needed",
                &None,
                &None,
            ),
        };
        elem(
            "stencila-code-dependency",
            &[
                attr("node-kind", node_kind),
                id.as_ref()
                    .map_or("".to_string(), |value| attr("node-id", value.as_str())),
                label
                    .as_ref()
                    .map_or("".to_string(), |value| attr("label", value)),
                programming_language
                    .as_ref()
                    .map_or("".to_string(), |value| attr("programming-language", value)),
                attr("execute-auto", execute_auto),
                execute_required.as_ref().map_or("".to_string(), |value| {
                    attr("execute-required", value.as_ref())
                }),
                execute_status.as_ref().map_or("".to_string(), |value| {
                    attr("execute-status", value.as_ref())
                }),
            ],
            "",
        )
    }
}

/// Encode a dependent of an executable code node
impl ToHtml for CodeExecutableCodeDependents {
    fn to_html(&self, _context: &EncodeContext) -> String {
        let (
            node_kind,
            id,
            label,
            programming_language,
            execute_auto,
            execute_required,
            execute_status,
        ) = match self {
            CodeExecutableCodeDependents::CodeChunk(CodeChunk {
                id,
                label,
                programming_language,
                execute_auto,
                execute_required,
                execute_status,
                ..
            }) => (
                "CodeChunk",
                id,
                label,
                programming_language,
                execute_auto
                    .as_ref()
                    .map_or("Needed", |value| value.as_ref()),
                execute_required,
                execute_status,
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
                &None,
                programming_language,
                "Needed",
                execute_required,
                execute_status,
            ),
        };
        elem(
            "stencila-code-dependency",
            &[
                attr("node-kind", node_kind),
                id.as_ref()
                    .map_or("".to_string(), |value| attr("node-id", value.as_str())),
                label
                    .as_ref()
                    .map_or("".to_string(), |value| attr("label", value.as_ref())),
                attr("programming-language", programming_language),
                attr("execute-auto", execute_auto),
                execute_required.as_ref().map_or("".to_string(), |value| {
                    attr("execute-required", value.as_ref())
                }),
                execute_status.as_ref().map_or("".to_string(), |value| {
                    attr("execute-status", value.as_ref())
                }),
            ],
            "",
        )
    }
}
