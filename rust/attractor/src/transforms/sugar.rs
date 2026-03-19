//! Node sugar transform.
//!
//! Normalizes shorthand node conventions into canonical attributes so that
//! users do not need to remember shape names.
//!
//! This transform runs before variable expansion and stylesheet application,
//! rewriting sugar forms into the canonical DOT attributes that the rest of the
//! engine expects.
//!
//! ## Resolution order
//!
//! For each node without an explicit `shape`, the transform applies the first
//! matching rule:
//!
//! 1. **Property shortcuts** — `ask`, `shell`, `branch`, `workflow` imply a
//!    specific handler type. All sugar keys are always removed regardless of
//!    which one wins, so they never leak into the graph.
//! 2. **`interview`** — if the `interview` attribute is present, the node is a
//!    human gate (`hexagon`), supporting multi-question interviews.
//! 3. **`fan_out`** — if present, the node is a parallel fan-out (`component`).
//!    Unlike other sugar keys, `fan_out` is **not** drained because the
//!    `ParallelHandler` reads it at runtime.
//! 4. **`prompt` / `agent`** — if either is present the node is an LLM task
//!    (`box`), overriding prefix-based ID inference. Reserved structural IDs
//!    (`Start`/`End`/`Fail`) are exempt — they always get their structural
//!    shape because `Node::handler_type()` treats them specially even when
//!    shape is `box`.
//! 5. **Node ID** — exact or prefix match maps to a shape (only
//!    `FanOut`/`FanIn` prefixes remain; other node types use property shortcuts
//!    instead).
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
//! | `persist="…"`          | `fidelity="…"`, `thread_id="persist:{id}"`   |
//!
//! ## Persist sugar
//!
//! The `persist` attribute is a shorthand for setting `fidelity` and
//! `thread_id` on a node. It is always removed after processing.
//!
//! ### Value mapping
//!
//! | `persist` value              | `fidelity`         | `thread_id`             |
//! |------------------------------|--------------------|-------------------------|
//! | `"true"`, `true`, `"full"`   | `"full"`           | `"persist:{node_id}"`   |
//! | `"summary"`                  | `"summary:medium"` | `"persist:{node_id}"`   |
//! | `"gist"`                     | `"summary:low"`    | `"persist:{node_id}"`   |
//! | `"details"`                  | `"summary:high"`   | `"persist:{node_id}"`   |
//! | `"false"`, `false`, `"off"`  | *(disabled)*        | *(disabled)*            |
//!
//! If `fidelity` is already set explicitly, it is preserved and no `thread_id`
//! is auto-generated. If `thread_id` is already set explicitly, it is
//! preserved.
//!
//! > **Note:** The `compact` and `truncate` fidelity modes are not available
//! > via `persist`. To use those modes, set the `fidelity` attribute directly
//! > (e.g., `fidelity="compact"` or `fidelity="truncate"`).
//!
//! ### Examples
//!
//! Full conversation persistence on an LLM node:
//!
//! ```dot
//! Build [agent="code-engineer", persist="full"]
//! ```
//!
//! Review loop with mixed persistence — the implementation node keeps full
//! context across iterations while the review gate uses a summary:
//!
//! ```dot
//! digraph ReviewLoop {
//!     Start -> Implement
//!     Implement [agent="code-engineer", persist="full"]
//!     Implement -> Review
//!     Review [ask="Approve or revise?", persist="summary"]
//!     Review -> End [label="Approve"]
//!     Review -> Implement [label="Revise"]
//! }
//! ```
//!
//! Gist-level persistence for a lightweight planning step:
//!
//! ```dot
//! Plan [agent="planner", persist="gist"]
//! ```
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

use tracing::{debug, warn};

use crate::error::AttractorResult;
use crate::graph::{AttrValue, Graph, Node, attr};
use crate::transform::Transform;

pub struct NodeSugarTransform;

/// Set an attribute on the node only if it is not already present.
fn set_attr_if_absent(node: &mut Node, key: &str, value: impl Into<AttrValue>) {
    if !node.attrs.contains_key(key) {
        node.attrs.insert(key.into(), value.into());
    }
}

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

/// Resolve the fidelity level for a `persist` attribute value.
///
/// Returns `Some(fidelity_str)` for valid values that enable persistence,
/// `None` for values that disable it or are unrecognized.
fn resolve_persist_fidelity(persist_val: &AttrValue, node_id: &str) -> Option<&'static str> {
    match persist_val {
        AttrValue::String(s) => match s.as_str() {
            "true" | "full" => Some("full"),
            "summary" => Some("summary:medium"),
            "gist" => Some("summary:low"),
            "details" => Some("summary:high"),
            "false" | "off" => None,
            other => {
                warn!(
                    "unknown persist value \"{other}\" on node \"{node_id}\"; \
                     removing without expansion"
                );
                None
            }
        },
        AttrValue::Boolean(true) => Some("full"),
        AttrValue::Boolean(false) => None,
        other => {
            warn!(
                "persist attribute on node \"{node_id}\" has unsupported type {}; \
                 removing without expansion",
                other.type_name()
            );
            None
        }
    }
}

/// Expand the `persist` sugar attribute on a single node.
///
/// Parses the `persist` value (string or boolean), maps it to a fidelity
/// level, and auto-generates a `thread_id` when appropriate. The `persist`
/// key is always removed regardless of value.
fn expand_persist(node: &mut Node) {
    let Some(persist_val) = node.attrs.shift_remove(attr::PERSIST) else {
        return;
    };

    let Some(fidelity) = resolve_persist_fidelity(&persist_val, &node.id) else {
        debug!("persist disabled on node \"{}\"", node.id);
        return;
    };

    // Explicit fidelity wins; do not generate thread_id either.
    if node.attrs.contains_key(attr::FIDELITY) {
        debug!(
            "persist on node \"{}\" — explicit fidelity preserved, skipping thread_id generation",
            node.id
        );
        return;
    }

    node.attrs.insert(attr::FIDELITY.into(), fidelity.into());
    set_attr_if_absent(node, attr::THREAD_ID, format!("persist:{}", node.id));
}

impl Transform for NodeSugarTransform {
    fn name(&self) -> &'static str {
        "node_sugar"
    }

    fn apply(&self, graph: &mut Graph) -> AttractorResult<()> {
        for node in graph.nodes.values_mut() {
            // Drain all sugar keys up front so none leak into the graph.
            // Precedence: ask > workflow > shell > branch (first present wins).
            let ask_val = node.attrs.shift_remove(attr::ASK);
            let shell_val = node.attrs.shift_remove(attr::SHELL);
            let branch_val = node.attrs.shift_remove(attr::BRANCH);
            let workflow_val = node.attrs.shift_remove(attr::WORKFLOW);

            // --- Property shortcuts (highest priority) ---

            // `ask` implies human gate (shape=hexagon, value→label)
            if let Some(val) = ask_val {
                set_attr_if_absent(node, attr::SHAPE, Graph::HUMAN_SHAPE);
                set_attr_if_absent(node, attr::LABEL, val);
                continue;
            }

            // `workflow` implies a workflow composition node
            if let Some(val) = workflow_val {
                set_attr_if_absent(node, attr::TYPE, attr::WORKFLOW);
                node.attrs.insert(attr::WORKFLOW.into(), val);
                continue;
            }

            // `shell` implies shell node (shape=parallelogram, value→shell_command)
            if let Some(val) = shell_val {
                set_attr_if_absent(node, attr::SHAPE, Graph::SHELL_SHAPE);
                set_attr_if_absent(node, attr::SHELL_COMMAND, val);
                continue;
            }

            // `branch` implies conditional node (shape=diamond, value→label)
            if let Some(val) = branch_val {
                set_attr_if_absent(node, attr::SHAPE, Graph::CONDITIONAL_SHAPE);
                set_attr_if_absent(node, attr::LABEL, val);
                continue;
            }

            // `interview` implies human gate (multi-question interview)
            if node.attrs.contains_key(attr::INTERVIEW) {
                set_attr_if_absent(node, attr::SHAPE, Graph::HUMAN_SHAPE);
                continue;
            }

            // `fan_out` implies parallel fan-out node (component shape).
            // Unlike other sugar keys, `fan_out` is NOT drained — it must
            // remain on the node for the `ParallelHandler` to read at runtime.
            if node.attrs.contains_key(attr::FAN_OUT) {
                set_attr_if_absent(node, attr::SHAPE, Graph::PARALLEL_SHAPE);
                continue;
            }

            // --- Remaining inference only when no explicit shape ---
            if node.attrs.contains_key(attr::SHAPE) {
                continue;
            }

            // `prompt` or `agent` means this is an LLM node — skip prefix-
            // based ID inference so e.g. `FanOutAnalysis [prompt="..."]`
            // stays codergen. Structural IDs (Start/End/Fail) are exempt.
            if node.attrs.contains_key(attr::PROMPT)
                || node.attrs.contains_key(attr::AGENT)
                || node.attrs.keys().any(|k| k.starts_with("agent."))
            {
                if let Some(shape) = structural_shape(&node.id) {
                    node.attrs.insert(attr::SHAPE.into(), shape.into());
                }
                continue;
            }

            // --- Node ID-based shape inference ---
            if let Some(shape) = infer_shape_from_id(&node.id) {
                node.attrs.insert(attr::SHAPE.into(), shape.into());
            }
        }

        // --- Second pass: expand `persist` sugar on every node ---
        for node in graph.nodes.values_mut() {
            expand_persist(node);
        }

        Ok(())
    }
}
