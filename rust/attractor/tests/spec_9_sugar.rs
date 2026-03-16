#![allow(clippy::unwrap_used)]
//! Tests for node sugar transform.

use stencila_attractor::error::AttractorResult;
use stencila_attractor::parser::parse_dot;
use stencila_attractor::transform::Transform;
use stencila_attractor::transforms::NodeSugarTransform;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn apply_sugar(dot: &str) -> AttractorResult<stencila_attractor::graph::Graph> {
    let mut graph = parse_dot(dot)?;
    NodeSugarTransform.apply(&mut graph)?;
    Ok(graph)
}

#[test]
fn workflow_attribute_implies_workflow_type() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> Implement -> End
            Implement [workflow="software-implement-iterative", goal="$last_output"]
        }
        "#,
    )?;
    assert_eq!(
        g.get_node("Implement").and_then(|n| n.get_str_attr("type")),
        Some("workflow")
    );
    assert_eq!(
        g.get_node("Implement")
            .and_then(|n| n.get_str_attr("workflow")),
        Some("software-implement-iterative")
    );
    Ok(())
}

fn node_shape<'a>(graph: &'a stencila_attractor::graph::Graph, id: &str) -> &'a str {
    graph.get_node(id).map(|n| n.shape()).unwrap_or("MISSING")
}

// ===========================================================================
// Node ID prefix aliases
// ===========================================================================

#[test]
fn fanout_id_implies_component_shape() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> FanOut -> End
        }
        "#,
    )?;
    assert_eq!(node_shape(&g, "FanOut"), "component");
    Ok(())
}

#[test]
fn fanout_suffixed_id_implies_component_shape() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> FanOutSearch -> End
        }
        "#,
    )?;
    assert_eq!(node_shape(&g, "FanOutSearch"), "component");
    Ok(())
}

#[test]
fn fanout_lowercase_variant() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> FanoutTasks -> End
        }
        "#,
    )?;
    assert_eq!(node_shape(&g, "FanoutTasks"), "component");
    Ok(())
}

#[test]
fn fanin_id_implies_tripleoctagon_shape() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> FanIn -> End
        }
        "#,
    )?;
    assert_eq!(node_shape(&g, "FanIn"), "tripleoctagon");
    Ok(())
}

#[test]
fn fanin_suffixed_id_implies_tripleoctagon_shape() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> FanInResults -> End
        }
        "#,
    )?;
    assert_eq!(node_shape(&g, "FanInResults"), "tripleoctagon");
    Ok(())
}

#[test]
fn fanin_lowercase_variant() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> FaninCollect -> End
        }
        "#,
    )?;
    assert_eq!(node_shape(&g, "FaninCollect"), "tripleoctagon");
    Ok(())
}

// ===========================================================================
// Explicit shape is never overridden
// ===========================================================================

#[test]
fn explicit_shape_not_overridden_by_id() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> FanOut -> End
            FanOut [shape=box]
        }
        "#,
    )?;
    assert_eq!(node_shape(&g, "FanOut"), "box");
    Ok(())
}

// ===========================================================================
// Start/End/Fail IDs get canonical shapes
// ===========================================================================

#[test]
fn start_id_gets_mdiamond() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> Task -> End
        }
        "#,
    )?;
    assert_eq!(node_shape(&g, "Start"), "Mdiamond");
    Ok(())
}

#[test]
fn lowercase_start_id_gets_mdiamond() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            start -> Task -> End
        }
        "#,
    )?;
    assert_eq!(node_shape(&g, "start"), "Mdiamond");
    Ok(())
}

#[test]
fn end_id_gets_msquare() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> Task -> End
        }
        "#,
    )?;
    assert_eq!(node_shape(&g, "End"), "Msquare");
    Ok(())
}

#[test]
fn exit_id_gets_msquare() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> Task -> Exit
        }
        "#,
    )?;
    assert_eq!(node_shape(&g, "Exit"), "Msquare");
    Ok(())
}

#[test]
fn fail_id_gets_invtriangle() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> Fail
        }
        "#,
    )?;
    assert_eq!(node_shape(&g, "Fail"), "invtriangle");
    Ok(())
}

#[test]
fn start_explicit_shape_not_overridden() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> End
            Start [shape=box]
        }
        "#,
    )?;
    assert_eq!(node_shape(&g, "Start"), "box");
    Ok(())
}

// ===========================================================================
// Property-based sugar: ask
// ===========================================================================

#[test]
fn ask_attr_implies_hexagon_and_label() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> Gate -> End
            Gate [ask="Do you approve?"]
        }
        "#,
    )?;
    let node = g.get_node("Gate").unwrap();
    assert_eq!(node.shape(), "hexagon");
    assert_eq!(node.label(), "Do you approve?");
    // The `ask` attribute itself should be removed
    assert!(node.get_attr("ask").is_none());
    Ok(())
}

#[test]
fn ask_does_not_override_explicit_shape() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> Gate -> End
            Gate [ask="Approve?", shape=diamond]
        }
        "#,
    )?;
    let node = g.get_node("Gate").unwrap();
    assert_eq!(node.shape(), "diamond");
    assert_eq!(node.label(), "Approve?");
    Ok(())
}

#[test]
fn ask_does_not_override_explicit_label() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> Gate -> End
            Gate [ask="Approve?", label="Custom Label"]
        }
        "#,
    )?;
    let node = g.get_node("Gate").unwrap();
    assert_eq!(node.label(), "Custom Label");
    Ok(())
}

// ===========================================================================
// Property-based sugar: shell
// ===========================================================================

#[test]
fn shell_attr_implies_parallelogram_and_shell_command() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> Build -> End
            Build [shell="cargo test"]
        }
        "#,
    )?;
    let node = g.get_node("Build").unwrap();
    assert_eq!(node.shape(), "parallelogram");
    assert_eq!(node.get_str_attr("shell_command"), Some("cargo test"));
    assert!(node.get_attr("shell").is_none());
    Ok(())
}

#[test]
fn shell_does_not_override_explicit_shape() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> Build -> End
            Build [shell="cargo test", shape=box]
        }
        "#,
    )?;
    let node = g.get_node("Build").unwrap();
    assert_eq!(node.shape(), "box");
    assert_eq!(node.get_str_attr("shell_command"), Some("cargo test"));
    Ok(())
}

#[test]
fn shell_does_not_override_explicit_shell_command() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> Build -> End
            Build [shell="cargo test", shell_command="make"]
        }
        "#,
    )?;
    let node = g.get_node("Build").unwrap();
    assert_eq!(node.get_str_attr("shell_command"), Some("make"));
    Ok(())
}

#[test]
fn shell_handler_type_resolution() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> Build -> End
            Build [shell="make"]
        }
        "#,
    )?;
    let node = g.get_node("Build").unwrap();
    assert_eq!(node.handler_type(), "shell");
    Ok(())
}

// ===========================================================================
// Property-based sugar: branch
// ===========================================================================

#[test]
fn branch_attr_implies_diamond_and_label() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> Decision -> End
            Decision [branch="Is quality OK?"]
        }
        "#,
    )?;
    let node = g.get_node("Decision").unwrap();
    assert_eq!(node.shape(), "diamond");
    assert_eq!(node.label(), "Is quality OK?");
    assert!(node.get_attr("branch").is_none());
    Ok(())
}

#[test]
fn branch_does_not_override_explicit_shape() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> Decision -> End
            Decision [branch="Is quality OK?", shape=box]
        }
        "#,
    )?;
    let node = g.get_node("Decision").unwrap();
    assert_eq!(node.shape(), "box");
    assert_eq!(node.label(), "Is quality OK?");
    Ok(())
}

// ===========================================================================
// Ordinary nodes are not affected
// ===========================================================================

#[test]
fn ordinary_node_not_affected() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> Analyze -> Summarize -> End
            Analyze [prompt="Analyze the data"]
            Summarize [prompt="Summarize findings"]
        }
        "#,
    )?;
    assert_eq!(node_shape(&g, "Analyze"), "box");
    assert_eq!(node_shape(&g, "Summarize"), "box");
    Ok(())
}

// ===========================================================================
// Combined: sugar form of the combined example from docs
// ===========================================================================

#[test]
fn combined_sugar_example() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph ResearchWorkflow {
            Start -> Search -> Screen -> Analyze -> CheckQuality
            CheckQuality -> Review    [label="Pass", condition="outcome=success"]
            CheckQuality -> Analyze   [label="Fail", condition="outcome!=success"]
            Review -> Publish         [label="[A] Approve"]
            Review -> Search          [label="[R] Revise"]
            Publish -> End

            Search       [prompt="Search databases for papers"]
            Screen       [prompt="Screen papers for relevance"]
            Analyze      [prompt="Extract and synthesize key findings"]
            CheckQuality [branch="Analysis meets quality criteria?"]
            Review       [ask="Review the systematic review draft"]
            Publish      [prompt="Format the final review for publication"]
        }
        "#,
    )?;

    // CheckQuality inferred as diamond via `branch` shortcut
    assert_eq!(node_shape(&g, "CheckQuality"), "diamond");
    // Review inferred as hexagon via `ask` shortcut
    assert_eq!(node_shape(&g, "Review"), "hexagon");
    // Ordinary LLM nodes remain box
    assert_eq!(node_shape(&g, "Search"), "box");
    assert_eq!(node_shape(&g, "Screen"), "box");
    assert_eq!(node_shape(&g, "Analyze"), "box");
    assert_eq!(node_shape(&g, "Publish"), "box");
    Ok(())
}

// ===========================================================================
// Integration: sugar works with full parse_dot → transform pipeline
// ===========================================================================

#[test]
fn sugar_then_handler_type_resolution() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> FanOut -> A -> B -> End
            FanOut -> A
            FanOut -> B
            A [prompt="Do A"]
            B [prompt="Do B"]
        }
        "#,
    )?;

    let fanout = g.get_node("FanOut").unwrap();
    assert_eq!(fanout.handler_type(), "parallel");
    Ok(())
}

#[test]
fn review_handler_type_resolution() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> Review -> End
            Review [ask="All good?"]
        }
        "#,
    )?;

    let review = g.get_node("Review").unwrap();
    assert_eq!(review.handler_type(), "wait.human");
    Ok(())
}

#[test]
fn branch_handler_type_resolution() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> CheckValid -> End
            CheckValid [branch="Is it valid?"]
        }
        "#,
    )?;

    let node = g.get_node("CheckValid").unwrap();
    assert_eq!(node.handler_type(), "conditional");
    Ok(())
}

#[test]
fn cmd_handler_type_resolution() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> Build -> End
            Build [shell="make"]
        }
        "#,
    )?;

    let node = g.get_node("Build").unwrap();
    assert_eq!(node.handler_type(), "shell");
    Ok(())
}

// ===========================================================================
// prompt/agent override ID-based inference
// ===========================================================================

#[test]
fn agent_on_fanout_id_keeps_codergen() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> FanOutAnalysis -> End
            FanOutAnalysis [agent="deep-analyst"]
        }
        "#,
    )?;
    assert_eq!(node_shape(&g, "FanOutAnalysis"), "box");
    assert_eq!(
        g.get_node("FanOutAnalysis").unwrap().handler_type(),
        "codergen"
    );
    Ok(())
}

#[test]
fn agent_on_fanin_id_keeps_codergen() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> FanInResults -> End
            FanInResults [agent="aggregator"]
        }
        "#,
    )?;
    assert_eq!(node_shape(&g, "FanInResults"), "box");
    assert_eq!(
        g.get_node("FanInResults").unwrap().handler_type(),
        "codergen"
    );
    Ok(())
}

#[test]
fn agent_trust_level_on_node_keeps_codergen() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> Risky -> End
            Risky [agent.trust-level="high", agent.max-turns="20"]
        }
        "#,
    )?;
    assert_eq!(node_shape(&g, "Risky"), "box");
    assert_eq!(g.get_node("Risky").unwrap().handler_type(), "codergen");
    Ok(())
}

#[test]
fn prompt_does_not_override_explicit_shape() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> Review -> End
            Review [prompt="Review the data", shape=hexagon]
        }
        "#,
    )?;
    // Explicit shape wins over prompt
    assert_eq!(node_shape(&g, "Review"), "hexagon");
    Ok(())
}

#[test]
fn prompt_does_not_affect_start_end() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> Task -> End
        }
        "#,
    )?;
    assert_eq!(node_shape(&g, "Start"), "Mdiamond");
    assert_eq!(node_shape(&g, "End"), "Msquare");
    Ok(())
}

#[test]
fn ask_beats_prompt_precedence() -> AttractorResult<()> {
    // Property shortcut `ask` takes priority over `prompt` presence
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> Gate -> End
            Gate [ask="Approve?", prompt="ignored"]
        }
        "#,
    )?;
    assert_eq!(node_shape(&g, "Gate"), "hexagon");
    Ok(())
}

// ===========================================================================
// Finding 1: structural IDs are exempt from prompt/agent override
// ===========================================================================

#[test]
fn start_with_prompt_keeps_structural_shape() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> End
            Start [prompt="Initialize the workflow"]
        }
        "#,
    )?;
    // Start is a reserved structural ID — prompt does not make it codergen
    assert_eq!(node_shape(&g, "Start"), "Mdiamond");
    assert_eq!(g.get_node("Start").unwrap().handler_type(), "start");
    Ok(())
}

#[test]
fn end_with_agent_keeps_structural_shape() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> End
            End [agent="finalizer"]
        }
        "#,
    )?;
    assert_eq!(node_shape(&g, "End"), "Msquare");
    assert_eq!(g.get_node("End").unwrap().handler_type(), "exit");
    Ok(())
}

// ===========================================================================
// Finding 2: all sugar keys are drained even when another wins
// ===========================================================================

#[test]
fn coexisting_ask_and_branch_drains_both() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> Gate -> End
            Gate [ask="Approve?", branch="leftover"]
        }
        "#,
    )?;
    let node = g.get_node("Gate").unwrap();
    // `ask` wins
    assert_eq!(node.shape(), "hexagon");
    assert_eq!(node.label(), "Approve?");
    // `branch` was drained — not leaked
    assert!(node.get_attr("branch").is_none());
    Ok(())
}

#[test]
fn coexisting_ask_and_cmd_drains_both() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> Node1 -> End
            Node1 [ask="Ready?", shell="echo yes"]
        }
        "#,
    )?;
    let node = g.get_node("Node1").unwrap();
    // `ask` wins (highest precedence)
    assert_eq!(node.shape(), "hexagon");
    assert_eq!(node.label(), "Ready?");
    // `shell` was drained
    assert!(node.get_attr("shell").is_none());
    // `shell` value was NOT applied as shell_command since ask won
    assert!(node.get_attr("shell_command").is_none());
    Ok(())
}

#[test]
fn all_sugar_keys_drained() -> AttractorResult<()> {
    let g = apply_sugar(
        r#"
        digraph T {
            Start -> Node1 -> End
            Node1 [ask="Q?", shell="s", branch="b"]
        }
        "#,
    )?;
    let node = g.get_node("Node1").unwrap();
    assert!(node.get_attr("ask").is_none());
    assert!(node.get_attr("shell").is_none());
    assert!(node.get_attr("branch").is_none());
    Ok(())
}
