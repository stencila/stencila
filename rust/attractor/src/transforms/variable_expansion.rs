//! Variable expansion transform (§9.2).
//!
//! Replaces `$goal` placeholders in node `prompt` attributes with
//! the graph-level `goal` attribute value.

use crate::error::AttractorResult;
use crate::graph::{AttrValue, Graph};
use crate::transform::Transform;

/// Expands `$goal` variables in node prompt attributes.
///
/// Reads the graph-level `goal` attribute (defaulting to an empty
/// string if absent) and replaces all occurrences of `$goal` in
/// every node's `prompt` attribute.
pub struct VariableExpansionTransform;

impl Transform for VariableExpansionTransform {
    fn name(&self) -> &'static str {
        "variable_expansion"
    }

    fn apply(&self, graph: &mut Graph) -> AttractorResult<()> {
        let goal = graph
            .get_graph_attr("goal")
            .and_then(AttrValue::as_str)
            .unwrap_or("")
            .to_string();

        for node in graph.nodes.values_mut() {
            if let Some(AttrValue::String(prompt)) = node.attrs.get_mut("prompt")
                && prompt.contains("$goal")
            {
                *prompt = prompt.replace("$goal", &goal);
            }
        }

        Ok(())
    }
}
