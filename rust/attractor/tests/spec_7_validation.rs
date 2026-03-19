//! Tests for validation and linting (§7).

use stencila_attractor::error::AttractorResult;
use stencila_attractor::graph::{AttrValue, Edge, Graph, Node, attr};
use stencila_attractor::validation::{self, Diagnostic, LintRule, Severity};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal valid pipeline: start -> task -> exit.
fn valid_pipeline() -> Graph {
    let mut g = Graph::new("test");

    let mut start = Node::new("start");
    start
        .attrs
        .insert(attr::SHAPE.into(), Graph::START_SHAPE.into());
    g.add_node(start);

    let mut task = Node::new("task");
    task.attrs.insert("label".into(), AttrValue::from("Task"));
    task.attrs
        .insert("prompt".into(), AttrValue::from("Do something"));
    g.add_node(task);

    let mut exit = Node::new("exit");
    exit.attrs
        .insert(attr::SHAPE.into(), Graph::EXIT_SHAPE.into());
    g.add_node(exit);

    g.add_edge(Edge::new("start", "task"));
    g.add_edge(Edge::new("task", "exit"));

    g
}

/// Count diagnostics of a given severity.
fn count_severity(diagnostics: &[Diagnostic], severity: Severity) -> usize {
    diagnostics
        .iter()
        .filter(|d| d.severity == severity)
        .count()
}

/// Find diagnostics by rule name.
fn find_by_rule<'a>(diagnostics: &'a [Diagnostic], rule: &str) -> Vec<&'a Diagnostic> {
    diagnostics.iter().filter(|d| d.rule == rule).collect()
}

// ===========================================================================
// Valid pipeline produces no errors
// ===========================================================================

#[test]
fn valid_pipeline_no_errors() {
    let g = valid_pipeline();
    let diagnostics = validation::validate(&g, &[]);
    let errors = count_severity(&diagnostics, Severity::Error);
    assert_eq!(
        errors, 0,
        "valid pipeline should have no errors: {diagnostics:?}"
    );
}

#[test]
fn valid_pipeline_validate_or_raise_succeeds() -> AttractorResult<()> {
    let g = valid_pipeline();
    let _warnings = validation::validate_or_raise(&g, &[])?;
    Ok(())
}

// ===========================================================================
// 1. start_node (ERROR)
// ===========================================================================

#[test]
fn rule_start_node_missing() {
    let mut g = Graph::new("test");
    let mut exit = Node::new("exit");
    exit.attrs
        .insert(attr::SHAPE.into(), Graph::EXIT_SHAPE.into());
    g.add_node(exit);

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "start_node");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].severity, Severity::Error);
}

#[test]
fn rule_start_node_multiple() {
    let mut g = Graph::new("test");

    let mut s1 = Node::new("s1");
    s1.attrs
        .insert(attr::SHAPE.into(), Graph::START_SHAPE.into());
    g.add_node(s1);

    let mut s2 = Node::new("start");
    s2.attrs
        .insert(attr::SHAPE.into(), Graph::START_SHAPE.into());
    g.add_node(s2);

    let mut exit = Node::new("exit");
    exit.attrs
        .insert(attr::SHAPE.into(), Graph::EXIT_SHAPE.into());
    g.add_node(exit);

    g.add_edge(Edge::new("s1", "exit"));
    g.add_edge(Edge::new("start", "exit"));

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "start_node");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].severity, Severity::Error);
}

#[test]
fn rule_start_node_by_id() {
    // Start node found by ID (not shape)
    let mut g = Graph::new("test");

    g.add_node(Node::new("start"));
    let mut exit = Node::new("exit");
    exit.attrs
        .insert(attr::SHAPE.into(), Graph::EXIT_SHAPE.into());
    g.add_node(exit);
    g.add_edge(Edge::new("start", "exit"));

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "start_node");
    assert!(hits.is_empty(), "start by ID should be valid");
}

// ===========================================================================
// 2. terminal_node (ERROR)
// ===========================================================================

#[test]
fn rule_terminal_node_missing() {
    let mut g = Graph::new("test");
    let mut start = Node::new("start");
    start
        .attrs
        .insert(attr::SHAPE.into(), Graph::START_SHAPE.into());
    g.add_node(start);

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "terminal_node");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].severity, Severity::Error);
}

#[test]
fn rule_terminal_node_by_id() {
    let mut g = Graph::new("test");
    let mut start = Node::new("start");
    start
        .attrs
        .insert(attr::SHAPE.into(), Graph::START_SHAPE.into());
    g.add_node(start);
    g.add_node(Node::new("exit"));
    g.add_edge(Edge::new("start", "exit"));

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "terminal_node");
    assert!(hits.is_empty());
}

#[test]
fn rule_terminal_node_by_id_end() {
    // §7.2: terminal node can also be found by id="end"
    let mut g = Graph::new("test");
    let mut start = Node::new("start");
    start
        .attrs
        .insert(attr::SHAPE.into(), Graph::START_SHAPE.into());
    g.add_node(start);
    g.add_node(Node::new("end"));
    g.add_edge(Edge::new("start", "end"));

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "terminal_node");
    assert!(hits.is_empty(), "id=end should be recognized as exit node");
}

/// Regression: graphs with multiple exit nodes must be rejected (§11.2
/// requires exactly one), not silently accepted.
#[test]
fn rule_terminal_node_rejects_multiple() {
    let mut g = Graph::new("test");
    let mut start = Node::new("start");
    start
        .attrs
        .insert(attr::SHAPE.into(), Graph::START_SHAPE.into());
    g.add_node(start);

    let mut exit1 = Node::new("exit1");
    exit1
        .attrs
        .insert(attr::SHAPE.into(), Graph::EXIT_SHAPE.into());
    g.add_node(exit1);

    let mut exit2 = Node::new("exit2");
    exit2
        .attrs
        .insert(attr::SHAPE.into(), Graph::EXIT_SHAPE.into());
    g.add_node(exit2);

    g.add_edge(Edge::new("start", "exit1"));
    g.add_edge(Edge::new("start", "exit2"));

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "terminal_node");
    assert_eq!(hits.len(), 1, "multiple exit nodes should produce an error");
    assert_eq!(hits[0].severity, Severity::Error);
    assert!(
        hits[0].message.contains("2 exit nodes"),
        "error message should mention the count: {}",
        hits[0].message
    );
}

// ===========================================================================
// 3. reachability (ERROR)
// ===========================================================================

#[test]
fn rule_reachability_orphan() {
    let mut g = valid_pipeline();
    g.add_node(Node::new("orphan"));

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "reachability");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].severity, Severity::Error);
    assert_eq!(hits[0].node_id.as_deref(), Some("orphan"));
}

#[test]
fn rule_reachability_all_connected() {
    let g = valid_pipeline();
    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "reachability");
    assert!(hits.is_empty());
}

// ===========================================================================
// 4. edge_target_exists (ERROR)
// ===========================================================================

#[test]
fn rule_edge_target_missing() {
    let mut g = valid_pipeline();
    g.add_edge(Edge::new("task", "nonexistent"));

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "edge_target_exists");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].severity, Severity::Error);
    assert!(hits[0].message.contains("nonexistent"));
}

#[test]
fn rule_edge_source_missing() {
    let mut g = valid_pipeline();
    g.add_edge(Edge::new("ghost", "task"));

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "edge_target_exists");
    assert!(!hits.is_empty());
}

// ===========================================================================
// 5. start_no_incoming (ERROR)
// ===========================================================================

#[test]
fn rule_start_has_incoming() {
    let mut g = valid_pipeline();
    g.add_edge(Edge::new("task", "start"));

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "start_no_incoming");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].severity, Severity::Error);
}

// ===========================================================================
// 6. exit_no_outgoing (ERROR)
// ===========================================================================

#[test]
fn rule_exit_has_outgoing() {
    let mut g = valid_pipeline();
    g.add_edge(Edge::new("exit", "task"));

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "exit_no_outgoing");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].severity, Severity::Error);
}

// ===========================================================================
// 7. condition_syntax (ERROR)
// ===========================================================================

#[test]
fn rule_condition_syntax_valid() {
    let mut g = valid_pipeline();
    let mut edge = Edge::new("task", "exit");
    edge.attrs
        .insert("condition".into(), AttrValue::from("outcome=success"));
    // Replace the existing task->exit edge
    g.edges.retain(|e| !(e.from == "task" && e.to == "exit"));
    g.add_edge(edge);

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "condition_syntax");
    assert!(hits.is_empty());
}

#[test]
fn rule_condition_syntax_invalid() {
    let mut g = valid_pipeline();
    let mut edge = Edge::new("task", "exit");
    edge.attrs
        .insert("condition".into(), AttrValue::from("==bad"));
    g.edges.retain(|e| !(e.from == "task" && e.to == "exit"));
    g.add_edge(edge);

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "condition_syntax");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].severity, Severity::Error);
}

// ===========================================================================
// 8. stylesheet_syntax (ERROR)
// ===========================================================================

#[test]
fn rule_stylesheet_syntax_valid() {
    let mut g = valid_pipeline();
    g.graph_attrs.insert(
        "model_stylesheet".into(),
        AttrValue::from("* { llm_model: gpt-4; }"),
    );

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "stylesheet_syntax");
    assert!(hits.is_empty());
}

#[test]
fn rule_stylesheet_syntax_invalid() {
    let mut g = valid_pipeline();
    g.graph_attrs
        .insert("model_stylesheet".into(), AttrValue::from("{ broken }"));

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "stylesheet_syntax");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].severity, Severity::Error);
}

#[test]
fn rule_stylesheet_syntax_absent_is_ok() {
    let g = valid_pipeline();
    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "stylesheet_syntax");
    assert!(hits.is_empty());
}

#[test]
fn rule_stylesheet_syntax_non_string_is_error() {
    let mut g = valid_pipeline();
    g.graph_attrs
        .insert("model_stylesheet".into(), AttrValue::Integer(42));

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "stylesheet_syntax");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].severity, Severity::Error);
    assert!(
        hits[0].message.contains("must be a string"),
        "{}",
        hits[0].message
    );
}

#[test]
fn rule_stylesheet_syntax_overrides_key_valid() {
    let mut g = valid_pipeline();
    g.graph_attrs.insert(
        "overrides".into(),
        AttrValue::from("* { llm_model: gpt-4; }"),
    );

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "stylesheet_syntax");
    assert!(hits.is_empty());
}

#[test]
fn rule_stylesheet_syntax_overrides_key_invalid() {
    let mut g = valid_pipeline();
    g.graph_attrs
        .insert("overrides".into(), AttrValue::from("{ broken }"));

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "stylesheet_syntax");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].severity, Severity::Error);
}

// ===========================================================================
// 9. type_known (WARNING)
// ===========================================================================

#[test]
fn rule_type_known_warns_on_unknown() {
    let mut g = valid_pipeline();
    let mut n = Node::new("weird");
    n.attrs
        .insert("type".into(), AttrValue::from("alien_handler"));
    g.add_node(n);
    g.add_edge(Edge::new("start", "weird"));
    g.add_edge(Edge::new("weird", "exit"));
    // Remove task edges to avoid reachability errors for task
    g.edges
        .retain(|e| !(e.from == "start" && e.to == "task" || e.from == "task" && e.to == "exit"));
    g.nodes.swap_remove("task");

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "type_known");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].severity, Severity::Warning);
    assert!(hits[0].message.contains("alien_handler"));
}

#[test]
fn rule_type_known_all_built_in_ok() {
    let g = valid_pipeline();
    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "type_known");
    assert!(hits.is_empty());
}

// ===========================================================================
// 10. fidelity_valid (WARNING)
// ===========================================================================

#[test]
fn rule_fidelity_valid_bad_node() {
    let mut g = valid_pipeline();
    if let Some(node) = g.get_node_mut("task") {
        node.attrs
            .insert("fidelity".into(), AttrValue::from("ultra_hd"));
    }

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "fidelity_valid");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].severity, Severity::Warning);
}

#[test]
fn rule_fidelity_valid_ok() {
    let mut g = valid_pipeline();
    if let Some(node) = g.get_node_mut("task") {
        node.attrs
            .insert("fidelity".into(), AttrValue::from("summary:high"));
    }

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "fidelity_valid");
    assert!(hits.is_empty());
}

#[test]
fn rule_fidelity_valid_graph_level() {
    let mut g = valid_pipeline();
    g.graph_attrs
        .insert("default_fidelity".into(), AttrValue::from("bogus"));

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "fidelity_valid");
    assert_eq!(hits.len(), 1);
}

// ===========================================================================
// 11. retry_target_exists (WARNING)
// ===========================================================================

#[test]
fn rule_retry_target_missing_node() {
    let mut g = valid_pipeline();
    if let Some(node) = g.get_node_mut("task") {
        node.attrs
            .insert("retry_target".into(), AttrValue::from("ghost"));
    }

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "retry_target_exists");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].severity, Severity::Warning);
}

#[test]
fn rule_retry_target_exists_ok() {
    let mut g = valid_pipeline();
    if let Some(node) = g.get_node_mut("task") {
        node.attrs
            .insert("retry_target".into(), AttrValue::from("task"));
    }

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "retry_target_exists");
    assert!(hits.is_empty());
}

#[test]
fn rule_retry_target_graph_level_missing() {
    let mut g = valid_pipeline();
    g.graph_attrs
        .insert("retry_target".into(), AttrValue::from("phantom"));

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "retry_target_exists");
    assert_eq!(hits.len(), 1);
}

// ===========================================================================
// 12. goal_gate_has_retry (WARNING)
// ===========================================================================

#[test]
fn rule_goal_gate_no_retry() {
    let mut g = valid_pipeline();
    if let Some(node) = g.get_node_mut("task") {
        node.attrs.insert("goal_gate".into(), AttrValue::from(true));
    }

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "goal_gate_has_retry");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].severity, Severity::Warning);
}

#[test]
fn rule_goal_gate_false_does_not_trigger() {
    let mut g = valid_pipeline();
    if let Some(node) = g.get_node_mut("task") {
        node.attrs
            .insert("goal_gate".into(), AttrValue::from(false));
    }

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "goal_gate_has_retry");
    assert!(
        hits.is_empty(),
        "goal_gate=false should not trigger the rule"
    );
}

#[test]
fn rule_goal_gate_string_value_does_not_trigger() {
    // String values are not boolean true — should not be treated as goal gates
    let mut g = valid_pipeline();
    if let Some(node) = g.get_node_mut("task") {
        node.attrs
            .insert("goal_gate".into(), AttrValue::from("All tests pass"));
    }

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "goal_gate_has_retry");
    assert!(
        hits.is_empty(),
        "string-valued goal_gate should not be treated as boolean true"
    );
}

#[test]
fn rule_goal_gate_with_node_retry() {
    let mut g = valid_pipeline();
    if let Some(node) = g.get_node_mut("task") {
        node.attrs.insert("goal_gate".into(), AttrValue::from(true));
        node.attrs
            .insert("retry_target".into(), AttrValue::from("task"));
    }

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "goal_gate_has_retry");
    assert!(hits.is_empty());
}

#[test]
fn rule_goal_gate_with_graph_retry() {
    let mut g = valid_pipeline();
    g.graph_attrs
        .insert("retry_target".into(), AttrValue::from("task"));
    if let Some(node) = g.get_node_mut("task") {
        node.attrs.insert("goal_gate".into(), AttrValue::from(true));
    }

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "goal_gate_has_retry");
    assert!(hits.is_empty());
}

// ===========================================================================
// 13. prompt_on_llm_nodes (WARNING)
// ===========================================================================

#[test]
fn rule_prompt_missing_on_codergen() {
    let mut g = valid_pipeline();
    // task has default shape=box which maps to codergen
    // Remove its prompt and label attrs
    if let Some(node) = g.get_node_mut("task") {
        node.attrs.swap_remove("prompt");
        node.attrs.swap_remove("label");
    }

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "prompt_on_llm_nodes");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].severity, Severity::Warning);
}

#[test]
fn rule_prompt_present_on_codergen() {
    let g = valid_pipeline(); // task has both label and prompt
    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "prompt_on_llm_nodes");
    assert!(hits.is_empty());
}

// ===========================================================================
// 14. shell_command_present (WARNING)
// ===========================================================================

#[test]
fn rule_shell_command_missing() {
    let mut g = valid_pipeline();
    let mut shell_node = Node::new("runner");
    shell_node
        .attrs
        .insert(attr::SHAPE.into(), Graph::SHELL_SHAPE.into());
    g.add_node(shell_node);
    g.edges.retain(|e| !(e.from == "task" && e.to == "exit"));
    g.add_edge(Edge::new("task", "runner"));
    g.add_edge(Edge::new("runner", "exit"));

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "shell_command_present");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].severity, Severity::Warning);
    assert!(hits[0].message.contains("runner"));
}

#[test]
fn rule_shell_command_present_ok() {
    let mut g = valid_pipeline();
    let mut shell_node = Node::new("runner");
    shell_node
        .attrs
        .insert(attr::SHAPE.into(), Graph::SHELL_SHAPE.into());
    shell_node
        .attrs
        .insert("shell_command".into(), AttrValue::from("echo hello"));
    g.add_node(shell_node);
    g.edges.retain(|e| !(e.from == "task" && e.to == "exit"));
    g.add_edge(Edge::new("task", "runner"));
    g.add_edge(Edge::new("runner", "exit"));

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "shell_command_present");
    assert!(hits.is_empty());
}

// ===========================================================================
// validate_or_raise
// ===========================================================================

#[test]
fn validate_or_raise_errors_on_missing_start() {
    let mut g = Graph::new("test");
    let mut exit = Node::new("exit");
    exit.attrs
        .insert(attr::SHAPE.into(), Graph::EXIT_SHAPE.into());
    g.add_node(exit);

    let result = validation::validate_or_raise(&g, &[]);
    let err = result.expect_err("should fail with missing start node");
    assert!(err.to_string().contains("validation failed"));
}

#[test]
fn validate_or_raise_passes_with_warnings() -> AttractorResult<()> {
    let mut g = valid_pipeline();
    // Add a warning-level issue (invalid fidelity) but no errors
    if let Some(node) = g.get_node_mut("task") {
        node.attrs
            .insert("fidelity".into(), AttrValue::from("bogus"));
    }

    let diagnostics = validation::validate_or_raise(&g, &[])?;
    assert!(diagnostics.iter().any(|d| d.severity == Severity::Warning));
    Ok(())
}

// ===========================================================================
// Custom lint rules (§7.4)
// ===========================================================================

struct NoGoalRule;

impl LintRule for NoGoalRule {
    fn name(&self) -> &str {
        "custom_no_goal"
    }

    fn apply(&self, graph: &Graph) -> Vec<Diagnostic> {
        if graph.get_graph_attr("goal").is_none() {
            vec![Diagnostic {
                rule: self.name().to_string(),
                severity: Severity::Warning,
                message: "pipeline has no goal attribute".into(),
                node_id: None,
                edge: None,
                fix: Some("add graph [goal=\"...\"]".into()),
            }]
        } else {
            vec![]
        }
    }
}

#[test]
fn custom_rule_is_executed() {
    let g = valid_pipeline(); // no goal attr
    let custom = NoGoalRule;
    let diagnostics = validation::validate(&g, &[&custom]);
    let hits = find_by_rule(&diagnostics, "custom_no_goal");
    assert_eq!(hits.len(), 1);
}

#[test]
fn custom_rule_passes() {
    let mut g = valid_pipeline();
    g.graph_attrs
        .insert("goal".into(), AttrValue::from("test goal"));
    let custom = NoGoalRule;
    let diagnostics = validation::validate(&g, &[&custom]);
    let hits = find_by_rule(&diagnostics, "custom_no_goal");
    assert!(hits.is_empty());
}

// ===========================================================================
// mismatched_agent_thread_id (ERROR)
// ===========================================================================

#[test]
fn rule_mismatched_agent_thread_id_different_agents() {
    let mut g = valid_pipeline();

    // Two codergen nodes sharing thread_id="shared" but with different agents
    let mut node_a = Node::new("node_a");
    node_a
        .attrs
        .insert("thread_id".into(), AttrValue::from("shared"));
    node_a
        .attrs
        .insert("agent".into(), AttrValue::from("agent_alpha"));
    node_a
        .attrs
        .insert("prompt".into(), AttrValue::from("do A"));
    g.add_node(node_a);

    let mut node_b = Node::new("node_b");
    node_b
        .attrs
        .insert("thread_id".into(), AttrValue::from("shared"));
    node_b
        .attrs
        .insert("agent".into(), AttrValue::from("agent_beta"));
    node_b
        .attrs
        .insert("prompt".into(), AttrValue::from("do B"));
    g.add_node(node_b);

    // Wire them into the pipeline so they are reachable
    g.edges.retain(|e| !(e.from == "task" && e.to == "exit"));
    g.add_edge(Edge::new("task", "node_a"));
    g.add_edge(Edge::new("node_a", "node_b"));
    g.add_edge(Edge::new("node_b", "exit"));

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "mismatched_agent_thread_id");
    assert_eq!(
        hits.len(),
        1,
        "should emit exactly one diagnostic for mismatched agents on shared thread_id: {diagnostics:?}"
    );
    assert_eq!(hits[0].severity, Severity::Error);
}

#[test]
fn rule_mismatched_agent_thread_id_same_agent_no_diagnostic() {
    let mut g = valid_pipeline();

    // Two codergen nodes sharing thread_id="shared" and the *same* agent
    let mut node_a = Node::new("node_a");
    node_a
        .attrs
        .insert("thread_id".into(), AttrValue::from("shared"));
    node_a
        .attrs
        .insert("agent".into(), AttrValue::from("agent_alpha"));
    node_a
        .attrs
        .insert("prompt".into(), AttrValue::from("do A"));
    g.add_node(node_a);

    let mut node_b = Node::new("node_b");
    node_b
        .attrs
        .insert("thread_id".into(), AttrValue::from("shared"));
    node_b
        .attrs
        .insert("agent".into(), AttrValue::from("agent_alpha"));
    node_b
        .attrs
        .insert("prompt".into(), AttrValue::from("do B"));
    g.add_node(node_b);

    // Wire them into the pipeline so they are reachable
    g.edges.retain(|e| !(e.from == "task" && e.to == "exit"));
    g.add_edge(Edge::new("task", "node_a"));
    g.add_edge(Edge::new("node_a", "node_b"));
    g.add_edge(Edge::new("node_b", "exit"));

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "mismatched_agent_thread_id");
    assert!(
        hits.is_empty(),
        "same agent on shared thread_id should produce no diagnostic: {diagnostics:?}"
    );
}

#[test]
fn rule_mismatched_agent_thread_id_is_registered_in_builtin_rules() {
    use stencila_attractor::validation::rules::builtin_rules;

    let rules = builtin_rules();
    let found = rules
        .iter()
        .any(|r| r.name() == "mismatched_agent_thread_id");
    assert!(
        found,
        "MismatchedAgentThreadIdRule must be registered in builtin_rules()"
    );
}

// ===========================================================================
// parallel_branch_thread_id (ERROR) — same thread_id in different branches
// ===========================================================================

/// Helper: build a pipeline with a parallel fan-out node that has two branches,
/// each containing a task node. Returns the graph and the IDs of the two branch
/// task nodes so callers can set attributes on them.
fn parallel_two_branch_pipeline() -> Graph {
    let mut g = Graph::new("test");

    // start → parallel_node → {branch_a_task, branch_b_task} → fan_in → exit
    let mut start = Node::new("start");
    start
        .attrs
        .insert(attr::SHAPE.into(), Graph::START_SHAPE.into());
    g.add_node(start);

    let mut par = Node::new("parallel_node");
    par.attrs
        .insert(attr::SHAPE.into(), Graph::PARALLEL_SHAPE.into());
    g.add_node(par);

    let mut branch_a = Node::new("branch_a_task");
    branch_a
        .attrs
        .insert("prompt".into(), AttrValue::from("do A"));
    g.add_node(branch_a);

    let mut branch_b = Node::new("branch_b_task");
    branch_b
        .attrs
        .insert("prompt".into(), AttrValue::from("do B"));
    g.add_node(branch_b);

    let mut fan_in = Node::new("fan_in");
    fan_in
        .attrs
        .insert(attr::SHAPE.into(), Graph::PARALLEL_FAN_IN_SHAPE.into());
    g.add_node(fan_in);

    let mut exit = Node::new("exit");
    exit.attrs
        .insert(attr::SHAPE.into(), Graph::EXIT_SHAPE.into());
    g.add_node(exit);

    g.add_edge(Edge::new("start", "parallel_node"));
    g.add_edge(Edge::new("parallel_node", "branch_a_task"));
    g.add_edge(Edge::new("parallel_node", "branch_b_task"));
    g.add_edge(Edge::new("branch_a_task", "fan_in"));
    g.add_edge(Edge::new("branch_b_task", "fan_in"));
    g.add_edge(Edge::new("fan_in", "exit"));

    g
}

#[test]
fn rule_parallel_branch_thread_id_conflict_emits_error() {
    let mut g = parallel_two_branch_pipeline();

    // Both branch tasks share thread_id="conflict" with fidelity="full"
    if let Some(node) = g.get_node_mut("branch_a_task") {
        node.attrs
            .insert("fidelity".into(), AttrValue::from("full"));
        node.attrs
            .insert("thread_id".into(), AttrValue::from("conflict"));
    }
    if let Some(node) = g.get_node_mut("branch_b_task") {
        node.attrs
            .insert("fidelity".into(), AttrValue::from("full"));
        node.attrs
            .insert("thread_id".into(), AttrValue::from("conflict"));
    }

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "parallel_branch_thread_id");
    assert_eq!(
        hits.len(),
        1,
        "should emit exactly one ERROR for conflicting thread_id across parallel branches: {diagnostics:?}"
    );
    assert_eq!(hits[0].severity, Severity::Error);
    assert!(
        hits[0].message.contains("conflict"),
        "error message should mention the conflicting thread_id: {}",
        hits[0].message
    );
}

#[test]
fn rule_parallel_branch_thread_id_distinct_no_diagnostic() {
    let mut g = parallel_two_branch_pipeline();

    // Each branch task has a *different* thread_id with fidelity="full"
    if let Some(node) = g.get_node_mut("branch_a_task") {
        node.attrs
            .insert("fidelity".into(), AttrValue::from("full"));
        node.attrs
            .insert("thread_id".into(), AttrValue::from("thread_a"));
    }
    if let Some(node) = g.get_node_mut("branch_b_task") {
        node.attrs
            .insert("fidelity".into(), AttrValue::from("full"));
        node.attrs
            .insert("thread_id".into(), AttrValue::from("thread_b"));
    }

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "parallel_branch_thread_id");
    assert!(
        hits.is_empty(),
        "distinct thread_ids in different branches should produce no diagnostic: {diagnostics:?}"
    );
}

#[test]
fn rule_parallel_branch_thread_id_is_registered_in_builtin_rules() {
    use stencila_attractor::validation::rules::builtin_rules;

    let rules = builtin_rules();
    let found = rules
        .iter()
        .any(|r| r.name() == "parallel_branch_thread_id");
    assert!(
        found,
        "ParallelBranchThreadIdRule must be registered in builtin_rules()"
    );
}

// ===========================================================================
// thread_id_without_fidelity (WARNING) — thread_id has no effect without
// fidelity="full"
// ===========================================================================

/// AC1: A node with `thread_id` but *no* `fidelity` attribute produces a
/// WARNING diagnostic from `ThreadIdWithoutFidelityRule`.
#[test]
fn rule_thread_id_without_fidelity_no_fidelity_attr() {
    let mut g = valid_pipeline();

    // Set thread_id on the task node but do *not* set fidelity
    if let Some(node) = g.get_node_mut("task") {
        node.attrs
            .insert("thread_id".into(), AttrValue::from("my_thread"));
    }

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "thread_id_without_fidelity");
    assert_eq!(
        hits.len(),
        1,
        "should emit exactly one WARNING when thread_id is set but fidelity is absent: {diagnostics:?}"
    );
    assert_eq!(hits[0].severity, Severity::Warning);
    assert!(
        hits[0].message.contains("thread_id"),
        "warning message should mention thread_id: {}",
        hits[0].message
    );
}

/// AC1 (variant): A node with `thread_id` and `fidelity` set to something other
/// than `full` (e.g. `compact`) also produces a WARNING.
#[test]
fn rule_thread_id_without_fidelity_non_full_fidelity() {
    let mut g = valid_pipeline();

    if let Some(node) = g.get_node_mut("task") {
        node.attrs
            .insert("thread_id".into(), AttrValue::from("my_thread"));
        node.attrs
            .insert("fidelity".into(), AttrValue::from("compact"));
    }

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "thread_id_without_fidelity");
    assert_eq!(
        hits.len(),
        1,
        "should emit WARNING when thread_id is set but fidelity is not full: {diagnostics:?}"
    );
    assert_eq!(hits[0].severity, Severity::Warning);
}

/// AC3: A node with both `thread_id` and `fidelity="full"` produces no
/// diagnostic from this rule.
#[test]
fn rule_thread_id_with_fidelity_full_no_diagnostic() {
    let mut g = valid_pipeline();

    if let Some(node) = g.get_node_mut("task") {
        node.attrs
            .insert("thread_id".into(), AttrValue::from("my_thread"));
        node.attrs
            .insert("fidelity".into(), AttrValue::from("full"));
    }

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "thread_id_without_fidelity");
    assert!(
        hits.is_empty(),
        "thread_id with fidelity=full should produce no diagnostic: {diagnostics:?}"
    );
}

/// AC4: A node with neither `thread_id` nor `fidelity` produces no diagnostic
/// from this rule.
#[test]
fn rule_thread_id_without_fidelity_neither_set_no_diagnostic() {
    let g = valid_pipeline();

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "thread_id_without_fidelity");
    assert!(
        hits.is_empty(),
        "no thread_id and no fidelity should produce no diagnostic: {diagnostics:?}"
    );
}

/// AC2: The rule is registered in `builtin_rules()`.
#[test]
fn rule_thread_id_without_fidelity_is_registered_in_builtin_rules() {
    use stencila_attractor::validation::rules::builtin_rules;

    let rules = builtin_rules();
    let found = rules
        .iter()
        .any(|r| r.name() == "thread_id_without_fidelity");
    assert!(
        found,
        "ThreadIdWithoutFidelityRule must be registered in builtin_rules()"
    );
}

// ===========================================================================
// nodes_in_cycles() helper (AC1)
// ===========================================================================

/// AC1a: An acyclic graph returns an empty set from nodes_in_cycles().
#[test]
fn nodes_in_cycles_acyclic_returns_empty() {
    use stencila_attractor::validation::rules::nodes_in_cycles;

    // Simple chain: a -> b -> c (no cycles)
    let mut g = Graph::new("test");
    g.add_node(Node::new("a"));
    g.add_node(Node::new("b"));
    g.add_node(Node::new("c"));
    g.add_edge(Edge::new("a", "b"));
    g.add_edge(Edge::new("b", "c"));

    let cycle_nodes = nodes_in_cycles(&g);
    assert!(
        cycle_nodes.is_empty(),
        "acyclic graph should have no nodes in cycles: {cycle_nodes:?}"
    );
}

/// AC1b: A single-cycle graph returns exactly the cycle participants.
#[test]
fn nodes_in_cycles_single_cycle() {
    use stencila_attractor::validation::rules::nodes_in_cycles;

    // a -> b -> c -> a (all three in a cycle)
    let mut g = Graph::new("test");
    g.add_node(Node::new("a"));
    g.add_node(Node::new("b"));
    g.add_node(Node::new("c"));
    g.add_edge(Edge::new("a", "b"));
    g.add_edge(Edge::new("b", "c"));
    g.add_edge(Edge::new("c", "a"));

    let cycle_nodes = nodes_in_cycles(&g);
    assert!(cycle_nodes.contains("a"), "node 'a' should be in cycle");
    assert!(cycle_nodes.contains("b"), "node 'b' should be in cycle");
    assert!(cycle_nodes.contains("c"), "node 'c' should be in cycle");
    assert_eq!(cycle_nodes.len(), 3, "should have exactly 3 cycle nodes");
}

/// AC1c: A graph with multiple cycles returns all cycle participants.
#[test]
fn nodes_in_cycles_multi_cycle() {
    use stencila_attractor::validation::rules::nodes_in_cycles;

    // Two separate cycles: a -> b -> a, and c -> d -> c
    // Plus a non-cycle node e
    let mut g = Graph::new("test");
    g.add_node(Node::new("a"));
    g.add_node(Node::new("b"));
    g.add_node(Node::new("c"));
    g.add_node(Node::new("d"));
    g.add_node(Node::new("e"));
    g.add_edge(Edge::new("a", "b"));
    g.add_edge(Edge::new("b", "a"));
    g.add_edge(Edge::new("c", "d"));
    g.add_edge(Edge::new("d", "c"));
    g.add_edge(Edge::new("a", "e")); // e is not in any cycle

    let cycle_nodes = nodes_in_cycles(&g);
    assert!(cycle_nodes.contains("a"), "node 'a' should be in cycle");
    assert!(cycle_nodes.contains("b"), "node 'b' should be in cycle");
    assert!(cycle_nodes.contains("c"), "node 'c' should be in cycle");
    assert!(cycle_nodes.contains("d"), "node 'd' should be in cycle");
    assert!(
        !cycle_nodes.contains("e"),
        "node 'e' should NOT be in any cycle"
    );
}

/// AC1d: A self-loop is detected as a cycle.
#[test]
fn nodes_in_cycles_self_loop() {
    use stencila_attractor::validation::rules::nodes_in_cycles;

    // a -> a (self-loop), b is isolated
    let mut g = Graph::new("test");
    g.add_node(Node::new("a"));
    g.add_node(Node::new("b"));
    g.add_edge(Edge::new("a", "a"));

    let cycle_nodes = nodes_in_cycles(&g);
    assert!(
        cycle_nodes.contains("a"),
        "self-loop node 'a' should be in cycle"
    );
    assert!(
        !cycle_nodes.contains("b"),
        "node 'b' should NOT be in cycle"
    );
}

// ===========================================================================
// fidelity_full_no_cycle_thread_id (WARNING) — fallback thread_id unreliable
// in cycles (AC2)
// ===========================================================================

/// AC2: A node with fidelity="full", no explicit thread_id, participating in a
/// cycle should produce a WARNING.
#[test]
fn rule_fidelity_full_no_cycle_thread_id_warns_in_cycle() {
    // Build a pipeline with a cycle: start -> task -> loopback -> task -> exit
    // The task node has fidelity="full" and no thread_id.
    let mut g = Graph::new("test");

    let mut start = Node::new("start");
    start
        .attrs
        .insert(attr::SHAPE.into(), Graph::START_SHAPE.into());
    g.add_node(start);

    let mut task = Node::new("task");
    task.attrs
        .insert("prompt".into(), AttrValue::from("do something"));
    task.attrs
        .insert("fidelity".into(), AttrValue::from("full"));
    // No thread_id set — should trigger the warning
    g.add_node(task);

    let mut loopback = Node::new("loopback");
    loopback
        .attrs
        .insert(attr::SHAPE.into(), Graph::CONDITIONAL_SHAPE.into());
    g.add_node(loopback);

    let mut exit = Node::new("exit");
    exit.attrs
        .insert(attr::SHAPE.into(), Graph::EXIT_SHAPE.into());
    g.add_node(exit);

    // start -> task -> loopback -> task (cycle) and loopback -> exit
    g.add_edge(Edge::new("start", "task"));
    g.add_edge(Edge::new("task", "loopback"));
    g.add_edge(Edge::new("loopback", "task")); // creates a cycle
    g.add_edge(Edge::new("loopback", "exit"));

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "fidelity_full_no_cycle_thread_id");
    assert_eq!(
        hits.len(),
        1,
        "should emit exactly one WARNING for fidelity=full node in cycle without thread_id: {diagnostics:?}"
    );
    assert_eq!(hits[0].severity, Severity::Warning);
    assert_eq!(hits[0].node_id.as_deref(), Some("task"));
}

// ===========================================================================
// fidelity_full_no_cycle_thread_id — negative: explicit thread_id suppresses
// warning (AC3)
// ===========================================================================

/// AC3: A node with fidelity="full" AND explicit thread_id in a cycle should
/// produce NO warning from FidelityFullNoCycleThreadIdRule.
#[test]
fn rule_fidelity_full_no_cycle_thread_id_explicit_thread_id_no_warning() {
    let mut g = Graph::new("test");

    let mut start = Node::new("start");
    start
        .attrs
        .insert(attr::SHAPE.into(), Graph::START_SHAPE.into());
    g.add_node(start);

    let mut task = Node::new("task");
    task.attrs
        .insert("prompt".into(), AttrValue::from("do something"));
    task.attrs
        .insert("fidelity".into(), AttrValue::from("full"));
    task.attrs
        .insert("thread_id".into(), AttrValue::from("explicit_thread"));
    g.add_node(task);

    let mut loopback = Node::new("loopback");
    loopback
        .attrs
        .insert(attr::SHAPE.into(), Graph::CONDITIONAL_SHAPE.into());
    g.add_node(loopback);

    let mut exit = Node::new("exit");
    exit.attrs
        .insert(attr::SHAPE.into(), Graph::EXIT_SHAPE.into());
    g.add_node(exit);

    g.add_edge(Edge::new("start", "task"));
    g.add_edge(Edge::new("task", "loopback"));
    g.add_edge(Edge::new("loopback", "task")); // creates a cycle
    g.add_edge(Edge::new("loopback", "exit"));

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "fidelity_full_no_cycle_thread_id");
    assert!(
        hits.is_empty(),
        "node with explicit thread_id in a cycle should produce no warning from this rule: {diagnostics:?}"
    );
}

/// AC3 (variant): A node with fidelity="full" and no thread_id that is NOT in a
/// cycle should produce no warning from this rule.
#[test]
fn rule_fidelity_full_no_cycle_thread_id_no_cycle_no_warning() {
    let mut g = valid_pipeline();

    // Add fidelity="full" to the task node (which is NOT in any cycle)
    if let Some(node) = g.get_node_mut("task") {
        node.attrs
            .insert("fidelity".into(), AttrValue::from("full"));
        // No thread_id — but no cycle either, so no warning
    }

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "fidelity_full_no_cycle_thread_id");
    assert!(
        hits.is_empty(),
        "node not in a cycle should produce no warning: {diagnostics:?}"
    );
}

// ===========================================================================
// fidelity_full_no_cycle_thread_id — rule registration (AC4)
// ===========================================================================

/// AC4: The rule is registered in builtin_rules().
#[test]
fn rule_fidelity_full_no_cycle_thread_id_is_registered_in_builtin_rules() {
    use stencila_attractor::validation::rules::builtin_rules;

    let rules = builtin_rules();
    let found = rules
        .iter()
        .any(|r| r.name() == "fidelity_full_no_cycle_thread_id");
    assert!(
        found,
        "FidelityFullNoCycleThreadIdRule must be registered in builtin_rules()"
    );
}

// ===========================================================================
// persist_on_non_agent (INFO) — fidelity / persist:thread_id on non-codergen
// ===========================================================================

/// AC1: A Start node with `fidelity` set emits an INFO diagnostic.
#[test]
fn rule_persist_on_non_agent_start_node_with_fidelity() {
    let mut g = valid_pipeline();

    // Add fidelity to the start node (structural node → non-codergen)
    if let Some(node) = g.get_node_mut("start") {
        node.attrs
            .insert(attr::FIDELITY.into(), AttrValue::from("full"));
    }

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "persist_on_non_agent");
    assert_eq!(
        hits.len(),
        1,
        "fidelity on a Start node should emit exactly one INFO diagnostic: {diagnostics:?}"
    );
    assert_eq!(hits[0].severity, Severity::Info);
    assert_eq!(hits[0].node_id.as_deref(), Some("start"));
}

/// AC1: An End/Exit node with a thread_id starting with `persist:` emits INFO.
#[test]
fn rule_persist_on_non_agent_exit_node_with_persist_thread_id() {
    let mut g = valid_pipeline();

    if let Some(node) = g.get_node_mut("exit") {
        node.attrs
            .insert(attr::THREAD_ID.into(), AttrValue::from("persist:exit"));
    }

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "persist_on_non_agent");
    assert_eq!(
        hits.len(),
        1,
        "persist:* thread_id on an Exit node should emit INFO: {diagnostics:?}"
    );
    assert_eq!(hits[0].severity, Severity::Info);
    assert_eq!(hits[0].node_id.as_deref(), Some("exit"));
}

/// AC1: A Fail node with fidelity emits INFO.
#[test]
fn rule_persist_on_non_agent_fail_node_with_fidelity() {
    let mut g = Graph::new("test");

    let mut start = Node::new("start");
    start
        .attrs
        .insert(attr::SHAPE.into(), Graph::START_SHAPE.into());
    g.add_node(start);

    let mut fail = Node::new("fail");
    fail.attrs
        .insert(attr::SHAPE.into(), Graph::FAIL_SHAPE.into());
    fail.attrs
        .insert(attr::FIDELITY.into(), AttrValue::from("full"));
    g.add_node(fail);

    let mut exit = Node::new("exit");
    exit.attrs
        .insert(attr::SHAPE.into(), Graph::EXIT_SHAPE.into());
    g.add_node(exit);

    g.add_edge(Edge::new("start", "fail"));
    g.add_edge(Edge::new("start", "exit"));

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "persist_on_non_agent");
    assert_eq!(
        hits.len(),
        1,
        "fidelity on a Fail node should emit INFO: {diagnostics:?}"
    );
    assert_eq!(hits[0].severity, Severity::Info);
    assert_eq!(hits[0].node_id.as_deref(), Some("fail"));
}

/// AC1: A shell node (parallelogram shape) with fidelity emits INFO.
#[test]
fn rule_persist_on_non_agent_shell_node_with_fidelity() {
    let mut g = valid_pipeline();

    // Replace "task" with a shell node
    if let Some(node) = g.get_node_mut("task") {
        node.attrs
            .insert(attr::SHAPE.into(), Graph::SHELL_SHAPE.into());
        node.attrs
            .insert(attr::FIDELITY.into(), AttrValue::from("full"));
    }

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "persist_on_non_agent");
    assert_eq!(
        hits.len(),
        1,
        "fidelity on a shell node should emit INFO: {diagnostics:?}"
    );
    assert_eq!(hits[0].severity, Severity::Info);
}

/// AC1: A conditional (diamond) node with persist:* thread_id emits INFO.
#[test]
fn rule_persist_on_non_agent_conditional_node_with_persist_thread_id() {
    let mut g = valid_pipeline();

    if let Some(node) = g.get_node_mut("task") {
        node.attrs
            .insert(attr::SHAPE.into(), Graph::CONDITIONAL_SHAPE.into());
        node.attrs
            .insert(attr::THREAD_ID.into(), AttrValue::from("persist:task"));
    }

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "persist_on_non_agent");
    assert_eq!(
        hits.len(),
        1,
        "persist:* thread_id on a conditional node should emit INFO: {diagnostics:?}"
    );
    assert_eq!(hits[0].severity, Severity::Info);
}

/// AC3 (negative): A codergen (box) node with fidelity does NOT emit persist_on_non_agent.
#[test]
fn rule_persist_on_non_agent_codergen_node_no_diagnostic() {
    let mut g = valid_pipeline();

    // task is already a codergen node (default shape=box)
    if let Some(node) = g.get_node_mut("task") {
        node.attrs
            .insert(attr::FIDELITY.into(), AttrValue::from("full"));
        node.attrs
            .insert(attr::THREAD_ID.into(), AttrValue::from("persist:task"));
    }

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "persist_on_non_agent");
    assert!(
        hits.is_empty(),
        "fidelity on a codergen node should NOT emit persist_on_non_agent: {diagnostics:?}"
    );
}

/// AC3 (negative): A non-codergen node with a non-persist thread_id and no fidelity
/// should NOT fire the rule.
#[test]
fn rule_persist_on_non_agent_non_persist_thread_id_no_diagnostic() {
    let mut g = valid_pipeline();

    if let Some(node) = g.get_node_mut("task") {
        node.attrs
            .insert(attr::SHAPE.into(), Graph::SHELL_SHAPE.into());
        // thread_id that doesn't start with "persist:" — not relevant
        node.attrs
            .insert(attr::THREAD_ID.into(), AttrValue::from("manual:thread"));
    }

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "persist_on_non_agent");
    assert!(
        hits.is_empty(),
        "non-persist thread_id on non-codergen node should NOT emit persist_on_non_agent: {diagnostics:?}"
    );
}

/// The rule is registered in builtin_rules().
#[test]
fn rule_persist_on_non_agent_is_registered_in_builtin_rules() {
    use stencila_attractor::validation::rules::builtin_rules;

    let rules = builtin_rules();
    let found = rules.iter().any(|r| r.name() == "persist_on_non_agent");
    assert!(
        found,
        "persist_on_non_agent must be registered in builtin_rules()"
    );
}

// ===========================================================================
// persist_on_edge (WARNING) — persist attribute on an edge
// ===========================================================================

/// AC2: An edge with a `persist` attribute emits a WARNING.
#[test]
fn rule_persist_on_edge_fires_when_edge_has_persist() {
    let mut g = valid_pipeline();

    // Add persist attribute to the task->exit edge
    g.edges.retain(|e| !(e.from == "task" && e.to == "exit"));
    let mut edge = Edge::new("task", "exit");
    edge.attrs
        .insert(attr::PERSIST.into(), AttrValue::from("full"));
    g.add_edge(edge);

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "persist_on_edge");
    assert_eq!(
        hits.len(),
        1,
        "persist on an edge should emit exactly one WARNING: {diagnostics:?}"
    );
    assert_eq!(hits[0].severity, Severity::Warning);
    assert!(
        hits[0].edge.is_some(),
        "diagnostic should reference the offending edge"
    );
}

/// AC2 (multiple): Multiple edges with persist each produce a warning.
#[test]
fn rule_persist_on_edge_fires_for_each_edge_with_persist() {
    let mut g = valid_pipeline();

    // Add persist to start->task edge
    g.edges.retain(|e| !(e.from == "start" && e.to == "task"));
    let mut edge1 = Edge::new("start", "task");
    edge1
        .attrs
        .insert(attr::PERSIST.into(), AttrValue::from("true"));
    g.add_edge(edge1);

    // Add persist to task->exit edge
    g.edges.retain(|e| !(e.from == "task" && e.to == "exit"));
    let mut edge2 = Edge::new("task", "exit");
    edge2
        .attrs
        .insert(attr::PERSIST.into(), AttrValue::from("full"));
    g.add_edge(edge2);

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "persist_on_edge");
    assert_eq!(
        hits.len(),
        2,
        "two edges with persist should produce two WARNINGs: {diagnostics:?}"
    );
    assert!(hits.iter().all(|d| d.severity == Severity::Warning));
}

/// AC3 (negative): Edges without `persist` produce no persist_on_edge diagnostic.
#[test]
fn rule_persist_on_edge_no_persist_no_diagnostic() {
    let g = valid_pipeline();

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "persist_on_edge");
    assert!(
        hits.is_empty(),
        "edges without persist should produce no diagnostic: {diagnostics:?}"
    );
}

/// The rule is registered in builtin_rules().
#[test]
fn rule_persist_on_edge_is_registered_in_builtin_rules() {
    use stencila_attractor::validation::rules::builtin_rules;

    let rules = builtin_rules();
    let found = rules.iter().any(|r| r.name() == "persist_on_edge");
    assert!(
        found,
        "persist_on_edge must be registered in builtin_rules()"
    );
}

// ===========================================================================
// validate collects all diagnostics
// ===========================================================================

#[test]
fn validate_collects_multiple_diagnostics() {
    let mut g = Graph::new("test");
    // No start, no exit, orphan node
    g.add_node(Node::new("orphan"));

    let diagnostics = validation::validate(&g, &[]);
    // Should have at least: start_node error, terminal_node error
    assert!(count_severity(&diagnostics, Severity::Error) >= 2);
}
