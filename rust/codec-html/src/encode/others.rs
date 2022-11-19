use stencila_schema::*;

use super::{attr, elem, EncodeContext, ToHtml};

/// Encode a dependency of an executable code node
///
/// Note that for this type, and for `ExecutionDependents`, the node being
/// encoded is a _partial_ copy of a node. So any properties encoded here must be copied
/// across in the `node-execute::compile` module.
impl ToHtml for ExecutionDependency {
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        String::new()
        /*
        let (
            node_kind,
            id,
            label,
            programming_language,
            execution_auto,
            execution_required,
            execution_status,
        ) = match self {
            ExecutionDependency::CodeChunk(CodeChunk {
                id,
                label,
                programming_language,
                execution_auto,
                execution_required,
                execution_status,
                ..
            }) => (
                "CodeChunk",
                id.as_deref(),
                label.as_deref(),
                Some(programming_language),
                execution_auto.clone(),
                execution_required.as_ref().map(|value| value.as_ref()),
                execution_status.clone(),
            ),
            ExecutionDependency::Parameter(Parameter {
                id,
                name,
                execution_required,
                ..
            })
            | ExecutionDependency::Button(Button {
                id,
                name,
                execution_required,
                ..
            }) => (
                "Parameter",
                id.as_deref(),
                Some(name),
                None,
                None,
                execution_required.as_ref().map(|value| value.as_ref()),
                None,
            ),
            ExecutionDependency::File(File { path, .. }) => {
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
                execution_auto
                    .map(|value| attr("execution-auto", value.as_ref()))
                    .unwrap_or_default(),
                execution_required
                    .map(|value| attr("execution-required", value))
                    .unwrap_or_default(),
                execution_status
                    .map(|value| attr("execution-status", value.as_ref()))
                    .unwrap_or_default(),
            ],
            "",
        )
        */
    }
}

/// Encode a dependent of an executable code node
impl ToHtml for ExecutionDependent {
    fn to_html(&self, _context: &mut EncodeContext) -> String {
        String::new()
        /*
        let (
            node_kind,
            id,
            label,
            programming_language,
            execution_auto,
            execution_required,
            execution_status,
        ) = match self {
            ExecutionDependent::Call(Call {
                id,
                source,
                execution_auto,
                execution_required,
                execution_status,
                ..
            }) => (
                "Call",
                id.as_deref(),
                Some(source),
                None,
                execution_auto.as_ref().map(|value| value.as_ref()),
                execution_required.as_ref().map(|value| value.as_ref()),
                execution_status.as_ref().map(|value| value.as_ref()),
            ),

            ExecutionDependent::CodeChunk(CodeChunk {
                id,
                label,
                programming_language,
                execution_auto,
                execution_required,
                execution_status,
                ..
            }) => (
                "CodeChunk",
                id.as_deref(),
                label.as_deref(),
                Some(programming_language),
                execution_auto.as_ref().map(|value| value.as_ref()),
                execution_required.as_ref().map(|value| value.as_ref()),
                execution_status.as_ref().map(|value| value.as_ref()),
            ),

            ExecutionDependent::CodeExpression(CodeExpression {
                id,
                programming_language,
                execution_required,
                execution_status,
                ..
            }) => (
                "CodeExpression",
                id.as_deref(),
                None,
                Some(programming_language),
                None,
                execution_required.as_ref().map(|value| value.as_ref()),
                execution_status.as_ref().map(|value| value.as_ref()),
            ),

            ExecutionDependent::Division(Division {
                id,
                programming_language,
                execution_auto,
                execution_required,
                execution_status,
                ..
            }) => (
                "Division",
                id.as_deref(),
                None,
                Some(programming_language),
                execution_auto.as_ref().map(|value| value.as_ref()),
                execution_required.as_ref().map(|value| value.as_ref()),
                execution_status.as_ref().map(|value| value.as_ref()),
            ),

            ExecutionDependent::Span(Span {
                id,
                programming_language,
                execution_auto,
                execution_required,
                execution_status,
                ..
            }) => (
                "Span",
                id.as_deref(),
                None,
                Some(programming_language),
                execution_auto.as_ref().map(|value| value.as_ref()),
                execution_required.as_ref().map(|value| value.as_ref()),
                execution_status.as_ref().map(|value| value.as_ref()),
            ),

            ExecutionDependent::File(File { path, .. }) => {
                ("File", None, Some(path), None, None, None, None)
            }

            ExecutionDependent::If(_) | ExecutionDependent::For(_) => todo!(),
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
                execution_auto
                    .map(|value| attr("execution-auto", value))
                    .unwrap_or_default(),
                execution_required
                    .map(|value| attr("execution-required", value))
                    .unwrap_or_default(),
                execution_status
                    .map(|value| attr("execution-status", value))
                    .unwrap_or_default(),
            ],
            "",
        )
        */
    }
}
