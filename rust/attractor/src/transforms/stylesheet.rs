//! Stylesheet transform (§8.5, §9.2).
//!
//! Applies the `model_stylesheet` graph attribute to pipeline nodes
//! as a pre-execution transform. This runs after variable expansion
//! but before validation, so stylesheet-applied attributes are visible
//! to lint rules.

use crate::error::AttractorResult;
use crate::graph::Graph;
use crate::stylesheet::parse_and_apply_stylesheet;
use crate::transform::Transform;

/// Applies the model stylesheet to graph nodes.
///
/// Reads the `model_stylesheet` graph attribute, parses the CSS-like
/// rules, and sets `llm_model`, `llm_provider`, and `reasoning_effort`
/// attributes on matching nodes (unless already explicitly set).
pub struct StylesheetTransform;

impl Transform for StylesheetTransform {
    fn name(&self) -> &'static str {
        "stylesheet"
    }

    fn apply(&self, graph: &mut Graph) -> AttractorResult<()> {
        parse_and_apply_stylesheet(graph)
    }
}
