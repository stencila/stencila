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
                id.as_deref(),
                label.as_deref(),
                Some(programming_language),
                execute_auto.clone(),
                execute_required.as_ref().map(|value| value.as_ref()),
                execute_status.clone(),
            ),
            CodeExecutableCodeDependencies::Parameter(Parameter {
                id,
                name,
                execute_required,
                ..
            }) => (
                "Parameter",
                id.as_deref(),
                Some(name),
                None,
                None,
                execute_required.as_ref().map(|value| value.as_ref()),
                None,
            ),
            CodeExecutableCodeDependencies::File(File { path, .. }) => {
                ("File", None, Some(path), None, None, None, None)
            }
        };
        elem(
            "stencila-code-dependency",
            &[
                attr("node-kind", node_kind),
                id.map(|value| attr("node-id", value)).unwrap_or_default(),
                label.map(|value| attr("label", value)).unwrap_or_default(),
                programming_language
                    .map(|value| attr("programming-language", value))
                    .unwrap_or_default(),
                execute_auto
                    .map(|value| attr("execute-auto", value.as_ref()))
                    .unwrap_or_default(),
                execute_required
                    .map(|value| attr("execute-required", value))
                    .unwrap_or_default(),
                execute_status
                    .map(|value| attr("execute-status", value.as_ref()))
                    .unwrap_or_default(),
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
                id.as_deref(),
                label.as_deref(),
                Some(programming_language),
                execute_auto.as_ref().map(|value| value.as_ref()),
                execute_required.as_ref().map(|value| value.as_ref()),
                execute_status.as_ref().map(|value| value.as_ref()),
            ),
            CodeExecutableCodeDependents::CodeExpression(CodeExpression {
                id,
                programming_language,
                execute_required,
                execute_status,
                ..
            }) => (
                "CodeExpression",
                id.as_deref(),
                None,
                Some(programming_language),
                None,
                execute_required.as_ref().map(|value| value.as_ref()),
                execute_status.as_ref().map(|value| value.as_ref()),
            ),
            CodeExecutableCodeDependents::File(File { path, .. }) => {
                ("File", None, Some(path), None, None, None, None)
            }
        };
        elem(
            "stencila-code-dependency",
            &[
                attr("node-kind", node_kind),
                id.map(|value| attr("node-id", value)).unwrap_or_default(),
                label.map(|value| attr("label", value)).unwrap_or_default(),
                programming_language
                    .map(|value| attr("programming-language", value))
                    .unwrap_or_default(),
                execute_auto
                    .map(|value| attr("execute-auto", value))
                    .unwrap_or_default(),
                execute_required
                    .map(|value| attr("execute-required", value))
                    .unwrap_or_default(),
                execute_status
                    .map(|value| attr("execute-status", value))
                    .unwrap_or_default(),
            ],
            "",
        )
    }
}
