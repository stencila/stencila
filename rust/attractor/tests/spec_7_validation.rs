//! Tests for validation and linting (§7).

use stencila_attractor::error::AttractorResult;
use stencila_attractor::graph::{AttrValue, Edge, Graph, Node};
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
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);

    let mut task = Node::new("task");
    task.attrs.insert("label".into(), AttrValue::from("Task"));
    task.attrs
        .insert("prompt".into(), AttrValue::from("Do something"));
    g.add_node(task);

    let mut exit = Node::new("exit");
    exit.attrs
        .insert("shape".into(), AttrValue::from("Msquare"));
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
        .insert("shape".into(), AttrValue::from("Msquare"));
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
    s1.attrs.insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(s1);

    let mut s2 = Node::new("start");
    s2.attrs.insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(s2);

    let mut exit = Node::new("exit");
    exit.attrs
        .insert("shape".into(), AttrValue::from("Msquare"));
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
        .insert("shape".into(), AttrValue::from("Msquare"));
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
        .insert("shape".into(), AttrValue::from("Mdiamond"));
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
        .insert("shape".into(), AttrValue::from("Mdiamond"));
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
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);
    g.add_node(Node::new("end"));
    g.add_edge(Edge::new("start", "end"));

    let diagnostics = validation::validate(&g, &[]);
    let hits = find_by_rule(&diagnostics, "terminal_node");
    assert!(hits.is_empty(), "id=end should be recognized as exit node");
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
        .insert("fidelity".into(), AttrValue::from("bogus"));

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
// validate_or_raise
// ===========================================================================

#[test]
fn validate_or_raise_errors_on_missing_start() {
    let mut g = Graph::new("test");
    let mut exit = Node::new("exit");
    exit.attrs
        .insert("shape".into(), AttrValue::from("Msquare"));
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
