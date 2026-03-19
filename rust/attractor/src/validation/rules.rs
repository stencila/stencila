//! Built-in lint rules (§7.2).
//!
//! Provides all built-in validation rules covering structural integrity,
//! condition syntax, stylesheet syntax, and best-practice warnings.

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};

use super::{Diagnostic, LintRule, Severity};
use crate::graph::{Graph, attr};
use crate::types::HandlerType;

/// Return all built-in lint rules.
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
        Box::new(ShellCommandPresentRule),
        Box::new(InterviewSpecRule),
        Box::new(DynamicFanOutRule),
        Box::new(DynamicFanOutMissingFanInRule),
        Box::new(NestedDynamicFanOutRule),
        Box::new(MismatchedAgentThreadIdRule),
        Box::new(ParallelBranchThreadIdRule),
        Box::new(ThreadIdWithoutFidelityRule),
        Box::new(FidelityFullNoCycleThreadIdRule),
        Box::new(PersistOnNonAgentRule),
        Box::new(PersistOnEdgeRule),
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
// 8. stylesheet_syntax (ERROR) — model_stylesheet (or overrides) must parse
// ---------------------------------------------------------------------------

struct StylesheetSyntaxRule;

impl LintRule for StylesheetSyntaxRule {
    fn name(&self) -> &'static str {
        "stylesheet_syntax"
    }

    fn apply(&self, graph: &Graph) -> Vec<Diagnostic> {
        // Try `model_stylesheet` first, then fall back to `overrides`.
        let Some(attr_value) = graph
            .get_graph_attr(attr::MODEL_STYLESHEET)
            .or_else(|| graph.get_graph_attr(attr::OVERRIDES))
        else {
            return vec![];
        };

        let Some(stylesheet_str) = attr_value.as_str() else {
            // Attribute exists but is not a string — flag as an error
            return vec![Diagnostic {
                rule: self.name().to_string(),
                severity: Severity::Error,
                message: format!("overrides must be a string, got {}", attr_value.type_name()),
                node_id: None,
                edge: None,
                fix: Some("use a quoted string value for overrides".into()),
            }];
        };

        match crate::stylesheet_parser::parse_stylesheet(stylesheet_str) {
            Ok(_) => vec![],
            Err(err) => vec![Diagnostic {
                rule: self.name().to_string(),
                severity: Severity::Error,
                message: format!("overrides is invalid: {err}"),
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
            if let Some(fidelity_str) = node.get_str_attr(attr::FIDELITY)
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
            if let Some(fidelity_str) = edge.get_attr(attr::FIDELITY).and_then(|v| v.as_str())
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
            .get_graph_attr(attr::DEFAULT_FIDELITY)
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
const RETRY_TARGET_ATTRS: &[&str] = &[attr::RETRY_TARGET, attr::FALLBACK_RETRY_TARGET];

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
            .filter(|n| n.get_bool_attr(attr::GOAL_GATE))
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

/// Attributes that count as valid input for an agent node.
/// Includes both direct attributes and their `-ref` variants (resolved at
/// workflow level before execution).
const AGENT_INPUT_ATTRS: &[&str] = &[
    attr::PROMPT,
    "prompt-ref",
    "prompt_ref",
    attr::INTERVIEW,
    "interview-ref",
    "interview_ref",
    attr::SHELL,
    "shell-ref",
    "shell_ref",
    attr::ASK,
    "ask-ref",
    "ask_ref",
    attr::LABEL,
];

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
            .filter(|n| {
                !AGENT_INPUT_ATTRS
                    .iter()
                    .any(|attr| n.get_str_attr(attr).is_some())
            })
            .map(|n| Diagnostic {
                rule: self.name().to_string(),
                severity: Severity::Warning,
                message: format!("agent node `{}` has no input or label attribute", n.id),
                node_id: Some(n.id.clone()),
                edge: None,
                fix: Some("add an input or label attribute".into()),
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// 14. shell_command_present (WARNING) — shell nodes should have shell_command
// ---------------------------------------------------------------------------

struct ShellCommandPresentRule;

impl LintRule for ShellCommandPresentRule {
    fn name(&self) -> &'static str {
        "shell_command_present"
    }

    fn apply(&self, graph: &Graph) -> Vec<Diagnostic> {
        graph
            .nodes
            .values()
            .filter(|n| n.handler_type() == HandlerType::Shell)
            .filter(|n| n.get_str_attr(attr::SHELL_COMMAND).is_none())
            .map(|n| Diagnostic {
                rule: self.name().to_string(),
                severity: Severity::Warning,
                message: format!("shell node `{}` has no shell_command attribute", n.id),
                node_id: Some(n.id.clone()),
                edge: None,
                fix: Some("add shell_command=\"...\" or use the shell sugar attribute".into()),
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// 15. interview_spec (WARNING) — interview spec validation
// ---------------------------------------------------------------------------

struct InterviewSpecRule;

impl LintRule for InterviewSpecRule {
    fn name(&self) -> &'static str {
        "interview_spec"
    }

    #[allow(clippy::too_many_lines)]
    fn apply(&self, graph: &Graph) -> Vec<Diagnostic> {
        use stencila_interviews::spec::{InterviewSpec, QuestionTypeSpec};

        let mut diagnostics = Vec::new();

        for node in graph.nodes.values() {
            let Some(spec_str) = node.get_str_attr(attr::INTERVIEW) else {
                continue;
            };

            // Warn when node-level `store` is combined with `interview`;
            // per-question `store` keys are the intended mechanism.
            if node.attrs.contains_key(attr::STORE) {
                diagnostics.push(Diagnostic {
                    rule: self.name().to_string(),
                    severity: Severity::Warning,
                    message: format!(
                        "node `{}` combines node-level `store` with `interview`; \
                         only the first question's answer is stored under the \
                         node-level key — use per-question `store` keys instead",
                        node.id
                    ),
                    node_id: Some(node.id.clone()),
                    edge: None,
                    fix: Some(
                        "remove the node-level `store` attribute and add `store` \
                         keys to individual questions in the interview spec"
                            .into(),
                    ),
                });
            }

            let spec = match InterviewSpec::parse(spec_str) {
                Ok(s) => s,
                Err(e) => {
                    diagnostics.push(Diagnostic {
                        rule: self.name().to_string(),
                        severity: Severity::Warning,
                        message: format!("node `{}`: {e}", node.id),
                        node_id: Some(node.id.clone()),
                        edge: None,
                        fix: Some("check the interview YAML/JSON syntax".into()),
                    });
                    continue;
                }
            };

            // Surface errors from semantic validation (show-if, finish-if, etc.)
            if let Err(errors) = spec.validate() {
                for error in errors {
                    diagnostics.push(Diagnostic {
                        rule: self.name().to_string(),
                        severity: Severity::Warning,
                        message: format!("node `{}`: {error}", node.id),
                        node_id: Some(node.id.clone()),
                        edge: None,
                        fix: None,
                    });
                }
            }

            // Warn on freeform questions without a store key
            for (i, q) in spec.questions.iter().enumerate() {
                if q.r#type == QuestionTypeSpec::Freeform && q.store.is_none() {
                    diagnostics.push(Diagnostic {
                        rule: self.name().to_string(),
                        severity: Severity::Warning,
                        message: format!(
                            "node `{}` interview question {} is freeform but has no `store` key; \
                             the answer will be collected but never stored",
                            node.id,
                            i + 1
                        ),
                        node_id: Some(node.id.clone()),
                        edge: None,
                        fix: Some("add a `store` key to the question".into()),
                    });
                }
            }

            // Warn when the first single-select question's option labels
            // do not match any outgoing edge labels
            if let Some(routing_q) = spec
                .questions
                .iter()
                .find(|q| q.r#type == QuestionTypeSpec::SingleSelect)
            {
                let edge_labels: HashSet<String> = graph
                    .outgoing_edges(&node.id)
                    .iter()
                    .map(|e| {
                        if e.label().is_empty() {
                            e.to.clone()
                        } else {
                            e.label().to_string()
                        }
                    })
                    .collect();

                if !edge_labels.is_empty() {
                    for opt in &routing_q.options {
                        if !edge_labels.contains(&opt.label) {
                            diagnostics.push(Diagnostic {
                                rule: self.name().to_string(),
                                severity: Severity::Warning,
                                message: format!(
                                    "node `{}` interview routing option `{}` does not match \
                                     any outgoing edge label",
                                    node.id, opt.label
                                ),
                                node_id: Some(node.id.clone()),
                                edge: None,
                                fix: Some(
                                    "ensure routing option labels match outgoing edge labels"
                                        .into(),
                                ),
                            });
                        }
                    }
                }
            } else {
                // No routing question — warn if there are multiple outgoing edges
                let outgoing_count = graph.outgoing_edges(&node.id).len();
                if outgoing_count > 1 {
                    diagnostics.push(Diagnostic {
                        rule: self.name().to_string(),
                        severity: Severity::Warning,
                        message: format!(
                            "node `{}` interview has no single-select routing question \
                             but has {outgoing_count} outgoing edges; the first edge will \
                             always be followed",
                            node.id
                        ),
                        node_id: Some(node.id.clone()),
                        edge: None,
                        fix: Some(
                            "add a single-select question for routing, or reduce to one \
                             outgoing edge"
                                .into(),
                        ),
                    });
                }
            }
        }

        diagnostics
    }
}

// ---------------------------------------------------------------------------
// Helper: BFS over a template subgraph, stopping at fan-in boundaries
// ---------------------------------------------------------------------------

/// Collect all node IDs reachable from `start_id` via BFS, stopping at
/// nodes with handler type `parallel.fan_in` (which mark the boundary
/// of the template subgraph).
fn template_subgraph_nodes(graph: &Graph, start_id: &str) -> HashSet<String> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(start_id.to_string());

    while let Some(current) = queue.pop_front() {
        if !visited.insert(current.clone()) {
            continue;
        }
        if let Some(node) = graph.get_node(&current)
            && node.handler_type() == HandlerType::ParallelFanIn
        {
            continue;
        }
        for edge in graph.outgoing_edges(&current) {
            if !visited.contains(&edge.to) {
                queue.push_back(edge.to.clone());
            }
        }
    }

    visited
}

// ---------------------------------------------------------------------------
// 16. dynamic_fan_out (ERROR) — dynamic fan-out nodes must have exactly 1 edge
// ---------------------------------------------------------------------------

struct DynamicFanOutRule;

impl LintRule for DynamicFanOutRule {
    fn name(&self) -> &'static str {
        "dynamic_fan_out"
    }

    fn apply(&self, graph: &Graph) -> Vec<Diagnostic> {
        graph
            .nodes
            .values()
            .filter(|n| n.attrs.contains_key(attr::FAN_OUT))
            .filter_map(|n| {
                let edge_count = graph.outgoing_edges(&n.id).len();
                (edge_count != 1).then(|| Diagnostic {
                    rule: self.name().to_string(),
                    severity: Severity::Error,
                    message: format!(
                        "dynamic fan-out node `{}` has {edge_count} outgoing edge(s); \
                         exactly 1 is required",
                        n.id
                    ),
                    node_id: Some(n.id.clone()),
                    edge: None,
                    fix: Some(
                        "a dynamic fan-out node must have exactly 1 outgoing edge to the \
                         template entry node"
                            .into(),
                    ),
                })
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// 17. dynamic_fan_out_missing_fan_in (WARNING) — fan-in should be reachable
// ---------------------------------------------------------------------------

struct DynamicFanOutMissingFanInRule;

impl LintRule for DynamicFanOutMissingFanInRule {
    fn name(&self) -> &'static str {
        "dynamic_fan_out_missing_fan_in"
    }

    fn apply(&self, graph: &Graph) -> Vec<Diagnostic> {
        graph
            .nodes
            .values()
            .filter(|n| n.attrs.contains_key(attr::FAN_OUT))
            .filter(|n| {
                let edges = graph.outgoing_edges(&n.id);
                edges.len() == 1
            })
            .filter(|n| {
                let template_entry = &graph.outgoing_edges(&n.id)[0].to;
                let subgraph = template_subgraph_nodes(graph, template_entry);
                // If no fan-in node was reached, no node in the subgraph
                // will have the ParallelFanIn handler type.
                !subgraph.iter().any(|id| {
                    graph
                        .get_node(id)
                        .is_some_and(|node| node.handler_type() == HandlerType::ParallelFanIn)
                })
            })
            .map(|n| Diagnostic {
                rule: self.name().to_string(),
                severity: Severity::Warning,
                message: format!(
                    "dynamic fan-out node `{}` has no reachable fan-in (tripleoctagon) node; \
                     branch results will be written to context but there will be no \
                     fan-in handler to consolidate them",
                    n.id
                ),
                node_id: Some(n.id.clone()),
                edge: None,
                fix: Some(
                    "add a tripleoctagon fan-in node downstream of the template subgraph".into(),
                ),
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// 18. nested_dynamic_fan_out (ERROR) — no nested dynamic fan-outs
// ---------------------------------------------------------------------------

struct NestedDynamicFanOutRule;

impl LintRule for NestedDynamicFanOutRule {
    fn name(&self) -> &'static str {
        "nested_dynamic_fan_out"
    }

    fn apply(&self, graph: &Graph) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for node in graph.nodes.values() {
            if !node.attrs.contains_key(attr::FAN_OUT) {
                continue;
            }

            let edges = graph.outgoing_edges(&node.id);
            if edges.len() != 1 {
                continue; // DynamicFanOutRule handles this
            }

            let template_entry = &edges[0].to;
            let subgraph = template_subgraph_nodes(graph, template_entry);

            for sub_node_id in &subgraph {
                if let Some(sub_node) = graph.get_node(sub_node_id)
                    && sub_node.attrs.contains_key(attr::FAN_OUT)
                {
                    diagnostics.push(Diagnostic {
                        rule: self.name().to_string(),
                        severity: Severity::Error,
                        message: format!(
                            "nested dynamic fan-out: node `{}` is inside the template \
                                 subgraph of dynamic fan-out node `{}`; nested dynamic \
                                 fan-out is not supported",
                            sub_node_id, node.id
                        ),
                        node_id: Some(sub_node_id.clone()),
                        edge: None,
                        fix: Some(
                            "remove the inner fan_out attribute or restructure the \
                                 pipeline to avoid nesting"
                                .into(),
                        ),
                    });
                }
            }
        }

        diagnostics
    }
}

// ---------------------------------------------------------------------------
// 19. mismatched_agent_thread_id (ERROR) — nodes sharing a thread_id must use the same agent
// ---------------------------------------------------------------------------

struct MismatchedAgentThreadIdRule;

impl LintRule for MismatchedAgentThreadIdRule {
    fn name(&self) -> &'static str {
        "mismatched_agent_thread_id"
    }

    fn apply(&self, graph: &Graph) -> Vec<Diagnostic> {
        // Track the first agent seen for each thread_id; emit a diagnostic the
        // first time a different agent is encountered.
        let mut first_agent: HashMap<&str, &str> = HashMap::new();
        let mut mismatched: HashSet<&str> = HashSet::new();

        for node in graph.nodes.values() {
            if let Some(tid) = node.get_str_attr(attr::THREAD_ID) {
                let agent = node.get_str_attr(attr::AGENT).unwrap_or("");
                match first_agent.entry(tid) {
                    Entry::Vacant(e) => {
                        e.insert(agent);
                    }
                    Entry::Occupied(e) => {
                        if *e.get() != agent {
                            mismatched.insert(tid);
                        }
                    }
                }
            }
        }

        mismatched
            .into_iter()
            .map(|tid| Diagnostic {
                rule: self.name().to_string(),
                severity: Severity::Error,
                message: format!(
                    "nodes sharing thread_id `{tid}` have different `agent` attributes"
                ),
                node_id: None,
                edge: None,
                fix: Some("ensure all nodes with the same thread_id use the same agent".into()),
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Helper: check whether a node has fidelity="full"
// ---------------------------------------------------------------------------

fn has_full_fidelity(node: &crate::graph::Node) -> bool {
    node.get_str_attr(attr::FIDELITY)
        .is_some_and(|f| f == "full")
}

// ---------------------------------------------------------------------------
// 20. parallel_branch_thread_id (ERROR) — same thread_id must not appear in
//     different parallel branches when fidelity="full"
// ---------------------------------------------------------------------------

/// Validates that the same `thread_id` does not appear in different
/// parallel branches when `fidelity="full"`. Concurrent branches sharing
/// a thread ID would cause data races on the pooled session, so this
/// rule reports an error when it detects the conflict.
struct ParallelBranchThreadIdRule;

impl LintRule for ParallelBranchThreadIdRule {
    fn name(&self) -> &'static str {
        "parallel_branch_thread_id"
    }

    fn apply(&self, graph: &Graph) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Find all parallel fan-out nodes (shape = component).
        for node in graph.nodes.values() {
            if node.handler_type() != HandlerType::Parallel {
                continue;
            }

            let branches = graph.outgoing_edges(&node.id);
            if branches.len() < 2 {
                continue;
            }

            // For each branch, collect thread_ids from nodes with fidelity="full".
            // Map: thread_id → index of the first branch that contains it.
            let mut thread_id_branch: HashMap<String, usize> = HashMap::new();
            let mut conflicting: HashSet<String> = HashSet::new();

            for (branch_idx, edge) in branches.iter().enumerate() {
                let reachable = template_subgraph_nodes(graph, &edge.to);

                for nid in &reachable {
                    if let Some(n) = graph.get_node(nid)
                        && has_full_fidelity(n)
                        && let Some(tid) = n.get_str_attr(attr::THREAD_ID)
                    {
                        match thread_id_branch.entry(tid.to_string()) {
                            Entry::Vacant(e) => {
                                e.insert(branch_idx);
                            }
                            Entry::Occupied(e) => {
                                if *e.get() != branch_idx {
                                    conflicting.insert(tid.to_string());
                                }
                            }
                        }
                    }
                }
            }

            for tid in &conflicting {
                diagnostics.push(Diagnostic {
                    rule: self.name().to_string(),
                    severity: Severity::Error,
                    message: format!(
                        "thread_id `{tid}` with fidelity=full appears in multiple parallel \
                         branches of fan-out node `{}`; concurrent writes to the same thread \
                         would conflict",
                        node.id
                    ),
                    node_id: Some(node.id.clone()),
                    edge: None,
                    fix: Some(
                        "use distinct thread_id values for nodes in different parallel branches"
                            .into(),
                    ),
                });
            }
        }

        diagnostics
    }
}

// ---------------------------------------------------------------------------
// 21. thread_id_without_fidelity (WARNING) — thread_id has no effect without
//     fidelity="full"
// ---------------------------------------------------------------------------

struct ThreadIdWithoutFidelityRule;

impl LintRule for ThreadIdWithoutFidelityRule {
    fn name(&self) -> &'static str {
        "thread_id_without_fidelity"
    }

    fn apply(&self, graph: &Graph) -> Vec<Diagnostic> {
        graph
            .nodes
            .values()
            .filter(|n| n.get_str_attr(attr::THREAD_ID).is_some())
            .filter(|n| !has_full_fidelity(n))
            .map(|n| Diagnostic {
                rule: self.name().to_string(),
                severity: Severity::Warning,
                message: format!(
                    "node `{}` has `thread_id` but fidelity is not set to \"full\"; \
                     thread_id has no effect without fidelity=\"full\"",
                    n.id
                ),
                node_id: Some(n.id.clone()),
                edge: None,
                fix: Some("add fidelity=\"full\" or remove thread_id".into()),
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Helper: find all nodes participating in cycles (DFS-based)
// ---------------------------------------------------------------------------

/// Return the set of node IDs that participate in at least one cycle.
///
/// Uses Tarjan-style DFS with a recursion stack to detect back edges. A node
/// is "in a cycle" if it can reach itself through one or more directed edges,
/// including the trivial self-loop case.
#[must_use]
pub fn nodes_in_cycles(graph: &Graph) -> HashSet<String> {
    // Build an adjacency list for efficient lookup.  Nodes with no outgoing
    // edges are handled via `map_or` in `dfs_cycle`, so we only need entries
    // for edge sources.
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for edge in &graph.edges {
        adj.entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
    }

    let mut visited: HashSet<&str> = HashSet::new();
    let mut on_stack: HashSet<&str> = HashSet::new();
    let mut result: HashSet<String> = HashSet::new();

    for node_id in graph.nodes.keys() {
        if !visited.contains(node_id.as_str()) {
            dfs_cycle(
                node_id.as_str(),
                &adj,
                &mut visited,
                &mut on_stack,
                &mut result,
            );
        }
    }

    result
}

/// DFS helper that marks nodes participating in cycles.
///
/// Returns `true` if the current node can reach a node on the recursion stack
/// (i.e., it participates in a cycle).
fn dfs_cycle<'a>(
    node: &'a str,
    adj: &HashMap<&'a str, Vec<&'a str>>,
    visited: &mut HashSet<&'a str>,
    on_stack: &mut HashSet<&'a str>,
    cycle_nodes: &mut HashSet<String>,
) -> bool {
    visited.insert(node);
    on_stack.insert(node);

    let mut is_in_cycle = false;

    for &next in adj.get(node).map_or(&[][..], Vec::as_slice) {
        if !visited.contains(next) {
            if dfs_cycle(next, adj, visited, on_stack, cycle_nodes) {
                // `next` (or a descendant) can reach something on the stack.
                // If `node` is still on the stack, it is part of the cycle.
                if on_stack.contains(node) {
                    cycle_nodes.insert(node.to_string());
                    is_in_cycle = true;
                }
            }
        } else if on_stack.contains(next) {
            // Back edge: both `node` and `next` are in a cycle.
            cycle_nodes.insert(node.to_string());
            cycle_nodes.insert(next.to_string());
            is_in_cycle = true;
        }
    }

    on_stack.remove(node);
    is_in_cycle
}

// ---------------------------------------------------------------------------
// 22. fidelity_full_no_cycle_thread_id (WARNING) — fallback thread_id
//     unreliable in cycles
// ---------------------------------------------------------------------------

struct FidelityFullNoCycleThreadIdRule;

impl LintRule for FidelityFullNoCycleThreadIdRule {
    fn name(&self) -> &'static str {
        "fidelity_full_no_cycle_thread_id"
    }

    fn apply(&self, graph: &Graph) -> Vec<Diagnostic> {
        let cycle_members = nodes_in_cycles(graph);

        graph
            .nodes
            .values()
            .filter(|n| has_full_fidelity(n))
            .filter(|n| n.get_str_attr(attr::THREAD_ID).is_none())
            .filter(|n| cycle_members.contains(&n.id))
            .map(|n| Diagnostic {
                rule: self.name().to_string(),
                severity: Severity::Warning,
                message: format!(
                    "node `{}` has fidelity=\"full\" without an explicit thread_id and \
                     participates in a cycle; the fallback thread_id may be unreliable \
                     in loops",
                    n.id
                ),
                node_id: Some(n.id.clone()),
                edge: None,
                fix: Some("add an explicit thread_id attribute".into()),
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Helper: check whether a node has persist-related attributes
// ---------------------------------------------------------------------------

/// Return `true` if the node carries an explicit `fidelity` attribute or a
/// `thread_id` whose value starts with `persist:`.
fn has_persist_attrs(node: &crate::graph::Node) -> bool {
    node.attrs.contains_key(attr::FIDELITY)
        || node
            .get_str_attr(attr::THREAD_ID)
            .is_some_and(|t| t.starts_with("persist:"))
}

// ---------------------------------------------------------------------------
// 23. persist_on_non_agent (INFO) — fidelity or persist:* thread_id on a
//     non-codergen node
// ---------------------------------------------------------------------------

struct PersistOnNonAgentRule;

impl LintRule for PersistOnNonAgentRule {
    fn name(&self) -> &'static str {
        "persist_on_non_agent"
    }

    fn apply(&self, graph: &Graph) -> Vec<Diagnostic> {
        graph
            .nodes
            .values()
            .filter(|n| n.shape() != Graph::CODERGEN_SHAPE)
            .filter(|n| has_persist_attrs(n))
            .map(|n| Diagnostic {
                rule: self.name().to_string(),
                severity: Severity::Info,
                message: format!(
                    "node `{}` (shape={}) has persist-related attributes but is not a \
                     codergen node; persistence has no effect on this node type",
                    n.id,
                    n.shape()
                ),
                node_id: Some(n.id.clone()),
                edge: None,
                fix: Some(
                    "remove fidelity/persist thread_id or change the node shape to box".into(),
                ),
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// 24. persist_on_edge (WARNING) — persist attribute on an edge
// ---------------------------------------------------------------------------

struct PersistOnEdgeRule;

impl LintRule for PersistOnEdgeRule {
    fn name(&self) -> &'static str {
        "persist_on_edge"
    }

    fn apply(&self, graph: &Graph) -> Vec<Diagnostic> {
        graph
            .edges
            .iter()
            .filter(|e| e.attrs.contains_key(attr::PERSIST))
            .map(|e| Diagnostic {
                rule: self.name().to_string(),
                severity: Severity::Warning,
                message: format!(
                    "edge {} -> {} has a `persist` attribute; persist is a node-level \
                     attribute and has no effect on edges",
                    e.from, e.to
                ),
                node_id: None,
                edge: Some((e.from.clone(), e.to.clone())),
                fix: Some("move persist to the source or target node instead".into()),
            })
            .collect()
    }
}
