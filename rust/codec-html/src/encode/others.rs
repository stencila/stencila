use stencila_schema::*;

use super::{attr, elem, EncodeContext, ToHtml};

/// Encode a dependency of an executable code node
///
/// Note that for this type, and for `ExecutableCodeDependents`, the node being
/// encoded is a _partial_ copy of a node. So any properties encoded here must be copied
/// across in the `node-execute::compile` module.
impl ToHtml for ExecutableCodeDependencies {
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
            ExecutableCodeDependencies::CodeChunk(CodeChunk {
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
            ExecutableCodeDependencies::Parameter(Parameter {
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
            ExecutableCodeDependencies::File(File { path, .. }) => {
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
impl ToHtml for ExecutableCodeDependents {
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
            ExecutableCodeDependents::Call(Call {
                id,
                source,
                execute_auto,
                execute_required,
                execute_status,
                ..
            }) => (
                "Call",
                id.as_deref(),
                Some(source),
                None,
                execute_auto.as_ref().map(|value| value.as_ref()),
                execute_required.as_ref().map(|value| value.as_ref()),
                execute_status.as_ref().map(|value| value.as_ref()),
            ),

            ExecutableCodeDependents::CodeChunk(CodeChunk {
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

            ExecutableCodeDependents::CodeExpression(CodeExpression {
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

            ExecutableCodeDependents::File(File { path, .. }) => {
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
