//! Tests for model stylesheet (§8).

use stencila_attractor::error::AttractorResult;
use stencila_attractor::graph::{AttrValue, Edge, Graph, Node};
use stencila_attractor::stylesheet::{apply_stylesheet, parse_and_apply_stylesheet};
use stencila_attractor::stylesheet_parser::{
    Declaration, ParsedStylesheet, Selector, StylesheetRule, parse_stylesheet,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a pipeline with nodes for stylesheet testing.
fn stylesheet_pipeline() -> Graph {
    let mut g = Graph::new("Pipeline");

    let mut start = Node::new("start");
    start
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);

    let mut plan = Node::new("plan");
    plan.attrs.insert("label".into(), AttrValue::from("Plan"));
    g.add_node(plan);

    let mut implement = Node::new("implement");
    implement
        .attrs
        .insert("label".into(), AttrValue::from("Implement"));
    implement
        .attrs
        .insert("class".into(), AttrValue::from("code"));
    g.add_node(implement);

    let mut critical_review = Node::new("critical_review");
    critical_review
        .attrs
        .insert("label".into(), AttrValue::from("Critical Review"));
    g.add_node(critical_review);

    let mut exit = Node::new("exit");
    exit.attrs
        .insert("shape".into(), AttrValue::from("Msquare"));
    g.add_node(exit);

    g.add_edge(Edge::new("start", "plan"));
    g.add_edge(Edge::new("plan", "implement"));
    g.add_edge(Edge::new("implement", "critical_review"));
    g.add_edge(Edge::new("critical_review", "exit"));

    g
}

// ===========================================================================
// Parser: selectors
// ===========================================================================

#[test]
fn parse_universal_selector() -> AttractorResult<()> {
    let s = parse_stylesheet("* { llm_model: gpt-4; }")?;
    assert_eq!(s.rules[0].selector, Selector::Universal);
    assert_eq!(s.rules[0].selector.specificity(), 0);
    Ok(())
}

#[test]
fn parse_class_selector() -> AttractorResult<()> {
    let s = parse_stylesheet(".code { llm_model: claude-opus-4-6; }")?;
    assert_eq!(s.rules[0].selector, Selector::Class("code".into()));
    assert_eq!(s.rules[0].selector.specificity(), 1);
    Ok(())
}

#[test]
fn parse_id_selector() -> AttractorResult<()> {
    let s = parse_stylesheet("#review { reasoning_effort: high; }")?;
    assert_eq!(s.rules[0].selector, Selector::Id("review".into()));
    assert_eq!(s.rules[0].selector.specificity(), 2);
    Ok(())
}

// ===========================================================================
// Parser: declarations and rules
// ===========================================================================

#[test]
fn parse_multiple_declarations() -> AttractorResult<()> {
    let s = parse_stylesheet(
        "* { llm_model: gpt-4; llm_provider: openai; reasoning_effort: medium; }",
    )?;
    assert_eq!(s.rules[0].declarations.len(), 3);
    assert_eq!(s.rules[0].declarations[0].property, "llm_model");
    assert_eq!(s.rules[0].declarations[0].value, "gpt-4");
    assert_eq!(s.rules[0].declarations[1].property, "llm_provider");
    assert_eq!(s.rules[0].declarations[1].value, "openai");
    assert_eq!(s.rules[0].declarations[2].property, "reasoning_effort");
    assert_eq!(s.rules[0].declarations[2].value, "medium");
    Ok(())
}

#[test]
fn parse_multiple_rules() -> AttractorResult<()> {
    let s = parse_stylesheet("* { llm_model: a; } .code { llm_model: b; } #x { llm_model: c; }")?;
    assert_eq!(s.rules.len(), 3);
    assert_eq!(s.rules[0].selector.specificity(), 0);
    assert_eq!(s.rules[1].selector.specificity(), 1);
    assert_eq!(s.rules[2].selector.specificity(), 2);
    Ok(())
}

#[test]
fn parse_empty_stylesheet() -> AttractorResult<()> {
    let s = parse_stylesheet("")?;
    assert!(s.rules.is_empty());
    let s2 = parse_stylesheet("   ")?;
    assert!(s2.rules.is_empty());
    Ok(())
}

// ===========================================================================
// Parser: unknown property rejection (§8.2)
// ===========================================================================

#[test]
fn parse_rejects_unknown_property() {
    let result = parse_stylesheet("* { color: red; }");
    assert!(
        result.is_err(),
        "unknown property 'color' should be rejected"
    );
    let err = result.expect_err("should fail");
    assert!(err.to_string().contains("unknown property"));
}

#[test]
fn parse_rejects_unknown_among_valid() {
    let result = parse_stylesheet("* { llm_model: gpt-4; bad_prop: x; }");
    assert!(result.is_err());
}

// ===========================================================================
// Parser: value constraints (§8.4)
// ===========================================================================

#[test]
fn parse_rejects_invalid_reasoning_effort_value() {
    let result = parse_stylesheet("* { reasoning_effort: turbo; }");
    assert!(
        result.is_err(),
        "reasoning_effort: turbo should be rejected"
    );
}

#[test]
fn parse_accepts_all_valid_reasoning_effort_values() -> AttractorResult<()> {
    for val in &["low", "medium", "high"] {
        let input = format!("* {{ reasoning_effort: {val}; }}");
        let s = parse_stylesheet(&input)?;
        assert_eq!(s.rules[0].declarations[0].value, *val);
    }
    Ok(())
}

#[test]
fn parse_rejects_unquoted_value_with_spaces() {
    let result = parse_stylesheet("* { llm_model: has spaces; }");
    assert!(
        result.is_err(),
        "unquoted values with spaces should be rejected"
    );
}

// ===========================================================================
// Specificity
// ===========================================================================

#[test]
fn specificity_universal_lt_class_lt_id() {
    assert!(Selector::Universal.specificity() < Selector::Class("x".into()).specificity());
    assert!(Selector::Class("x".into()).specificity() < Selector::Id("x".into()).specificity());
}

#[test]
fn later_equal_specificity_wins() -> AttractorResult<()> {
    let mut g = stylesheet_pipeline();

    let stylesheet = parse_stylesheet("* { llm_model: first; } * { llm_model: second; }")?;
    apply_stylesheet(&mut g, &stylesheet)?;

    // "second" should win (later rule, same specificity)
    assert_eq!(
        g.get_node("plan").and_then(|n| n.get_str_attr("llm_model")),
        Some("second")
    );
    Ok(())
}

// ===========================================================================
// Application: universal applies to all
// ===========================================================================

#[test]
fn universal_applies_to_all() -> AttractorResult<()> {
    let mut g = stylesheet_pipeline();

    let stylesheet = parse_stylesheet("* { llm_model: gpt-4; llm_provider: openai; }")?;
    apply_stylesheet(&mut g, &stylesheet)?;

    // All non-start/exit nodes should get the model
    for node_id in &["plan", "implement", "critical_review"] {
        assert_eq!(
            g.get_node(node_id)
                .and_then(|n| n.get_str_attr("llm_model")),
            Some("gpt-4"),
            "node {node_id} should have llm_model"
        );
        assert_eq!(
            g.get_node(node_id)
                .and_then(|n| n.get_str_attr("llm_provider")),
            Some("openai"),
            "node {node_id} should have llm_provider"
        );
    }
    Ok(())
}

// ===========================================================================
// Application: class applies to matching nodes
// ===========================================================================

#[test]
fn class_applies_to_matching() -> AttractorResult<()> {
    let mut g = stylesheet_pipeline();

    let stylesheet =
        parse_stylesheet("* { llm_model: gpt-4; } .code { llm_model: claude-opus-4-6; }")?;
    apply_stylesheet(&mut g, &stylesheet)?;

    // "implement" has class=code → should get claude-opus-4-6
    assert_eq!(
        g.get_node("implement")
            .and_then(|n| n.get_str_attr("llm_model")),
        Some("claude-opus-4-6")
    );

    // "plan" has no class → should get universal gpt-4
    assert_eq!(
        g.get_node("plan").and_then(|n| n.get_str_attr("llm_model")),
        Some("gpt-4")
    );
    Ok(())
}

// ===========================================================================
// Application: ID applies to specific node
// ===========================================================================

#[test]
fn id_applies_to_specific_node() -> AttractorResult<()> {
    let mut g = stylesheet_pipeline();

    let stylesheet = parse_stylesheet(
        "* { llm_model: gpt-4; } #critical_review { llm_model: gpt-5; reasoning_effort: high; }",
    )?;
    apply_stylesheet(&mut g, &stylesheet)?;

    assert_eq!(
        g.get_node("critical_review")
            .and_then(|n| n.get_str_attr("llm_model")),
        Some("gpt-5")
    );
    assert_eq!(
        g.get_node("critical_review")
            .and_then(|n| n.get_str_attr("reasoning_effort")),
        Some("high")
    );

    // Other nodes should get universal
    assert_eq!(
        g.get_node("plan").and_then(|n| n.get_str_attr("llm_model")),
        Some("gpt-4")
    );
    Ok(())
}

// ===========================================================================
// Application: explicit node attribute has highest precedence
// ===========================================================================

#[test]
fn explicit_attr_overrides_stylesheet() -> AttractorResult<()> {
    let mut g = stylesheet_pipeline();

    // Set explicit llm_model on plan
    if let Some(node) = g.get_node_mut("plan") {
        node.attrs
            .insert("llm_model".into(), AttrValue::from("my-custom-model"));
    }

    let stylesheet = parse_stylesheet("* { llm_model: gpt-4; }")?;
    apply_stylesheet(&mut g, &stylesheet)?;

    // plan should keep its explicit value
    assert_eq!(
        g.get_node("plan").and_then(|n| n.get_str_attr("llm_model")),
        Some("my-custom-model")
    );

    // implement should get the stylesheet value
    assert_eq!(
        g.get_node("implement")
            .and_then(|n| n.get_str_attr("llm_model")),
        Some("gpt-4")
    );
    Ok(())
}

// ===========================================================================
// Full §8.6 example
// ===========================================================================

#[test]
fn full_spec_example() -> AttractorResult<()> {
    let mut g = stylesheet_pipeline();

    let stylesheet_str = r"
        * { llm_model: claude-sonnet-4-5; llm_provider: anthropic; }
        .code { llm_model: claude-opus-4-6; llm_provider: anthropic; }
        #critical_review { llm_model: gpt-5; llm_provider: openai; reasoning_effort: high; }
    ";

    let stylesheet = parse_stylesheet(stylesheet_str)?;
    apply_stylesheet(&mut g, &stylesheet)?;

    // plan: universal rule → claude-sonnet-4-5 / anthropic
    assert_eq!(
        g.get_node("plan").and_then(|n| n.get_str_attr("llm_model")),
        Some("claude-sonnet-4-5")
    );
    assert_eq!(
        g.get_node("plan")
            .and_then(|n| n.get_str_attr("llm_provider")),
        Some("anthropic")
    );

    // implement: class .code → claude-opus-4-6 / anthropic
    assert_eq!(
        g.get_node("implement")
            .and_then(|n| n.get_str_attr("llm_model")),
        Some("claude-opus-4-6")
    );
    assert_eq!(
        g.get_node("implement")
            .and_then(|n| n.get_str_attr("llm_provider")),
        Some("anthropic")
    );

    // critical_review: ID #critical_review → gpt-5 / openai / high
    assert_eq!(
        g.get_node("critical_review")
            .and_then(|n| n.get_str_attr("llm_model")),
        Some("gpt-5")
    );
    assert_eq!(
        g.get_node("critical_review")
            .and_then(|n| n.get_str_attr("llm_provider")),
        Some("openai")
    );
    assert_eq!(
        g.get_node("critical_review")
            .and_then(|n| n.get_str_attr("reasoning_effort")),
        Some("high")
    );
    Ok(())
}

// ===========================================================================
// Graph-level default as fallback
// ===========================================================================

#[test]
fn graph_level_default_fallback_via_apply() -> AttractorResult<()> {
    // §8.5: graph-level defaults should apply even with an empty stylesheet
    let mut g = stylesheet_pipeline();
    g.graph_attrs
        .insert("llm_model".into(), AttrValue::from("graph-default"));

    let stylesheet = parse_stylesheet("")?;
    apply_stylesheet(&mut g, &stylesheet)?;

    // All nodes without explicit llm_model should get the graph-level default
    assert_eq!(
        g.get_node("plan").and_then(|n| n.get_str_attr("llm_model")),
        Some("graph-default")
    );
    assert_eq!(
        g.get_node("implement")
            .and_then(|n| n.get_str_attr("llm_model")),
        Some("graph-default")
    );
    Ok(())
}

#[test]
fn parse_and_apply_empty_stylesheet_applies_graph_defaults() -> AttractorResult<()> {
    // §8.5: even with empty/absent model_stylesheet, graph-level defaults apply
    let mut g = stylesheet_pipeline();
    g.graph_attrs
        .insert("llm_model".into(), AttrValue::from("graph-default"));
    g.graph_attrs
        .insert("model_stylesheet".into(), AttrValue::from(""));

    parse_and_apply_stylesheet(&mut g)?;

    assert_eq!(
        g.get_node("plan").and_then(|n| n.get_str_attr("llm_model")),
        Some("graph-default")
    );
    Ok(())
}

#[test]
fn parse_and_apply_stylesheet_with_graph_default() -> AttractorResult<()> {
    let mut g = stylesheet_pipeline();
    g.graph_attrs
        .insert("llm_model".into(), AttrValue::from("graph-default"));
    g.graph_attrs.insert(
        "model_stylesheet".into(),
        AttrValue::from("#critical_review { llm_model: special; }"),
    );

    parse_and_apply_stylesheet(&mut g)?;

    // critical_review gets the ID rule value
    assert_eq!(
        g.get_node("critical_review")
            .and_then(|n| n.get_str_attr("llm_model")),
        Some("special")
    );

    // plan gets graph-level default
    assert_eq!(
        g.get_node("plan").and_then(|n| n.get_str_attr("llm_model")),
        Some("graph-default")
    );
    Ok(())
}

// ===========================================================================
// Subgraph class derivation + stylesheet matching
// ===========================================================================

#[test]
fn subgraph_class_matching() -> AttractorResult<()> {
    let mut g = stylesheet_pipeline();

    // Add "code" class to "plan" to simulate subgraph derivation
    if let Some(node) = g.get_node_mut("plan") {
        node.attrs.insert("class".into(), AttrValue::from("code"));
    }

    let stylesheet = parse_stylesheet(".code { llm_model: opus; }")?;
    apply_stylesheet(&mut g, &stylesheet)?;

    // Both plan and implement have class=code
    assert_eq!(
        g.get_node("plan").and_then(|n| n.get_str_attr("llm_model")),
        Some("opus")
    );
    assert_eq!(
        g.get_node("implement")
            .and_then(|n| n.get_str_attr("llm_model")),
        Some("opus")
    );
    Ok(())
}

// ===========================================================================
// Non-string model_stylesheet → error
// ===========================================================================

#[test]
fn parse_and_apply_non_string_stylesheet_errors() {
    let mut g = stylesheet_pipeline();
    g.graph_attrs
        .insert("model_stylesheet".into(), AttrValue::Integer(42));

    let result = parse_and_apply_stylesheet(&mut g);
    assert!(result.is_err(), "non-string model_stylesheet should error");
    let err = format!("{}", result.expect_err("should fail"));
    assert!(err.contains("must be a string"), "{err}");
}

// ===========================================================================
// No model_stylesheet attr → no-op
// ===========================================================================

#[test]
fn no_stylesheet_attr_is_noop() -> AttractorResult<()> {
    let mut g = stylesheet_pipeline();
    parse_and_apply_stylesheet(&mut g)?;

    // No attributes should be added
    assert!(
        g.get_node("plan")
            .and_then(|n| n.get_str_attr("llm_model"))
            .is_none()
    );
    Ok(())
}

// ===========================================================================
// Transform integration
// ===========================================================================

#[test]
fn stylesheet_transform_applied_via_registry() -> AttractorResult<()> {
    use stencila_attractor::transform::TransformRegistry;

    let mut g = stylesheet_pipeline();
    g.graph_attrs.insert(
        "model_stylesheet".into(),
        AttrValue::from("* { llm_model: test-model; }"),
    );

    let registry = TransformRegistry::with_defaults();
    registry.apply_all(&mut g)?;

    assert_eq!(
        g.get_node("plan").and_then(|n| n.get_str_attr("llm_model")),
        Some("test-model")
    );
    Ok(())
}

// ===========================================================================
// Programmatic stylesheet construction
// ===========================================================================

#[test]
fn programmatic_stylesheet() -> AttractorResult<()> {
    let mut g = stylesheet_pipeline();

    let stylesheet = ParsedStylesheet {
        rules: vec![
            StylesheetRule {
                selector: Selector::Universal,
                declarations: vec![Declaration {
                    property: "llm_model".into(),
                    value: "default-model".into(),
                }],
            },
            StylesheetRule {
                selector: Selector::Id("plan".into()),
                declarations: vec![Declaration {
                    property: "llm_model".into(),
                    value: "plan-model".into(),
                }],
            },
        ],
    };

    apply_stylesheet(&mut g, &stylesheet)?;

    assert_eq!(
        g.get_node("plan").and_then(|n| n.get_str_attr("llm_model")),
        Some("plan-model")
    );
    assert_eq!(
        g.get_node("implement")
            .and_then(|n| n.get_str_attr("llm_model")),
        Some("default-model")
    );
    Ok(())
}
