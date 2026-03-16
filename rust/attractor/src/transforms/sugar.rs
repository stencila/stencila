//! Node sugar transform.
//!
//! Normalizes shorthand node conventions into canonical attributes
//! so that users do not need to remember shape names.
//!
//! This transform runs before variable expansion and stylesheet
//! application, rewriting sugar forms into the canonical DOT attributes
//! that the rest of the engine expects.
//!
//! ## Resolution order
//!
//! For each node without an explicit `shape`, the transform applies the
//! first matching rule:
//!
//! 1. **Property shortcuts** — `ask`, `shell`, `branch`, `workflow`
//!    imply a specific handler type. All sugar keys are always removed
//!    regardless of which one wins, so they never leak into the graph.
//! 2. **`interview`** — if the `interview` attribute is present, the node
//!    is a human gate (`hexagon`), supporting multi-question interviews.
//! 3. **`fan_out`** — if present, the node is a parallel fan-out
//!    (`component`). Unlike other sugar keys, `fan_out` is **not** drained
//!    because the `ParallelHandler` reads it at runtime.
//! 4. **`prompt` / `agent`** — if either is present the node is an LLM
//!    task (`box`), overriding prefix-based ID inference. Reserved
//!    structural IDs (`Start`/`End`/`Fail`) are exempt — they always
//!    get their structural shape because `Node::handler_type()` treats
//!    them specially even when shape is `box`.
//! 5. **Node ID** — exact or prefix match maps to a shape (only
//!    `FanOut`/`FanIn` prefixes remain; other node types use property
//!    shortcuts instead).
//!
//! An explicit `shape` attribute always wins over all of the above.
//!
//! ## Property shortcuts
//!
//! | Sugar attribute        | Canonical form                               |
//! |------------------------|----------------------------------------------|
//! | `ask="..."`            | `shape=hexagon`, `label="..."`               |
//! | `shell="…"`            | `shape=parallelogram`, `shell_command="…"`   |
//! | `branch="..."`         | `shape=diamond`, `label="..."`               |
//! | `workflow="..."`       | `type="workflow"`                            |
//!
//! ## Node ID aliases
//!
//! | Node ID (exact)          | Implied shape    | Handler          |
//! |--------------------------|------------------|------------------|
//! | `Start`, `start`         | `Mdiamond`       | start            |
//! | `End`, `end`, etc.       | `Msquare`        | exit             |
//! | `Fail`, `fail`           | `invtriangle`    | fail             |
//!
//! | Node ID (prefix)         | Implied shape    | Handler          |
//! |--------------------------|------------------|------------------|
//! | `FanOut*`, `Fanout*`     | `component`      | parallel fan-out |
//! | `FanIn*`, `Fanin*`       | `tripleoctagon`  | parallel fan-in  |

use crate::error::AttractorResult;
use crate::graph::{AttrValue, Graph};
use crate::transform::Transform;

pub struct NodeSugarTransform;

/// Infer the shape for a structural node ID, returning `Some` only for
/// reserved start/exit/fail IDs.
fn structural_shape(id: &str) -> Option<&'static str> {
    if Graph::START_IDS.contains(&id) {
        Some(Graph::START_SHAPE)
    } else if Graph::EXIT_IDS.contains(&id) {
        Some(Graph::EXIT_SHAPE)
    } else if Graph::FAIL_IDS.contains(&id) {
        Some(Graph::FAIL_SHAPE)
    } else {
        None
    }
}

/// Infer the shape from a node's ID using both exact and prefix matching.
fn infer_shape_from_id(id: &str) -> Option<&'static str> {
    structural_shape(id).or_else(|| {
        if id.starts_with("FanOut") || id.starts_with("Fanout") {
            Some(Graph::PARALLEL_SHAPE)
        } else if id.starts_with("FanIn") || id.starts_with("Fanin") {
            Some(Graph::PARALLEL_FAN_IN_SHAPE)
        } else {
            None
        }
    })
}

impl Transform for NodeSugarTransform {
    fn name(&self) -> &'static str {
        "node_sugar"
    }

    fn apply(&self, graph: &mut Graph) -> AttractorResult<()> {
        for node in graph.nodes.values_mut() {
            let has_shape = node.attrs.contains_key("shape");

            // Drain all sugar keys up front so none leak into the graph.
            // Precedence: ask > shell > branch (first present wins).
            let ask_val = node.attrs.shift_remove("ask");
            let shell_val = node.attrs.shift_remove("shell");
            let branch_val = node.attrs.shift_remove("branch");
            let workflow_val = node.attrs.shift_remove("workflow");

            // --- Property shortcuts (highest priority) ---

            // `ask` implies human gate
            if let Some(val) = ask_val {
                if !has_shape {
                    node.attrs.insert(
                        "shape".to_string(),
                        AttrValue::String(Graph::HUMAN_SHAPE.into()),
                    );
                }
                if !node.attrs.contains_key("label") {
                    node.attrs.insert("label".to_string(), val);
                }
                continue;
            }

            // `workflow` implies a workflow composition node
            if let Some(val) = workflow_val {
                if !node.attrs.contains_key("type") {
                    node.attrs
                        .insert("type".to_string(), AttrValue::String("workflow".into()));
                }
                node.attrs.insert("workflow".to_string(), val);
                continue;
            }

            // `shell` implies shell node
            if let Some(val) = shell_val {
                if !has_shape {
                    node.attrs.insert(
                        "shape".to_string(),
                        AttrValue::String(Graph::SHELL_SHAPE.into()),
                    );
                }
                if !node.attrs.contains_key("shell_command") {
                    node.attrs.insert("shell_command".to_string(), val);
                }
                continue;
            }

            // `branch` implies conditional node
            if let Some(val) = branch_val {
                if !has_shape {
                    node.attrs.insert(
                        "shape".to_string(),
                        AttrValue::String(Graph::CONDITIONAL_SHAPE.into()),
                    );
                }
                if !node.attrs.contains_key("label") {
                    node.attrs.insert("label".to_string(), val);
                }
                continue;
            }

            // `interview` implies human gate (multi-question interview)
            if node.attrs.contains_key("interview") && !has_shape {
                node.attrs.insert(
                    "shape".to_string(),
                    AttrValue::String(Graph::HUMAN_SHAPE.into()),
                );
                continue;
            }

            // `fan_out` implies parallel fan-out node (component shape).
            // Unlike other sugar keys, `fan_out` is NOT drained — it must
            // remain on the node for the `ParallelHandler` to read at runtime.
            if node.attrs.contains_key("fan_out") && !has_shape {
                node.attrs.insert(
                    "shape".to_string(),
                    AttrValue::String(Graph::PARALLEL_SHAPE.into()),
                );
                continue;
            }

            // --- Remaining inference only when no explicit shape ---
            if has_shape {
                continue;
            }

            // `prompt` or `agent` means this is an LLM node — skip prefix-
            // based ID inference so e.g. `FanOutAnalysis [prompt="..."]`
            // stays codergen. Structural IDs (Start/End/Fail) are exempt.
            if node.attrs.contains_key("prompt")
                || node.attrs.contains_key("agent")
                || node.attrs.keys().any(|k| k.starts_with("agent."))
            {
                if let Some(shape) = structural_shape(&node.id) {
                    node.attrs
                        .insert("shape".to_string(), AttrValue::String(shape.to_string()));
                }
                continue;
            }

            // --- Node ID-based shape inference ---
            if let Some(shape) = infer_shape_from_id(&node.id) {
                node.attrs
                    .insert("shape".to_string(), AttrValue::String(shape.to_string()));
            }
        }

        Ok(())
    }
}
