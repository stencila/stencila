//! Built-in lint rules (§7.2).
//!
//! Provides all 13 built-in validation rules covering structural integrity,
//! condition syntax, stylesheet syntax, and best-practice warnings.

use std::collections::HashSet;

use super::{Diagnostic, LintRule, Severity};
use crate::graph::Graph;
use crate::types::HandlerType;

/// Return all 13 built-in lint rules.
#[must_use]
pub fn builtin_rules() -> Vec<Box<dyn LintRule>> {
    vec![
        Box::new(StartNodeRule),
        Box::new(TerminalNodeRule),
        Box::new(ReachabilityRule),
        Box::new(EdgeTargetExistsRule),
        Box::new(StartNoIncomingRule),
        Box::new(ExitNoOutgoingRule),
        Box::new(ConditionSyntaxRule),
        Box::new(StylesheetSyntaxRule),
        Box::new(TypeKnownRule),
        Box::new(FidelityValidRule),
        Box::new(RetryTargetExistsRule),
        Box::new(GoalGateHasRetryRule),
        Box::new(PromptOnLlmNodesRule),
    ]
}

// ---------------------------------------------------------------------------
// Helper: find start/exit nodes using shared criteria from Graph
// ---------------------------------------------------------------------------

fn find_start_nodes(graph: &Graph) -> Vec<&str> {
    graph
        .nodes
        .values()
        .filter(|n| Graph::is_start_node(n))
        .map(|n| n.id.as_str())
        .collect()
}

fn find_exit_nodes(graph: &Graph) -> Vec<&str> {
    graph
        .nodes
        .values()
        .filter(|n| Graph::is_exit_node(n))
        .map(|n| n.id.as_str())
        .collect()
}

// ---------------------------------------------------------------------------
// 1. start_node (ERROR) — exactly one start node
// ---------------------------------------------------------------------------

struct StartNodeRule;

impl LintRule for StartNodeRule {
    fn name(&self) -> &'static str {
        "start_node"
    }

    fn apply(&self, graph: &Graph) -> Vec<Diagnostic> {
        let starts = find_start_nodes(graph);
        match starts.len() {
            0 => vec![Diagnostic {
                rule: self.name().to_string(),
                severity: Severity::Error,
                message: "pipeline has no start node (shape=Mdiamond or id=start)".into(),
                node_id: None,
                edge: None,
                fix: Some("add a node with shape=Mdiamond".into()),
            }],
            1 => vec![],
            _ => vec![Diagnostic {
                rule: self.name().to_string(),
                severity: Severity::Error,
                message: format!(
                    "pipeline has {} start nodes (expected exactly 1): {}",
                    starts.len(),
                    starts.join(", ")
                ),
                node_id: None,
                edge: None,
                fix: Some("ensure exactly one start node".into()),
            }],
        }
    }
}

// ---------------------------------------------------------------------------
// 2. terminal_node (ERROR) — exactly one exit node
// ---------------------------------------------------------------------------

struct TerminalNodeRule;

impl LintRule for TerminalNodeRule {
    fn name(&self) -> &'static str {
        "terminal_node"
    }

    fn apply(&self, graph: &Graph) -> Vec<Diagnostic> {
        let exits = find_exit_nodes(graph);
        match exits.len() {
            0 => vec![Diagnostic {
                rule: self.name().to_string(),
                severity: Severity::Error,
                message: "pipeline has no exit node (shape=Msquare or id=exit/end)".into(),
                node_id: None,
                edge: None,
                fix: Some("add a node with shape=Msquare".into()),
            }],
            1 => vec![],
            _ => vec![Diagnostic {
                rule: self.name().to_string(),
                severity: Severity::Error,
                message: format!(
                    "pipeline has {} exit nodes (expected exactly 1): {}",
                    exits.len(),
                    exits.join(", ")
                ),
                node_id: None,
                edge: None,
                fix: Some("ensure exactly one exit node".into()),
            }],
        }
    }
}

// ---------------------------------------------------------------------------
// 3. reachability (ERROR) — all nodes reachable from start
// ---------------------------------------------------------------------------

// TODO(spec-ambiguity): §7.2 lists reachability as ERROR severity, but §11.12
// parity matrix item says "orphan node → warning". Using ERROR per §7.2
// (normative validation section). (spec: §7.2 vs §11.12)
struct ReachabilityRule;

impl LintRule for ReachabilityRule {
    fn name(&self) -> &'static str {
        "reachability"
    }

    fn apply(&self, graph: &Graph) -> Vec<Diagnostic> {
        let starts = find_start_nodes(graph);
        if starts.is_empty() {
            // start_node rule will report this
            return vec![];
        }

        let mut reachable = HashSet::new();
        let mut stack: Vec<&str> = starts.clone();

        while let Some(node_id) = stack.pop() {
            if !reachable.insert(node_id) {
                continue;
            }
            for edge in graph.outgoing_edges(node_id) {
                if !reachable.contains(edge.to.as_str()) {
                    stack.push(&edge.to);
                }
            }
        }

        graph
            .nodes
            .keys()
            .filter(|id| !reachable.contains(id.as_str()))
            .map(|id| Diagnostic {
                rule: self.name().to_string(),
                severity: Severity::Error,
                message: format!("node `{id}` is unreachable from the start node"),
                node_id: Some(id.clone()),
                edge: None,
                fix: Some("add an edge from a reachable node or remove this node".into()),
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// 4. edge_target_exists (ERROR) — all edge targets must exist
// ---------------------------------------------------------------------------

struct EdgeTargetExistsRule;

impl LintRule for EdgeTargetExistsRule {
    fn name(&self) -> &'static str {
        "edge_target_exists"
    }

    fn apply(&self, graph: &Graph) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        for edge in &graph.edges {
            if !graph.nodes.contains_key(&edge.from) {
                diagnostics.push(Diagnostic {
                    rule: self.name().to_string(),
                    severity: Severity::Error,
                    message: format!("edge source `{}` does not exist", edge.from),
                    node_id: None,
                    edge: Some((edge.from.clone(), edge.to.clone())),
                    fix: None,
                });
            }
            if !graph.nodes.contains_key(&edge.to) {
                diagnostics.push(Diagnostic {
                    rule: self.name().to_string(),
                    severity: Severity::Error,
                    message: format!("edge target `{}` does not exist", edge.to),
                    node_id: None,
                    edge: Some((edge.from.clone(), edge.to.clone())),
                    fix: None,
                });
            }
        }
        diagnostics
    }
}

// ---------------------------------------------------------------------------
// 5. start_no_incoming (ERROR) — start node must have no incoming edges
// ---------------------------------------------------------------------------

struct StartNoIncomingRule;

impl LintRule for StartNoIncomingRule {
    fn name(&self) -> &'static str {
        "start_no_incoming"
    }

    fn apply(&self, graph: &Graph) -> Vec<Diagnostic> {
        find_start_nodes(graph)
            .into_iter()
            .filter(|id| !graph.incoming_edges(id).is_empty())
            .map(|id| Diagnostic {
                rule: self.name().to_string(),
                severity: Severity::Error,
                message: format!("start node `{id}` has incoming edges"),
                node_id: Some(id.to_string()),
                edge: None,
                fix: Some("remove incoming edges to the start node".into()),
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// 6. exit_no_outgoing (ERROR) — exit node must have no outgoing edges
// ---------------------------------------------------------------------------

struct ExitNoOutgoingRule;

impl LintRule for ExitNoOutgoingRule {
    fn name(&self) -> &'static str {
        "exit_no_outgoing"
    }

    fn apply(&self, graph: &Graph) -> Vec<Diagnostic> {
        find_exit_nodes(graph)
            .into_iter()
            .filter(|id| !graph.outgoing_edges(id).is_empty())
            .map(|id| Diagnostic {
                rule: self.name().to_string(),
                severity: Severity::Error,
                message: format!("exit node `{id}` has outgoing edges"),
                node_id: Some(id.to_string()),
                edge: None,
                fix: Some("remove outgoing edges from the exit node".into()),
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// 7. condition_syntax (ERROR) — edge conditions must parse
// ---------------------------------------------------------------------------

struct ConditionSyntaxRule;

impl LintRule for ConditionSyntaxRule {
    fn name(&self) -> &'static str {
        "condition_syntax"
    }

    fn apply(&self, graph: &Graph) -> Vec<Diagnostic> {
        graph
            .edges
            .iter()
            .filter(|e| !e.condition().is_empty())
            .filter_map(|e| {
                crate::condition::parse_condition(e.condition())
                    .err()
                    .map(|err| Diagnostic {
                        rule: self.name().to_string(),
                        severity: Severity::Error,
                        message: format!(
                            "edge {} -> {} has invalid condition: {err}",
                            e.from, e.to
                        ),
                        node_id: None,
                        edge: Some((e.from.clone(), e.to.clone())),
                        fix: None,
                    })
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// 8. stylesheet_syntax (ERROR) — model_stylesheet must parse
// ---------------------------------------------------------------------------

struct StylesheetSyntaxRule;

impl LintRule for StylesheetSyntaxRule {
    fn name(&self) -> &'static str {
        "stylesheet_syntax"
    }

    fn apply(&self, graph: &Graph) -> Vec<Diagnostic> {
        let Some(attr_value) = graph.get_graph_attr("model_stylesheet") else {
            return vec![];
        };

        let Some(stylesheet_str) = attr_value.as_str() else {
            // Attribute exists but is not a string — flag as an error
            return vec![Diagnostic {
                rule: self.name().to_string(),
                severity: Severity::Error,
                message: format!(
                    "model_stylesheet must be a string, got {}",
                    attr_value.type_name()
                ),
                node_id: None,
                edge: None,
                fix: Some("use a quoted string value for model_stylesheet".into()),
            }];
        };

        match crate::stylesheet_parser::parse_stylesheet(stylesheet_str) {
            Ok(_) => vec![],
            Err(err) => vec![Diagnostic {
                rule: self.name().to_string(),
                severity: Severity::Error,
                message: format!("model_stylesheet is invalid: {err}"),
                node_id: None,
                edge: None,
                fix: None,
            }],
        }
    }
}

// ---------------------------------------------------------------------------
// 9. type_known (WARNING) — handler types should be recognized
// ---------------------------------------------------------------------------

struct TypeKnownRule;

impl LintRule for TypeKnownRule {
    fn name(&self) -> &'static str {
        "type_known"
    }

    fn apply(&self, graph: &Graph) -> Vec<Diagnostic> {
        graph
            .nodes
            .values()
            .filter(|n| n.handler_type().parse::<HandlerType>().is_err())
            .map(|n| Diagnostic {
                rule: self.name().to_string(),
                severity: Severity::Warning,
                message: format!(
                    "node `{}` has unrecognized handler type `{}`",
                    n.id,
                    n.handler_type()
                ),
                node_id: Some(n.id.clone()),
                edge: None,
                fix: None,
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// 10. fidelity_valid (WARNING) — fidelity mode must be valid
// ---------------------------------------------------------------------------

struct FidelityValidRule;

impl LintRule for FidelityValidRule {
    fn name(&self) -> &'static str {
        "fidelity_valid"
    }

    fn apply(&self, graph: &Graph) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Check node-level fidelity attrs
        for node in graph.nodes.values() {
            if let Some(fidelity_str) = node.get_str_attr("fidelity")
                && fidelity_str.parse::<crate::types::FidelityMode>().is_err()
            {
                diagnostics.push(Diagnostic {
                        rule: self.name().to_string(),
                        severity: Severity::Warning,
                        message: format!(
                            "node `{}` has invalid fidelity mode `{fidelity_str}`",
                            node.id
                        ),
                        node_id: Some(node.id.clone()),
                        edge: None,
                        fix: Some(
                            "use one of: full, truncate, compact, summary:low, summary:medium, summary:high"
                                .into(),
                        ),
                    });
            }
        }

        // Check edge-level fidelity attrs
        for edge in &graph.edges {
            if let Some(fidelity_str) = edge.get_attr("fidelity").and_then(|v| v.as_str())
                && fidelity_str.parse::<crate::types::FidelityMode>().is_err()
            {
                diagnostics.push(Diagnostic {
                        rule: self.name().to_string(),
                        severity: Severity::Warning,
                        message: format!(
                            "edge {} -> {} has invalid fidelity mode `{fidelity_str}`",
                            edge.from, edge.to
                        ),
                        node_id: None,
                        edge: Some((edge.from.clone(), edge.to.clone())),
                        fix: Some(
                            "use one of: full, truncate, compact, summary:low, summary:medium, summary:high"
                                .into(),
                        ),
                    });
            }
        }

        // Check graph-level default_fidelity (§2.5, §5.4)
        if let Some(fidelity_str) = graph
            .get_graph_attr("default_fidelity")
            .and_then(|v| v.as_str())
            && fidelity_str.parse::<crate::types::FidelityMode>().is_err()
        {
            diagnostics.push(Diagnostic {
                    rule: self.name().to_string(),
                    severity: Severity::Warning,
                    message: format!(
                        "graph-level default_fidelity mode `{fidelity_str}` is invalid"
                    ),
                    node_id: None,
                    edge: None,
                    fix: Some(
                        "use one of: full, truncate, compact, summary:low, summary:medium, summary:high"
                            .into(),
                    ),
                });
        }

        diagnostics
    }
}

// ---------------------------------------------------------------------------
// Shared: retry target attribute names and helpers
// ---------------------------------------------------------------------------

/// Attribute names for retry target references, checked on both nodes and graphs.
const RETRY_TARGET_ATTRS: &[&str] = &["retry_target", "fallback_retry_target"];

/// Check whether a node or the graph has any retry target defined.
fn has_any_retry_target(node: &crate::graph::Node, graph: &Graph) -> bool {
    RETRY_TARGET_ATTRS
        .iter()
        .any(|attr| node.get_str_attr(attr).is_some())
        || RETRY_TARGET_ATTRS.iter().any(|attr| {
            graph
                .get_graph_attr(attr)
                .and_then(|v| v.as_str())
                .is_some()
        })
}

// ---------------------------------------------------------------------------
// 11. retry_target_exists (WARNING) — retry_target nodes must exist
// ---------------------------------------------------------------------------

struct RetryTargetExistsRule;

impl LintRule for RetryTargetExistsRule {
    fn name(&self) -> &'static str {
        "retry_target_exists"
    }

    fn apply(&self, graph: &Graph) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Check node-level retry targets
        for node in graph.nodes.values() {
            for &attr_name in RETRY_TARGET_ATTRS {
                if let Some(target) = node.get_str_attr(attr_name)
                    && !graph.nodes.contains_key(target)
                {
                    diagnostics.push(Diagnostic {
                        rule: self.name().to_string(),
                        severity: Severity::Warning,
                        message: format!(
                            "node `{}` has {attr_name}=`{target}` but node `{target}` does not exist",
                            node.id
                        ),
                        node_id: Some(node.id.clone()),
                        edge: None,
                        fix: None,
                    });
                }
            }
        }

        // Check graph-level retry targets
        for &attr_name in RETRY_TARGET_ATTRS {
            if let Some(target) = graph.get_graph_attr(attr_name).and_then(|v| v.as_str())
                && !graph.nodes.contains_key(target)
            {
                diagnostics.push(Diagnostic {
                    rule: self.name().to_string(),
                    severity: Severity::Warning,
                    message: format!(
                        "graph has {attr_name}=`{target}` but node `{target}` does not exist"
                    ),
                    node_id: None,
                    edge: None,
                    fix: None,
                });
            }
        }

        diagnostics
    }
}

// ---------------------------------------------------------------------------
// 12. goal_gate_has_retry (WARNING) — goal_gate nodes should have retry_target
// ---------------------------------------------------------------------------

struct GoalGateHasRetryRule;

impl LintRule for GoalGateHasRetryRule {
    fn name(&self) -> &'static str {
        "goal_gate_has_retry"
    }

    fn apply(&self, graph: &Graph) -> Vec<Diagnostic> {
        graph
            .nodes
            .values()
            .filter(|n| n.get_bool_attr("goal_gate"))
            .filter(|n| !has_any_retry_target(n, graph))
            .map(|n| Diagnostic {
                rule: self.name().to_string(),
                severity: Severity::Warning,
                message: format!(
                    "node `{}` has goal_gate=true but no retry_target is defined \
                     (goal gate failures will terminate the pipeline)",
                    n.id
                ),
                node_id: Some(n.id.clone()),
                edge: None,
                fix: Some("add retry_target attribute to the node or graph".into()),
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// 13. prompt_on_llm_nodes (WARNING) — LLM nodes should have prompt/label
// ---------------------------------------------------------------------------

/// Handler types considered LLM-backed (need a prompt or label).
const LLM_HANDLER_TYPES: &[&str] = &[HandlerType::Codergen.as_str()];

struct PromptOnLlmNodesRule;

impl LintRule for PromptOnLlmNodesRule {
    fn name(&self) -> &'static str {
        "prompt_on_llm_nodes"
    }

    fn apply(&self, graph: &Graph) -> Vec<Diagnostic> {
        graph
            .nodes
            .values()
            .filter(|n| LLM_HANDLER_TYPES.contains(&n.handler_type()))
            .filter(|n| n.get_str_attr("prompt").is_none() && n.get_str_attr("label").is_none())
            .map(|n| Diagnostic {
                rule: self.name().to_string(),
                severity: Severity::Warning,
                message: format!("LLM node `{}` has no input or label attribute", n.id),
                node_id: Some(n.id.clone()),
                edge: None,
                fix: Some("add an input or label attribute".into()),
            })
            .collect()
    }
}
