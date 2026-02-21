//! Model stylesheet application (§8).
//!
//! Applies CSS-like rules to pipeline nodes, setting default LLM
//! configuration (`llm_model`, `llm_provider`, `reasoning_effort`).
//!
//! # Application order (§8.5)
//!
//! 1. Explicit node attribute (highest precedence)
//! 2. Stylesheet rule by specificity (ID > class > universal)
//! 3. Graph-level default attribute
//! 4. Handler/system default (lowest precedence)
//!
//! Within equal specificity, later rules override earlier ones.
//!
// TODO(spec-ambiguity): §11.10 mentions "universal < shape < class < ID" specificity
// but §8.2–8.3 grammar only defines 3 selector types (*, .class, #id) with no shape
// selector. Using 3-level specificity per grammar. (spec: §11.10 vs §8.2–8.3)

use crate::error::AttractorResult;
use crate::graph::{AttrValue, Graph, Node};
use crate::stylesheet_parser::{ALLOWED_PROPERTIES, ParsedStylesheet, Selector, parse_stylesheet};

/// Apply a parsed stylesheet to a graph, setting node attributes per §8.5.
///
/// For each node, iterates through all matching rules in specificity order
/// and sets properties that are not already explicitly set on the node.
///
/// # Errors
///
/// Returns an error if the stylesheet cannot be applied (currently infallible).
pub fn apply_stylesheet(graph: &mut Graph, stylesheet: &ParsedStylesheet) -> AttractorResult<()> {
    // Collect node classes for matching. Each node may have classes from:
    // 1. Explicit "class" attribute (comma-separated)
    // 2. Subgraph-derived class (already merged into node attrs by parser)
    let node_ids: Vec<String> = graph.nodes.keys().cloned().collect();

    for node_id in &node_ids {
        // Resolve each property: find the highest-specificity rule that declares it
        for &prop in ALLOWED_PROPERTIES {
            // Skip if the node already has this attribute explicitly set
            if graph
                .nodes
                .get(node_id)
                .is_some_and(|n| n.get_str_attr(prop).is_some())
            {
                continue;
            }

            // Find the best matching rule for this property
            if let Some(value) = resolve_property(graph, node_id, stylesheet, prop)
                && let Some(node) = graph.get_node_mut(node_id)
            {
                node.attrs.insert(prop.to_string(), AttrValue::from(value));
            }
        }
    }

    Ok(())
}

/// Resolve a single property for a node from the stylesheet.
///
/// Returns the value from the highest-specificity matching rule.
/// Among rules of equal specificity, the last one in source order wins.
fn resolve_property(
    graph: &Graph,
    node_id: &str,
    stylesheet: &ParsedStylesheet,
    property: &str,
) -> Option<String> {
    let node = graph.nodes.get(node_id)?;

    let mut best_value: Option<String> = None;
    let mut best_specificity: Option<u8> = None;

    for rule in &stylesheet.rules {
        if !selector_matches(&rule.selector, node) {
            continue;
        }

        for decl in &rule.declarations {
            if decl.property == property {
                let spec = rule.selector.specificity();
                // Later rules of equal or higher specificity win
                if best_specificity.is_none_or(|bs| spec >= bs) {
                    best_value = Some(decl.value.clone());
                    best_specificity = Some(spec);
                }
            }
        }
    }

    // If no stylesheet match, try graph-level default
    if best_value.is_none()
        && let Some(graph_val) = graph.get_graph_attr(property).and_then(|v| v.as_str())
    {
        return Some(graph_val.to_string());
    }

    best_value
}

/// Check whether a selector matches a given node.
fn selector_matches(selector: &Selector, node: &Node) -> bool {
    match selector {
        Selector::Universal => true,
        Selector::Id(id) => node.id == *id,
        Selector::Class(class_name) => node_has_class(node, class_name),
    }
}

/// Check whether a node has a given class.
///
/// Classes are stored as a comma-separated string in the `class` attribute.
fn node_has_class(node: &Node, class_name: &str) -> bool {
    node.get_str_attr("class")
        .is_some_and(|classes| classes.split(',').any(|c| c.trim() == class_name))
}

/// Parse a stylesheet string from a graph attribute and apply it.
///
/// Reads the `model_stylesheet` graph attribute, parses it, and applies
/// the resulting rules to all nodes. Even if the stylesheet is empty or
/// absent, graph-level default attributes are still applied per §8.5.
///
/// # Errors
///
/// Returns an error if the stylesheet cannot be parsed.
pub fn parse_and_apply_stylesheet(graph: &mut Graph) -> AttractorResult<()> {
    // Check for non-string model_stylesheet (e.g. numeric) — treat as absent
    // with an error if the attribute exists but is not a string.
    let stylesheet_str = match graph.get_graph_attr("model_stylesheet") {
        Some(v) => match v.as_str() {
            Some(s) => s.to_string(),
            None => {
                return Err(crate::error::AttractorError::InvalidPipeline {
                    reason: format!("model_stylesheet must be a string, got {}", v.type_name()),
                });
            }
        },
        None => String::new(),
    };

    let parsed = if stylesheet_str.is_empty() {
        ParsedStylesheet { rules: Vec::new() }
    } else {
        parse_stylesheet(&stylesheet_str)?
    };

    // Always apply — even an empty stylesheet triggers graph-level default
    // fallback per §8.5 resolution order.
    apply_stylesheet(graph, &parsed)
}
