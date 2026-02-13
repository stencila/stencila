use stencila_attractor::{AttrValue, AttractorError, Duration, Graph, Node, parse_dot};

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Parse DOT and get a node by ID, or return a descriptive error.
fn parse_and_get_node(
    dot: &str,
    node_id: &str,
) -> Result<(Graph, Node), Box<dyn std::error::Error>> {
    let g = parse_dot(dot)?;
    let node = g
        .get_node(node_id)
        .ok_or_else(|| format!("node '{node_id}' not found"))?
        .clone();
    Ok((g, node))
}

// ---------------------------------------------------------------------------
// Basic parsing (~12)
// ---------------------------------------------------------------------------

#[test]
fn parse_empty_digraph() -> TestResult {
    let g = parse_dot("digraph G {}")?;
    assert_eq!(g.name, "G");
    assert!(g.nodes.is_empty());
    assert!(g.edges.is_empty());
    Ok(())
}

#[test]
fn parse_single_node() -> TestResult {
    let g = parse_dot("digraph G { A; }")?;
    assert_eq!(g.nodes.len(), 1);
    assert!(g.get_node("A").is_some());
    Ok(())
}

#[test]
fn parse_node_with_attrs() -> TestResult {
    let (_, node) = parse_and_get_node(
        r#"digraph G { A [shape="diamond", label="Check", prompt="Do it"]; }"#,
        "A",
    )?;
    assert_eq!(node.shape(), "diamond");
    assert_eq!(node.label(), "Check");
    assert_eq!(node.get_str_attr("prompt"), Some("Do it"));
    Ok(())
}

#[test]
fn parse_single_edge() -> TestResult {
    let g = parse_dot("digraph G { A -> B; }")?;
    assert_eq!(g.edges.len(), 1);
    assert_eq!(g.edges[0].from, "A");
    assert_eq!(g.edges[0].to, "B");
    // Both nodes should be auto-created
    assert!(g.get_node("A").is_some());
    assert!(g.get_node("B").is_some());
    Ok(())
}

#[test]
fn parse_edge_with_attrs() -> TestResult {
    let g = parse_dot(r#"digraph G { A -> B [label="next", weight=5]; }"#)?;
    assert_eq!(g.edges.len(), 1);
    assert_eq!(g.edges[0].label(), "next");
    assert_eq!(g.edges[0].weight(), 5);
    Ok(())
}

#[test]
fn parse_minimal_linear() -> TestResult {
    // §2.13 simple linear example
    let dot = r#"
        digraph Pipeline {
            graph [goal="Run tests and report"]
            start [shape=Mdiamond]
            run_tests [label="Run Tests", prompt="Execute the test suite"]
            report [label="Report", prompt="Summarize results"]
            exit [shape=Msquare]

            start -> run_tests -> report -> exit
        }
    "#;
    let g = parse_dot(dot)?;
    assert_eq!(g.name, "Pipeline");
    assert_eq!(g.nodes.len(), 4);
    assert_eq!(g.edges.len(), 3);

    let start = g.find_start_node()?;
    assert_eq!(start.id, "start");
    let exit_node = g.find_exit_node()?;
    assert_eq!(exit_node.id, "exit");
    Ok(())
}

#[test]
fn parse_minimal_branching() -> TestResult {
    // §2.13 branching example
    let dot = r#"
        digraph Branching {
            start [shape=Mdiamond]
            check [shape=diamond, label="Check"]
            pass [label="Pass"]
            fail_node [label="Fail"]
            exit [shape=Msquare]

            start -> check
            check -> pass [label="yes"]
            check -> fail_node [label="no"]
            pass -> exit
            fail_node -> exit
        }
    "#;
    let g = parse_dot(dot)?;
    assert_eq!(g.nodes.len(), 5);
    assert_eq!(g.edges.len(), 5);

    let check = g.get_node("check").ok_or("check not found")?;
    assert_eq!(check.handler_type(), "conditional");

    // The "yes" edge
    let check_edges = g.outgoing_edges("check");
    assert_eq!(check_edges.len(), 2);
    Ok(())
}

#[test]
fn parse_minimal_human_gate() -> TestResult {
    let dot = r#"
        digraph HumanGate {
            start [shape=Mdiamond]
            review [shape=hexagon, label="Human Review"]
            exit [shape=Msquare]

            start -> review -> exit
        }
    "#;
    let g = parse_dot(dot)?;
    let review = g.get_node("review").ok_or("review not found")?;
    assert_eq!(review.handler_type(), "wait.human");
    Ok(())
}

#[test]
fn parse_graph_level_attrs() -> TestResult {
    let dot = r#"
        digraph G {
            graph [goal="Build something", label="My Pipeline"]
        }
    "#;
    let g = parse_dot(dot)?;
    assert_eq!(
        g.get_graph_attr("goal").and_then(AttrValue::as_str),
        Some("Build something")
    );
    assert_eq!(
        g.get_graph_attr("label").and_then(AttrValue::as_str),
        Some("My Pipeline")
    );
    Ok(())
}

#[test]
fn parse_top_level_attr_decl() -> TestResult {
    let dot = r#"
        digraph G {
            rankdir="LR"
        }
    "#;
    let g = parse_dot(dot)?;
    assert_eq!(
        g.get_graph_attr("rankdir").and_then(AttrValue::as_str),
        Some("LR")
    );
    Ok(())
}

#[test]
fn parse_nodes_from_edges() -> TestResult {
    // Nodes mentioned only in edges should be auto-created
    let g = parse_dot("digraph G { X -> Y -> Z; }")?;
    assert_eq!(g.nodes.len(), 3);
    assert!(g.get_node("X").is_some());
    assert!(g.get_node("Y").is_some());
    assert!(g.get_node("Z").is_some());
    Ok(())
}

#[test]
fn parse_node_reuse() -> TestResult {
    let dot = r#"
        digraph G {
            A [label="Alpha"]
            B
            C
            A -> B
            A -> C
        }
    "#;
    let g = parse_dot(dot)?;
    assert_eq!(g.nodes.len(), 3);
    assert_eq!(g.edges.len(), 2);
    let a = g.get_node("A").ok_or("A not found")?;
    assert_eq!(a.label(), "Alpha");
    Ok(())
}

// ---------------------------------------------------------------------------
// Value parsing (~7)
// ---------------------------------------------------------------------------

#[test]
fn value_quoted_string() -> TestResult {
    let (_, a) = parse_and_get_node(r#"digraph G { A [label="hello world"]; }"#, "A")?;
    assert_eq!(a.get_str_attr("label"), Some("hello world"));
    Ok(())
}

#[test]
fn value_string_escapes() -> TestResult {
    let dot = r#"digraph G { A [prompt="line1\nline2\ttab\\slash\"quote"]; }"#;
    let (_, a) = parse_and_get_node(dot, "A")?;
    let prompt = a.get_str_attr("prompt").ok_or("no prompt")?;
    assert!(prompt.contains('\n'));
    assert!(prompt.contains('\t'));
    assert!(prompt.contains('\\'));
    assert!(prompt.contains('"'));
    Ok(())
}

#[test]
fn value_integer() -> TestResult {
    let dot = r#"digraph G { A -> B [weight=42]; A -> C [weight=-7]; A -> D [weight=0]; }"#;
    let g = parse_dot(dot)?;
    assert_eq!(g.edges[0].weight(), 42);
    assert_eq!(g.edges[1].weight(), -7);
    assert_eq!(g.edges[2].weight(), 0);
    Ok(())
}

#[test]
fn value_float() -> TestResult {
    let dot = r#"digraph G { A [temperature=0.7]; B [score=-3.125]; }"#;
    let g = parse_dot(dot)?;
    let a = g.get_node("A").ok_or("A not found")?;
    let temp = a
        .get_attr("temperature")
        .and_then(AttrValue::as_f64)
        .ok_or("no temp")?;
    assert!((temp - 0.7).abs() < f64::EPSILON);
    let b = g.get_node("B").ok_or("B not found")?;
    let score = b
        .get_attr("score")
        .and_then(AttrValue::as_f64)
        .ok_or("no score")?;
    assert!((score - (-3.125)).abs() < f64::EPSILON);
    Ok(())
}

#[test]
fn value_boolean() -> TestResult {
    let dot = r#"digraph G { A [enabled=true, verbose=false]; }"#;
    let g = parse_dot(dot)?;
    let a = g.get_node("A").ok_or("A not found")?;
    assert_eq!(
        a.get_attr("enabled").and_then(AttrValue::as_bool),
        Some(true)
    );
    assert_eq!(
        a.get_attr("verbose").and_then(AttrValue::as_bool),
        Some(false)
    );
    Ok(())
}

#[test]
fn value_duration() -> TestResult {
    let dot = r#"digraph G { A [timeout=900s, delay=15m, fast=250ms]; }"#;
    let g = parse_dot(dot)?;
    let a = g.get_node("A").ok_or("A not found")?;

    let timeout = a
        .get_attr("timeout")
        .and_then(AttrValue::as_duration)
        .ok_or("no timeout")?;
    assert_eq!(timeout.inner().as_secs(), 900);

    let delay = a
        .get_attr("delay")
        .and_then(AttrValue::as_duration)
        .ok_or("no delay")?;
    assert_eq!(delay.inner().as_secs(), 900); // 15m = 900s

    let fast = a
        .get_attr("fast")
        .and_then(AttrValue::as_duration)
        .ok_or("no fast")?;
    assert_eq!(fast.inner().as_millis(), 250);
    Ok(())
}

#[test]
fn value_empty_string() -> TestResult {
    let (_, a) = parse_and_get_node(r#"digraph G { A [label=""]; }"#, "A")?;
    assert_eq!(a.get_str_attr("label"), Some(""));
    Ok(())
}

// ---------------------------------------------------------------------------
// Attribute handling (~8)
// ---------------------------------------------------------------------------

#[test]
fn attr_block_multi_attrs() -> TestResult {
    let dot = r#"digraph G { A [shape="box", label="Build", prompt="Do it", weight=10]; }"#;
    let g = parse_dot(dot)?;
    let a = g.get_node("A").ok_or("A not found")?;
    assert_eq!(a.attrs.len(), 4);
    Ok(())
}

#[test]
fn attr_block_empty() -> TestResult {
    let g = parse_dot("digraph G { A []; }")?;
    let a = g.get_node("A").ok_or("A not found")?;
    assert!(a.attrs.is_empty());
    Ok(())
}

#[test]
fn node_defaults_apply() -> TestResult {
    let dot = r#"
        digraph G {
            node [shape="box"]
            A
            B
        }
    "#;
    let g = parse_dot(dot)?;
    let a = g.get_node("A").ok_or("A not found")?;
    assert_eq!(a.shape(), "box");
    let b = g.get_node("B").ok_or("B not found")?;
    assert_eq!(b.shape(), "box");
    Ok(())
}

#[test]
fn edge_defaults_apply() -> TestResult {
    let dot = r#"
        digraph G {
            edge [weight=5]
            A -> B
            C -> D
        }
    "#;
    let g = parse_dot(dot)?;
    assert_eq!(g.edges[0].weight(), 5);
    assert_eq!(g.edges[1].weight(), 5);
    Ok(())
}

#[test]
fn node_explicit_overrides_default() -> TestResult {
    let dot = r#"
        digraph G {
            node [shape="box"]
            A [shape="diamond"]
            B
        }
    "#;
    let g = parse_dot(dot)?;
    let a = g.get_node("A").ok_or("A not found")?;
    assert_eq!(a.shape(), "diamond");
    let b = g.get_node("B").ok_or("B not found")?;
    assert_eq!(b.shape(), "box");
    Ok(())
}

#[test]
fn subgraph_scoped_defaults() -> TestResult {
    let dot = r#"
        digraph G {
            node [shape="box"]
            A
            subgraph cluster_inner {
                node [shape="diamond"]
                B
            }
            C
        }
    "#;
    let g = parse_dot(dot)?;
    let a = g.get_node("A").ok_or("A not found")?;
    assert_eq!(a.shape(), "box");
    let b = g.get_node("B").ok_or("B not found")?;
    assert_eq!(b.shape(), "diamond");
    // C should have the outer default, not the subgraph's
    let c = g.get_node("C").ok_or("C not found")?;
    assert_eq!(c.shape(), "box");
    Ok(())
}

#[test]
fn subgraph_class_derivation() -> TestResult {
    let dot = r#"
        digraph G {
            subgraph cluster_loop {
                graph [label="Loop A"]
                X
                Y
            }
        }
    "#;
    let g = parse_dot(dot)?;
    let x = g.get_node("X").ok_or("X not found")?;
    assert_eq!(x.get_str_attr("class"), Some("loop-a"));
    let y = g.get_node("Y").ok_or("Y not found")?;
    assert_eq!(y.get_str_attr("class"), Some("loop-a"));
    Ok(())
}

#[test]
fn multi_line_attr_block() -> TestResult {
    let dot = r#"
        digraph G {
            A [
                shape="box",
                label="Build",
                prompt="Do the thing"
            ];
        }
    "#;
    let g = parse_dot(dot)?;
    let a = g.get_node("A").ok_or("A not found")?;
    assert_eq!(a.attrs.len(), 3);
    assert_eq!(a.shape(), "box");
    Ok(())
}

// ---------------------------------------------------------------------------
// Edge features (~5)
// ---------------------------------------------------------------------------

#[test]
fn chained_edges() -> TestResult {
    let dot = r#"digraph G { A -> B -> C [label="x"]; }"#;
    let g = parse_dot(dot)?;
    assert_eq!(g.edges.len(), 2);
    assert_eq!(g.edges[0].from, "A");
    assert_eq!(g.edges[0].to, "B");
    assert_eq!(g.edges[0].label(), "x");
    assert_eq!(g.edges[1].from, "B");
    assert_eq!(g.edges[1].to, "C");
    assert_eq!(g.edges[1].label(), "x");
    Ok(())
}

#[test]
fn chained_edges_three_segments() -> TestResult {
    let g = parse_dot("digraph G { A -> B -> C -> D; }")?;
    assert_eq!(g.edges.len(), 3);
    assert_eq!(g.edges[0].from, "A");
    assert_eq!(g.edges[0].to, "B");
    assert_eq!(g.edges[1].from, "B");
    assert_eq!(g.edges[1].to, "C");
    assert_eq!(g.edges[2].from, "C");
    assert_eq!(g.edges[2].to, "D");
    Ok(())
}

#[test]
fn multiple_edges_same_nodes() -> TestResult {
    let dot = r#"
        digraph G {
            A -> B [label="first"]
            A -> B [label="second"]
        }
    "#;
    let g = parse_dot(dot)?;
    assert_eq!(g.edges.len(), 2);
    assert_eq!(g.edges[0].label(), "first");
    assert_eq!(g.edges[1].label(), "second");
    Ok(())
}

#[test]
fn edge_all_attr_types() -> TestResult {
    let dot = r#"
        digraph G {
            A -> B [label="next", condition="status = success", weight=10, fidelity="compact", thread_id="t1", loop_restart=true]
        }
    "#;
    let g = parse_dot(dot)?;
    let edge = &g.edges[0];
    assert_eq!(edge.label(), "next");
    assert_eq!(edge.condition(), "status = success");
    assert_eq!(edge.weight(), 10);
    assert_eq!(
        edge.get_attr("fidelity").and_then(AttrValue::as_str),
        Some("compact")
    );
    assert_eq!(
        edge.get_attr("thread_id").and_then(AttrValue::as_str),
        Some("t1")
    );
    assert_eq!(
        edge.get_attr("loop_restart").and_then(AttrValue::as_bool),
        Some(true)
    );
    Ok(())
}

#[test]
fn chained_edge_with_defaults() -> TestResult {
    let dot = r#"
        digraph G {
            edge [weight=3]
            A -> B -> C
        }
    "#;
    let g = parse_dot(dot)?;
    assert_eq!(g.edges[0].weight(), 3);
    assert_eq!(g.edges[1].weight(), 3);
    Ok(())
}

// ---------------------------------------------------------------------------
// Comments and strings (~6)
// ---------------------------------------------------------------------------

#[test]
fn strip_line_comments() -> TestResult {
    let dot = r#"
        digraph G {
            // This is a comment
            A // inline comment
            B
        }
    "#;
    let g = parse_dot(dot)?;
    assert_eq!(g.nodes.len(), 2);
    Ok(())
}

#[test]
fn strip_block_comments() -> TestResult {
    let dot = r#"
        digraph G {
            /* block comment */
            A
            /* multi
               line
               comment */
            B
        }
    "#;
    let g = parse_dot(dot)?;
    assert_eq!(g.nodes.len(), 2);
    Ok(())
}

#[test]
fn preserve_comments_in_strings() -> TestResult {
    let dot = r#"digraph G { A [label="has // comment"]; }"#;
    let g = parse_dot(dot)?;
    let a = g.get_node("A").ok_or("A not found")?;
    assert_eq!(a.get_str_attr("label"), Some("has // comment"));
    Ok(())
}

#[test]
fn unicode_in_strings() -> TestResult {
    let dot = r#"digraph G { A [label="こんにちは 🌍"]; }"#;
    let g = parse_dot(dot)?;
    let a = g.get_node("A").ok_or("A not found")?;
    assert_eq!(a.get_str_attr("label"), Some("こんにちは 🌍"));
    Ok(())
}

#[test]
fn multiline_string() -> TestResult {
    let dot = r#"digraph G { A [prompt="line1\nline2\nline3"]; }"#;
    let g = parse_dot(dot)?;
    let a = g.get_node("A").ok_or("A not found")?;
    let prompt = a.get_str_attr("prompt").ok_or("no prompt")?;
    assert_eq!(prompt.lines().count(), 3);
    Ok(())
}

#[test]
fn nested_block_comments() -> TestResult {
    let dot = r#"
        digraph G {
            /* outer /* inner */ still comment */
            A
        }
    "#;
    let g = parse_dot(dot)?;
    assert_eq!(g.nodes.len(), 1);
    Ok(())
}

// ---------------------------------------------------------------------------
// Rejection/error cases (~6)
// ---------------------------------------------------------------------------

#[test]
fn reject_undirected_graph() -> TestResult {
    let result = parse_dot("graph G { A; }");
    assert!(result.is_err());
    let err = result.err().ok_or("expected error")?;
    assert!(matches!(err, AttractorError::InvalidPipeline { .. }));
    assert!(err.to_string().contains("directed"));
    Ok(())
}

#[test]
fn reject_strict_modifier() -> TestResult {
    let result = parse_dot("strict digraph G { A; }");
    assert!(result.is_err());
    let err = result.err().ok_or("expected error")?;
    assert!(matches!(err, AttractorError::InvalidPipeline { .. }));
    assert!(err.to_string().contains("strict"));
    Ok(())
}

#[test]
fn reject_undirected_edge() -> TestResult {
    let result = parse_dot("digraph G { A -- B; }");
    assert!(result.is_err());
    Ok(())
}

#[test]
fn reject_empty_input() -> TestResult {
    let result = parse_dot("");
    assert!(result.is_err());
    let err = result.err().ok_or("expected error")?;
    assert!(matches!(err, AttractorError::InvalidPipeline { .. }));
    assert!(err.to_string().contains("empty"));
    Ok(())
}

#[test]
fn reject_malformed_attr_block() -> TestResult {
    let result = parse_dot(r#"digraph G { A [label="unclosed }"#);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn error_messages_contain_context() -> TestResult {
    let result = parse_dot("not a graph at all");
    assert!(result.is_err());
    let err = result.err().ok_or("expected error")?;
    // Should be an InvalidPipeline with some useful context
    assert!(matches!(err, AttractorError::InvalidPipeline { .. }));
    Ok(())
}

// ---------------------------------------------------------------------------
// Keyword node ID rejection (review finding 2)
// ---------------------------------------------------------------------------

#[test]
fn reject_keyword_node_id() -> TestResult {
    // `node` as a bare node ID is ambiguous with `node [defaults]`
    let result = parse_dot("digraph G { node; }");
    assert!(result.is_err());
    Ok(())
}

#[test]
fn reject_keyword_edge_as_node_id() -> TestResult {
    // `edge` as a bare node ID is ambiguous with `edge [defaults]`
    let result = parse_dot("digraph G { edge; }");
    assert!(result.is_err());
    Ok(())
}

#[test]
fn reject_keyword_graph_as_node_id() -> TestResult {
    // `graph` as a bare node ID is ambiguous with `graph [attrs]`
    let result = parse_dot("digraph G { graph; }");
    assert!(result.is_err());
    Ok(())
}

// ---------------------------------------------------------------------------
// Graph query methods (~8)
// ---------------------------------------------------------------------------

#[test]
fn find_start_node_by_shape() -> TestResult {
    let dot = r#"
        digraph G {
            begin [shape=Mdiamond]
            A
        }
    "#;
    let g = parse_dot(dot)?;
    let start = g.find_start_node()?;
    assert_eq!(start.id, "begin");
    Ok(())
}

#[test]
fn find_start_node_by_id_fallback() -> TestResult {
    let dot = r#"
        digraph G {
            start [label="Go"]
            A
        }
    "#;
    let g = parse_dot(dot)?;
    let start = g.find_start_node()?;
    assert_eq!(start.id, "start");
    Ok(())
}

#[test]
fn find_start_node_missing() -> TestResult {
    let dot = r#"
        digraph G {
            A
            B
        }
    "#;
    let g = parse_dot(dot)?;
    let result = g.find_start_node();
    assert!(result.is_err());
    assert!(matches!(
        result.err().ok_or("expected error")?,
        AttractorError::NoStartNode
    ));
    Ok(())
}

#[test]
fn find_exit_node_by_shape() -> TestResult {
    let dot = r#"
        digraph G {
            done [shape=Msquare]
            A
        }
    "#;
    let g = parse_dot(dot)?;
    let exit_node = g.find_exit_node()?;
    assert_eq!(exit_node.id, "done");
    Ok(())
}

#[test]
fn find_exit_node_missing() -> TestResult {
    let dot = r#"
        digraph G {
            A
            B
        }
    "#;
    let g = parse_dot(dot)?;
    let result = g.find_exit_node();
    assert!(result.is_err());
    assert!(matches!(
        result.err().ok_or("expected error")?,
        AttractorError::NoExitNode
    ));
    Ok(())
}

#[test]
fn outgoing_edges_returns_correct() -> TestResult {
    let dot = r#"
        digraph G {
            A -> B
            A -> C
            B -> C
        }
    "#;
    let g = parse_dot(dot)?;
    let out_a = g.outgoing_edges("A");
    assert_eq!(out_a.len(), 2);
    let out_b = g.outgoing_edges("B");
    assert_eq!(out_b.len(), 1);
    let out_c = g.outgoing_edges("C");
    assert_eq!(out_c.len(), 0);
    Ok(())
}

#[test]
fn incoming_edges_returns_correct() -> TestResult {
    let dot = r#"
        digraph G {
            A -> C
            B -> C
        }
    "#;
    let g = parse_dot(dot)?;
    let inc_c = g.incoming_edges("C");
    assert_eq!(inc_c.len(), 2);
    let inc_a = g.incoming_edges("A");
    assert_eq!(inc_a.len(), 0);
    Ok(())
}

#[test]
fn handler_type_from_shape() -> TestResult {
    let dot = r#"
        digraph G {
            a [shape=Mdiamond]
            b [shape=Msquare]
            c [shape="box"]
            d [shape="hexagon"]
            e [shape="diamond"]
            f [shape="component"]
            g [shape="tripleoctagon"]
            h [shape="parallelogram"]
            i [shape="house"]
            j [shape="unknown_shape"]
        }
    "#;
    let g = parse_dot(dot)?;
    assert_eq!(g.get_node("a").ok_or("a")?.handler_type(), "start");
    assert_eq!(g.get_node("b").ok_or("b")?.handler_type(), "exit");
    assert_eq!(g.get_node("c").ok_or("c")?.handler_type(), "codergen");
    assert_eq!(g.get_node("d").ok_or("d")?.handler_type(), "wait.human");
    assert_eq!(g.get_node("e").ok_or("e")?.handler_type(), "conditional");
    assert_eq!(g.get_node("f").ok_or("f")?.handler_type(), "parallel");
    assert_eq!(
        g.get_node("g").ok_or("g")?.handler_type(),
        "parallel.fan_in"
    );
    assert_eq!(g.get_node("h").ok_or("h")?.handler_type(), "tool");
    assert_eq!(
        g.get_node("i").ok_or("i")?.handler_type(),
        "stack.manager_loop"
    );
    assert_eq!(g.get_node("j").ok_or("j")?.handler_type(), "codergen"); // unknown defaults to codergen
    Ok(())
}

#[test]
fn handler_type_explicit_overrides_shape() -> TestResult {
    let (_, a) = parse_and_get_node(
        r#"digraph G { A [shape="box", type="custom_handler"]; }"#,
        "A",
    )?;
    assert_eq!(a.handler_type(), "custom_handler");
    Ok(())
}

// ---------------------------------------------------------------------------
// AttrValue tests (~4)
// ---------------------------------------------------------------------------

#[test]
fn attr_value_accessors() -> TestResult {
    let s = AttrValue::from("hello");
    assert_eq!(s.as_str(), Some("hello"));
    assert_eq!(s.as_i64(), None);
    assert_eq!(s.as_f64(), None);
    assert_eq!(s.as_bool(), None);
    assert!(s.as_duration().is_none());

    let i = AttrValue::from(42_i64);
    assert_eq!(i.as_i64(), Some(42));
    assert_eq!(i.as_str(), None);

    let f = AttrValue::from(3.125_f64);
    assert!((f.as_f64().ok_or("no f64")? - 3.125).abs() < f64::EPSILON);

    let b = AttrValue::from(true);
    assert_eq!(b.as_bool(), Some(true));

    let d = AttrValue::Duration(Duration::from_spec_str("15m")?);
    assert!(d.as_duration().is_some());
    assert_eq!(d.as_duration().ok_or("no dur")?.inner().as_secs(), 900);
    Ok(())
}

#[test]
fn attr_value_display() -> TestResult {
    assert_eq!(AttrValue::from("hello").to_string(), r#""hello""#);
    assert_eq!(AttrValue::from(42_i64).to_string(), "42");
    assert_eq!(AttrValue::from(3.125_f64).to_string(), "3.125");
    assert_eq!(AttrValue::from(true).to_string(), "true");
    assert_eq!(AttrValue::from(false).to_string(), "false");
    let dur = AttrValue::Duration(Duration::from_spec_str("15m")?);
    assert_eq!(dur.to_string(), "15m");
    Ok(())
}

#[test]
fn attr_value_from_impls() -> TestResult {
    let _: AttrValue = "hello".into();
    let _: AttrValue = String::from("hello").into();
    let _: AttrValue = 42_i64.into();
    let _: AttrValue = 3.125_f64.into();
    let _: AttrValue = true.into();
    let _: AttrValue = Duration::from_spec_str("5s")?.into();
    Ok(())
}

#[test]
fn attr_value_serde_roundtrip() -> TestResult {
    // String roundtrip — exact equality
    let s = AttrValue::from("hello");
    let json = serde_json::to_string(&s)?;
    assert_eq!(json, r#""hello""#);
    let back: AttrValue = serde_json::from_str(&json)?;
    assert_eq!(back, s);

    // Integer roundtrip — exact equality
    let i = AttrValue::from(42_i64);
    let json = serde_json::to_string(&i)?;
    assert_eq!(json, "42");
    let back: AttrValue = serde_json::from_str(&json)?;
    assert_eq!(back, i);

    // Boolean roundtrip — exact equality
    let b = AttrValue::from(true);
    let json = serde_json::to_string(&b)?;
    assert_eq!(json, "true");
    let back: AttrValue = serde_json::from_str(&json)?;
    assert_eq!(back, b);

    // Float roundtrip — serde_json may deserialize as Integer if no fraction,
    // so use a value with a fractional part and check approximate equality
    let f = AttrValue::from(3.125_f64);
    let json = serde_json::to_string(&f)?;
    let back: AttrValue = serde_json::from_str(&json)?;
    let back_f64 = back.as_f64().ok_or("expected float back")?;
    assert!((back_f64 - 3.125).abs() < f64::EPSILON);

    // Duration serializes as string, so it roundtrips as String variant through serde
    // (serde_json doesn't know about Duration). This is expected with #[serde(untagged)].
    let dur = AttrValue::Duration(Duration::from_spec_str("15m")?);
    let json = serde_json::to_string(&dur)?;
    assert_eq!(json, r#""15m""#);
    let back: AttrValue = serde_json::from_str(&json)?;
    assert_eq!(back.as_str(), Some("15m"));
    Ok(())
}

// ---------------------------------------------------------------------------
// Additional edge cases
// ---------------------------------------------------------------------------

#[test]
fn qualified_attr_key() -> TestResult {
    let (_, a) = parse_and_get_node(r#"digraph G { A [tool_hooks.pre="validate"]; }"#, "A")?;
    assert_eq!(a.get_str_attr("tool_hooks.pre"), Some("validate"));
    Ok(())
}

#[test]
fn semicolons_optional() -> TestResult {
    let dot = r#"
        digraph G {
            A
            B
            A -> B
        }
    "#;
    let g = parse_dot(dot)?;
    assert_eq!(g.nodes.len(), 2);
    assert_eq!(g.edges.len(), 1);
    Ok(())
}

#[test]
fn whitespace_variations() -> TestResult {
    // Tabs and various whitespace
    let dot = "digraph\tG\t{\n\tA\n\tB\n\tA\t->\tB\n}";
    let g = parse_dot(dot)?;
    assert_eq!(g.nodes.len(), 2);
    assert_eq!(g.edges.len(), 1);
    Ok(())
}

// ---------------------------------------------------------------------------
// Nested subgraph class inheritance (§2.10)
// ---------------------------------------------------------------------------

#[test]
fn nested_subgraph_inherits_outer_class() -> TestResult {
    // Inner subgraph without label should inherit outer's derived class
    let dot = r#"
        digraph G {
            subgraph cluster_outer {
                graph [label="Outer"]
                subgraph cluster_inner {
                    X
                }
            }
        }
    "#;
    let (_, x) = parse_and_get_node(dot, "X")?;
    assert_eq!(x.get_str_attr("class"), Some("outer"));
    Ok(())
}

#[test]
fn nested_subgraph_own_class_overrides_parent() -> TestResult {
    // Inner subgraph with its own label uses its own class, not the parent's
    let dot = r#"
        digraph G {
            subgraph cluster_outer {
                graph [label="Outer"]
                subgraph cluster_inner {
                    graph [label="Inner"]
                    Y
                }
            }
        }
    "#;
    let (_, y) = parse_and_get_node(dot, "Y")?;
    assert_eq!(y.get_str_attr("class"), Some("inner"));
    Ok(())
}

// ---------------------------------------------------------------------------
// Subgraph without whitespace (§2.2 BNF)
// ---------------------------------------------------------------------------

#[test]
fn subgraph_no_whitespace_before_brace() -> TestResult {
    // subgraph{ ... } with no space is valid per BNF when identifier is omitted
    let dot = r#"
        digraph G {
            subgraph{
                A
            }
        }
    "#;
    let g = parse_dot(dot)?;
    assert!(g.get_node("A").is_some());
    Ok(())
}

#[test]
fn subgraph_with_name_no_extra_space() -> TestResult {
    // subgraph cluster_x{ ... } should also work
    let dot = "digraph G { subgraph cluster_x{ A } }";
    let g = parse_dot(dot)?;
    assert!(g.get_node("A").is_some());
    Ok(())
}

// ---------------------------------------------------------------------------
// Subgraph names accept DOT keywords (review finding 1)
// ---------------------------------------------------------------------------

#[test]
fn subgraph_keyword_name_accepted() -> TestResult {
    // Subgraph names should accept DOT keywords — no parsing ambiguity exists
    // because the `subgraph` keyword has already been consumed.
    let dot = r#"digraph G { subgraph node { A } }"#;
    let g = parse_dot(dot)?;
    assert!(g.get_node("A").is_some());
    Ok(())
}

#[test]
fn subgraph_keyword_name_graph_accepted() -> TestResult {
    let dot = r#"digraph G { subgraph graph { B } }"#;
    let g = parse_dot(dot)?;
    assert!(g.get_node("B").is_some());
    Ok(())
}

// ---------------------------------------------------------------------------
// Test helper usage
// ---------------------------------------------------------------------------

#[test]
fn parse_and_get_node_helper_works() -> TestResult {
    let dot = r#"digraph G { A [shape="diamond"]; }"#;
    let (_, a) = parse_and_get_node(dot, "A")?;
    assert_eq!(a.handler_type(), "conditional");
    Ok(())
}

// ---------------------------------------------------------------------------
// Unterminated block comments (finding 1)
// ---------------------------------------------------------------------------

#[test]
fn reject_unterminated_block_comment() -> TestResult {
    let result = parse_dot("digraph G { A } /*");
    assert!(result.is_err());
    let err = result.err().ok_or("expected error")?;
    assert!(matches!(err, AttractorError::InvalidPipeline { .. }));
    assert!(err.to_string().contains("unterminated"));
    Ok(())
}

#[test]
fn reject_unterminated_nested_block_comment() -> TestResult {
    let result = parse_dot("digraph G { /* outer /* inner */ }");
    assert!(result.is_err());
    Ok(())
}

// ---------------------------------------------------------------------------
// Unknown escape sequences (finding 3)
// ---------------------------------------------------------------------------

#[test]
fn reject_unknown_escape_sequence() -> TestResult {
    // \r is not in the spec's escape list
    let result = parse_dot(r#"digraph G { A [label="hello\rworld"]; }"#);
    assert!(result.is_err());
    Ok(())
}

#[test]
fn reject_unknown_escape_x() -> TestResult {
    let result = parse_dot(r#"digraph G { A [label="test\x41"]; }"#);
    assert!(result.is_err());
    Ok(())
}

// ---------------------------------------------------------------------------
// Multiple graphs rejected (finding 4)
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Node re-declaration preserves explicit attrs (§2.10/§2.11)
// ---------------------------------------------------------------------------

#[test]
fn node_redeclaration_preserves_explicit_attrs() -> TestResult {
    // Node A declared with explicit shape, then re-declared bare inside a
    // subgraph that has `node [shape="box"]` defaults. The explicit shape
    // from the first declaration must be preserved.
    let dot = r#"
        digraph G {
            A [shape="diamond", label="Check"]
            subgraph cluster_loop {
                graph [label="Loop"]
                node [shape="box"]
                A
            }
        }
    "#;
    let (_, a) = parse_and_get_node(dot, "A")?;
    assert_eq!(a.shape(), "diamond"); // explicit attr preserved, not overwritten by default
    assert_eq!(a.label(), "Check");
    // Subgraph class should still be appended
    assert_eq!(a.get_str_attr("class"), Some("loop"));
    Ok(())
}

#[test]
fn node_redeclaration_explicit_overrides_prior() -> TestResult {
    // When a re-declaration provides an explicit attr, it should override
    // the prior value (explicit beats explicit).
    let dot = r#"
        digraph G {
            A [label="First"]
            A [label="Second"]
        }
    "#;
    let (_, a) = parse_and_get_node(dot, "A")?;
    assert_eq!(a.label(), "Second");
    Ok(())
}

// ---------------------------------------------------------------------------
// Subgraph graph attrs must not leak to root (review finding 1)
// ---------------------------------------------------------------------------

#[test]
fn subgraph_graph_attrs_do_not_leak_to_root() -> TestResult {
    let dot = r#"
        digraph G {
            graph [goal="root goal"]
            subgraph cluster_x {
                graph [label="Inner"]
                A
            }
        }
    "#;
    let g = parse_dot(dot)?;
    // Root graph should keep its own goal
    assert_eq!(
        g.get_graph_attr("goal").and_then(AttrValue::as_str),
        Some("root goal")
    );
    // Subgraph's label should NOT appear as a root graph attr
    assert!(
        g.get_graph_attr("label").is_none(),
        "subgraph label should not leak to root graph attrs"
    );
    Ok(())
}

#[test]
fn subgraph_graph_attr_decl_does_not_leak_to_root() -> TestResult {
    let dot = r#"
        digraph G {
            goal = "root goal"
            subgraph cluster_x {
                label = "Inner"
                A
            }
        }
    "#;
    let g = parse_dot(dot)?;
    assert_eq!(
        g.get_graph_attr("goal").and_then(AttrValue::as_str),
        Some("root goal")
    );
    assert!(
        g.get_graph_attr("label").is_none(),
        "subgraph label decl should not leak to root graph attrs"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// derive_subgraph_class uses last label (review finding 2)
// ---------------------------------------------------------------------------

#[test]
fn subgraph_class_derives_from_last_label() -> TestResult {
    let dot = r#"
        digraph G {
            subgraph cluster_x {
                graph [label="First"]
                graph [label="Second"]
                A
            }
        }
    "#;
    let (_, a) = parse_and_get_node(dot, "A")?;
    assert_eq!(a.get_str_attr("class"), Some("second"));
    Ok(())
}

// ---------------------------------------------------------------------------
// Multiple graphs rejected (finding 4)
// ---------------------------------------------------------------------------

#[test]
fn reject_multiple_graphs() -> TestResult {
    let result = parse_dot("digraph A {} digraph B {}");
    assert!(result.is_err());
    let err = result.err().ok_or("expected error")?;
    assert!(matches!(err, AttractorError::InvalidPipeline { .. }));
    assert!(err.to_string().contains("one graph"));
    Ok(())
}
