//! Tests for dynamic parallel fan-out.

mod common;

use std::sync::Arc;

use async_trait::async_trait;

use stencila_attractor::context::Context;
use stencila_attractor::error::AttractorResult;
use stencila_attractor::events::{CollectingEmitter, NoOpEmitter, PipelineEvent};
use stencila_attractor::graph::{AttrValue, Edge, Graph, Node};
use stencila_attractor::handler::{Handler, HandlerRegistry};
use stencila_attractor::handlers::ParallelHandler;
use stencila_attractor::parser::parse_dot;
use stencila_attractor::transform::Transform;
use stencila_attractor::transforms::NodeSugarTransform;
use stencila_attractor::types::{Outcome, StageStatus};
use stencila_attractor::validation;

/// A handler that always returns FAIL.
struct FailHandler;

#[async_trait]
impl Handler for FailHandler {
    async fn execute(
        &self,
        _node: &Node,
        _context: &Context,
        _graph: &Graph,
    ) -> AttractorResult<Outcome> {
        Ok(Outcome::fail("intentional failure"))
    }
}

/// A handler that reads `fan_out.item.name` from context, writes it to
/// `last_output_full`, and succeeds. Used to verify object property flattening.
struct EchoFanOutItemNameHandler;

#[async_trait]
impl Handler for EchoFanOutItemNameHandler {
    async fn execute(
        &self,
        _node: &Node,
        context: &Context,
        _graph: &Graph,
    ) -> AttractorResult<Outcome> {
        let name = context.get_string("fan_out.item.name");
        let index = context.get_string("fan_out.index");
        let total = context.get_string("fan_out.total");
        let key = context.get_string("fan_out.key");
        let output = format!("name={name},index={index},total={total},key={key}");
        context.set("last_output_full", serde_json::Value::String(output));
        Ok(Outcome::success())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn default_registry() -> Arc<HandlerRegistry> {
    Arc::new(HandlerRegistry::with_defaults())
}

fn mixed_registry() -> Arc<HandlerRegistry> {
    let mut reg = HandlerRegistry::with_defaults();
    reg.register("fail_node", FailHandler);
    Arc::new(reg)
}

fn echo_registry() -> Arc<HandlerRegistry> {
    let mut reg = HandlerRegistry::with_defaults();
    reg.register("echo_fan_out", EchoFanOutItemNameHandler);
    Arc::new(reg)
}

/// Build a minimal dynamic fan-out graph:
///   fan_out_node [shape=component, fan_out="items"]
///   fan_out_node -> template_node
///   template_node -> fan_in [shape=tripleoctagon]
///   fan_in -> after
fn dynamic_fan_out_graph() -> Graph {
    let mut g = Graph::new("test_dynamic");

    let mut fan = Node::new("fan_out_node");
    fan.attrs
        .insert("shape".into(), AttrValue::from("component"));
    fan.attrs.insert("fan_out".into(), AttrValue::from("items"));
    g.add_node(fan);

    g.add_node(Node::new("template_node"));
    g.add_edge(Edge::new("fan_out_node", "template_node"));

    let mut fi = Node::new("fan_in");
    fi.attrs
        .insert("shape".into(), AttrValue::from("tripleoctagon"));
    g.add_node(fi);
    g.add_edge(Edge::new("template_node", "fan_in"));

    g.add_node(Node::new("after"));
    g.add_edge(Edge::new("fan_in", "after"));

    g
}

// ===========================================================================
// AC-1: basic dynamic fan-out with 3 string items
// ===========================================================================

#[tokio::test]
async fn dynamic_fan_out_basic_3_items() -> AttractorResult<()> {
    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let g = dynamic_fan_out_graph();
    let node = g.get_node("fan_out_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "fan_out_node".into(),
        }
    })?;
    let ctx = Context::new();
    ctx.set("items", serde_json::json!(["alpha", "beta", "gamma"]));

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert!(outcome.status.is_success());

    // parallel.results should have 3 entries
    let results = ctx.get("parallel.results");
    let arr = results.as_ref().and_then(|v| v.as_array());
    assert_eq!(arr.map(Vec::len), Some(3));

    // Each entry should have fan_out_index and fan_out_item
    let results_arr = arr.expect("results should be array");
    for (i, entry) in results_arr.iter().enumerate() {
        assert_eq!(
            entry.get("fan_out_index").and_then(|v| v.as_u64()),
            Some(i as u64)
        );
        assert!(entry.get("fan_out_item").is_some());
    }

    // parallel.outputs should also have 3 entries
    let outputs = ctx.get("parallel.outputs");
    let outputs_arr = outputs.as_ref().and_then(|v| v.as_array());
    assert_eq!(outputs_arr.map(Vec::len), Some(3));

    // jump target should be the fan-in node
    assert_eq!(outcome.jump_target.as_deref(), Some("fan_in"));

    Ok(())
}

// ===========================================================================
// AC-2: fan_out=true convention-based key derivation
// ===========================================================================

#[tokio::test]
async fn dynamic_fan_out_true_derives_key_from_id() -> AttractorResult<()> {
    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let mut g = Graph::new("test_fanout_true");

    let mut fan = Node::new("FanOutItems");
    fan.attrs
        .insert("shape".into(), AttrValue::from("component"));
    fan.attrs.insert("fan_out".into(), AttrValue::Boolean(true));
    g.add_node(fan);

    g.add_node(Node::new("process"));
    g.add_edge(Edge::new("FanOutItems", "process"));

    let mut fi = Node::new("merge");
    fi.attrs
        .insert("shape".into(), AttrValue::from("tripleoctagon"));
    g.add_node(fi);
    g.add_edge(Edge::new("process", "merge"));

    let node = g.get_node("FanOutItems").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "FanOutItems".into(),
        }
    })?;
    let ctx = Context::new();
    // snake_case of "FanOutItems" is "fan_out_items"
    ctx.set("fan_out_items", serde_json::json!(["x", "y"]));

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert!(outcome.status.is_success());
    let results = ctx.get("parallel.results");
    let arr = results.as_ref().and_then(|v| v.as_array());
    assert_eq!(arr.map(Vec::len), Some(2));

    Ok(())
}

// ===========================================================================
// AC-3: non-array context value returns fail
// ===========================================================================

#[tokio::test]
async fn dynamic_fan_out_non_array_fails() -> AttractorResult<()> {
    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let g = dynamic_fan_out_graph();
    let node = g.get_node("fan_out_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "fan_out_node".into(),
        }
    })?;
    let ctx = Context::new();
    ctx.set("items", serde_json::json!({"not": "an array"}));

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Fail);
    assert!(outcome.failure_reason.contains("object"));

    Ok(())
}

// ===========================================================================
// AC-3b: missing context key returns fail
// ===========================================================================

#[tokio::test]
async fn dynamic_fan_out_missing_key_fails() -> AttractorResult<()> {
    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let g = dynamic_fan_out_graph();
    let node = g.get_node("fan_out_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "fan_out_node".into(),
        }
    })?;
    let ctx = Context::new();
    // Don't set "items" in context

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Fail);
    assert!(outcome.failure_reason.contains("not found"));

    Ok(())
}

// ===========================================================================
// AC-6: empty list bypasses fan-in
// ===========================================================================

#[tokio::test]
async fn dynamic_fan_out_empty_list_bypasses_fan_in() -> AttractorResult<()> {
    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let g = dynamic_fan_out_graph();
    let node = g.get_node("fan_out_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "fan_out_node".into(),
        }
    })?;
    let ctx = Context::new();
    ctx.set("items", serde_json::json!([]));

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert!(outcome.status.is_success());

    // parallel.results and outputs should be empty
    let results = ctx.get("parallel.results");
    let arr = results.as_ref().and_then(|v| v.as_array());
    assert_eq!(arr.map(Vec::len), Some(0));

    let outputs = ctx.get("parallel.outputs");
    let outputs_arr = outputs.as_ref().and_then(|v| v.as_array());
    assert_eq!(outputs_arr.map(Vec::len), Some(0));

    // Jump target should be the fan-in's successor ("after"), not the fan-in itself
    assert_eq!(outcome.jump_target.as_deref(), Some("after"));

    Ok(())
}

// ===========================================================================
// AC-7: validation — multiple outgoing edges
// ===========================================================================

#[test]
fn validation_dynamic_fan_out_multiple_edges() {
    let mut g = Graph::new("test_validation");

    let mut start = Node::new("Start");
    start
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);

    let mut fan = Node::new("fan_out_node");
    fan.attrs
        .insert("shape".into(), AttrValue::from("component"));
    fan.attrs.insert("fan_out".into(), AttrValue::from("items"));
    g.add_node(fan);

    g.add_node(Node::new("branch_a"));
    g.add_node(Node::new("branch_b"));
    g.add_node(Node::new("branch_c"));

    let mut end = Node::new("End");
    end.attrs.insert("shape".into(), AttrValue::from("Msquare"));
    g.add_node(end);

    g.add_edge(Edge::new("Start", "fan_out_node"));
    g.add_edge(Edge::new("fan_out_node", "branch_a"));
    g.add_edge(Edge::new("fan_out_node", "branch_b"));
    g.add_edge(Edge::new("fan_out_node", "branch_c"));
    g.add_edge(Edge::new("branch_a", "End"));
    g.add_edge(Edge::new("branch_b", "End"));
    g.add_edge(Edge::new("branch_c", "End"));

    let diagnostics = validation::validate(&g, &[]);
    let fan_out_errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.rule == "dynamic_fan_out")
        .collect();

    assert_eq!(fan_out_errors.len(), 1);
    assert_eq!(fan_out_errors[0].severity, validation::Severity::Error);
    assert!(fan_out_errors[0].message.contains("3 outgoing edge(s)"));
}

// ===========================================================================
// AC-8: sugar transform — fan_out implies component shape
// ===========================================================================

#[test]
fn sugar_fan_out_string_implies_component() -> AttractorResult<()> {
    let mut g = parse_dot(
        r#"
        digraph T {
            Start -> Process -> End
            Process [fan_out="my_list"]
        }
        "#,
    )?;
    NodeSugarTransform.apply(&mut g)?;

    let node = g.get_node("Process").expect("Process node should exist");
    assert_eq!(node.shape(), "component");
    assert_eq!(node.handler_type(), "parallel");
    // fan_out should NOT be drained
    assert!(node.attrs.contains_key("fan_out"));

    Ok(())
}

// ===========================================================================
// AC-9: object property flattening — verify flattened keys are visible
// ===========================================================================

#[tokio::test]
async fn dynamic_fan_out_object_property_flattening() -> AttractorResult<()> {
    let registry = echo_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    // Build graph where the template node uses the echo handler
    let mut g = Graph::new("test_flattening");

    let mut fan = Node::new("fan_out_node");
    fan.attrs
        .insert("shape".into(), AttrValue::from("component"));
    fan.attrs.insert("fan_out".into(), AttrValue::from("items"));
    g.add_node(fan);

    let mut template = Node::new("template_node");
    template
        .attrs
        .insert("type".into(), AttrValue::from("echo_fan_out"));
    g.add_node(template);
    g.add_edge(Edge::new("fan_out_node", "template_node"));

    let mut fi = Node::new("fan_in");
    fi.attrs
        .insert("shape".into(), AttrValue::from("tripleoctagon"));
    g.add_node(fi);
    g.add_edge(Edge::new("template_node", "fan_in"));

    let node = g.get_node("fan_out_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "fan_out_node".into(),
        }
    })?;
    let ctx = Context::new();
    ctx.set(
        "items",
        serde_json::json!([
            {"name": "rust", "path": "/skills/rust"},
            {"name": "python", "path": "/skills/python"}
        ]),
    );

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert!(outcome.status.is_success());

    // parallel.outputs should contain the echoed values with flattened properties
    let outputs = ctx.get("parallel.outputs");
    let outputs_arr = outputs
        .as_ref()
        .and_then(|v| v.as_array())
        .expect("parallel.outputs should be an array");
    assert_eq!(outputs_arr.len(), 2);

    // Branch 0: name=rust
    let out0 = outputs_arr[0].as_str().expect("output 0 should be string");
    assert!(
        out0.contains("name=rust"),
        "expected 'name=rust' in output 0, got: {out0}"
    );
    assert!(
        out0.contains("index=0"),
        "expected 'index=0' in output 0, got: {out0}"
    );
    assert!(
        out0.contains("total=2"),
        "expected 'total=2' in output 0, got: {out0}"
    );
    assert!(
        out0.contains("key=items"),
        "expected 'key=items' in output 0, got: {out0}"
    );

    // Branch 1: name=python
    let out1 = outputs_arr[1].as_str().expect("output 1 should be string");
    assert!(
        out1.contains("name=python"),
        "expected 'name=python' in output 1, got: {out1}"
    );
    assert!(
        out1.contains("index=1"),
        "expected 'index=1' in output 1, got: {out1}"
    );

    Ok(())
}

// ===========================================================================
// AC-10: parallel.outputs populated
// ===========================================================================

#[tokio::test]
async fn dynamic_fan_out_outputs_populated() -> AttractorResult<()> {
    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let g = dynamic_fan_out_graph();
    let node = g.get_node("fan_out_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "fan_out_node".into(),
        }
    })?;
    let ctx = Context::new();
    ctx.set("items", serde_json::json!(["a", "b"]));

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert!(outcome.status.is_success());

    let outputs = ctx.get("parallel.outputs");
    assert!(outputs.is_some());
    let outputs_arr = outputs.as_ref().and_then(|v| v.as_array());
    assert_eq!(outputs_arr.map(Vec::len), Some(2));

    Ok(())
}

// ===========================================================================
// AC-11: static fan-out backward compatibility
// ===========================================================================

#[tokio::test]
async fn static_fan_out_backward_compatible() -> AttractorResult<()> {
    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    // Standard static fan-out graph (no fan_out attribute)
    let mut g = Graph::new("test_static");

    let mut par = Node::new("parallel_node");
    par.attrs
        .insert("shape".into(), AttrValue::from("component"));
    g.add_node(par);

    g.add_node(Node::new("branch_a"));
    g.add_node(Node::new("branch_b"));
    g.add_edge(Edge::new("parallel_node", "branch_a"));
    g.add_edge(Edge::new("parallel_node", "branch_b"));

    let node = g.get_node("parallel_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "parallel_node".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert!(outcome.status.is_success());

    let results = ctx.get("parallel.results");
    let arr = results.as_ref().and_then(|v| v.as_array());
    assert_eq!(arr.map(Vec::len), Some(2));

    // Static fan-out results should NOT have fan_out_index/fan_out_item
    let results_arr = arr.expect("results should be array");
    for entry in results_arr {
        assert!(entry.get("fan_out_index").is_none());
        assert!(entry.get("fan_out_item").is_none());
    }

    // parallel.outputs should also be populated for static fan-out
    let outputs = ctx.get("parallel.outputs");
    let outputs_arr = outputs.as_ref().and_then(|v| v.as_array());
    assert_eq!(outputs_arr.map(Vec::len), Some(2));

    Ok(())
}

// ===========================================================================
// AC-12: ParallelStarted event includes dynamic_item_count
// ===========================================================================

#[tokio::test]
async fn dynamic_fan_out_event_includes_item_count() -> AttractorResult<()> {
    let registry = default_registry();
    let emitter = Arc::new(CollectingEmitter::new());
    let handler = ParallelHandler::new(registry, Arc::clone(&emitter) as Arc<_>);

    let g = dynamic_fan_out_graph();
    let node = g.get_node("fan_out_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "fan_out_node".into(),
        }
    })?;
    let ctx = Context::new();
    ctx.set("items", serde_json::json!(["a", "b", "c"]));

    handler.execute(node, &ctx, &g).await?;

    let events = emitter.events();
    let started_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, PipelineEvent::ParallelStarted { .. }))
        .collect();

    assert_eq!(started_events.len(), 1);
    if let PipelineEvent::ParallelStarted {
        dynamic_item_count, ..
    } = &started_events[0]
    {
        assert_eq!(*dynamic_item_count, Some(3));
    }

    Ok(())
}

// ===========================================================================
// AC-13: fan_out + prompt — fan_out wins shape
// ===========================================================================

#[test]
fn sugar_fan_out_with_prompt_gets_component() -> AttractorResult<()> {
    let mut g = parse_dot(
        r#"
        digraph T {
            Start -> Process -> End
            Process [fan_out="my_list", prompt="do stuff"]
        }
        "#,
    )?;
    NodeSugarTransform.apply(&mut g)?;

    let node = g.get_node("Process").expect("Process node should exist");
    assert_eq!(node.shape(), "component");
    assert_eq!(node.handler_type(), "parallel");
    // Both attributes retained
    assert!(node.attrs.contains_key("fan_out"));
    assert!(node.attrs.contains_key("prompt"));

    Ok(())
}

// ===========================================================================
// AC-15: nested dynamic fan-out rejected by validation
// ===========================================================================

#[test]
fn validation_nested_dynamic_fan_out_rejected() {
    let mut g = Graph::new("test_nested");

    let mut start = Node::new("Start");
    start
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);

    let mut outer = Node::new("outer_fan");
    outer
        .attrs
        .insert("shape".into(), AttrValue::from("component"));
    outer
        .attrs
        .insert("fan_out".into(), AttrValue::from("outer_list"));
    g.add_node(outer);

    // Inner fan-out inside the template subgraph
    let mut inner = Node::new("inner_fan");
    inner
        .attrs
        .insert("shape".into(), AttrValue::from("component"));
    inner
        .attrs
        .insert("fan_out".into(), AttrValue::from("inner_list"));
    g.add_node(inner);

    g.add_node(Node::new("process"));

    let mut fi = Node::new("merge");
    fi.attrs
        .insert("shape".into(), AttrValue::from("tripleoctagon"));
    g.add_node(fi);

    let mut end = Node::new("End");
    end.attrs.insert("shape".into(), AttrValue::from("Msquare"));
    g.add_node(end);

    g.add_edge(Edge::new("Start", "outer_fan"));
    g.add_edge(Edge::new("outer_fan", "inner_fan"));
    g.add_edge(Edge::new("inner_fan", "process"));
    g.add_edge(Edge::new("process", "merge"));
    g.add_edge(Edge::new("merge", "End"));

    let diagnostics = validation::validate(&g, &[]);
    let nested_errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.rule == "nested_dynamic_fan_out")
        .collect();

    assert_eq!(nested_errors.len(), 1);
    assert_eq!(nested_errors[0].severity, validation::Severity::Error);
    assert!(nested_errors[0].message.contains("inner_fan"));
    assert!(nested_errors[0].message.contains("outer_fan"));
}

// ===========================================================================
// AC-16: static fan-out also populates parallel.outputs
// ===========================================================================

#[tokio::test]
async fn static_fan_out_also_populates_outputs() -> AttractorResult<()> {
    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let mut g = Graph::new("test_static_outputs");

    let mut par = Node::new("par");
    par.attrs
        .insert("shape".into(), AttrValue::from("component"));
    g.add_node(par);

    g.add_node(Node::new("a"));
    g.add_node(Node::new("b"));
    g.add_edge(Edge::new("par", "a"));
    g.add_edge(Edge::new("par", "b"));

    let node = g.get_node("par").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "par".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert!(outcome.status.is_success());

    let outputs = ctx.get("parallel.outputs");
    assert!(outputs.is_some());
    let outputs_arr = outputs.as_ref().and_then(|v| v.as_array());
    assert_eq!(outputs_arr.map(Vec::len), Some(2));

    Ok(())
}

// ===========================================================================
// AC-17: fan_out=true (boolean) sugar transform
// ===========================================================================

#[test]
fn sugar_fan_out_boolean_true_implies_component() -> AttractorResult<()> {
    // Build graph manually with AttrValue::Boolean(true)
    let mut g = Graph::new("test_sugar_bool");

    let mut start = Node::new("Start");
    start
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);

    let mut fan = Node::new("FanOutSkills");
    fan.attrs.insert("fan_out".into(), AttrValue::Boolean(true));
    g.add_node(fan);

    let mut end = Node::new("End");
    end.attrs.insert("shape".into(), AttrValue::from("Msquare"));
    g.add_node(end);

    g.add_edge(Edge::new("Start", "FanOutSkills"));
    g.add_edge(Edge::new("FanOutSkills", "End"));

    NodeSugarTransform.apply(&mut g)?;

    let node = g.get_node("FanOutSkills").expect("node should exist");
    assert_eq!(node.shape(), "component");
    assert_eq!(node.handler_type(), "parallel");
    // fan_out should still be on the node
    assert!(node.attrs.contains_key("fan_out"));

    Ok(())
}

// ===========================================================================
// AC-18: results ordered by fan-out index, not completion order
// ===========================================================================

#[tokio::test]
async fn dynamic_fan_out_results_ordered_by_index() -> AttractorResult<()> {
    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let g = dynamic_fan_out_graph();
    let node = g.get_node("fan_out_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "fan_out_node".into(),
        }
    })?;
    let ctx = Context::new();
    ctx.set(
        "items",
        serde_json::json!(["first", "second", "third", "fourth", "fifth"]),
    );

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert!(outcome.status.is_success());

    let results = ctx.get("parallel.results");
    let arr = results
        .as_ref()
        .and_then(|v| v.as_array())
        .expect("results should be array");

    // Verify results are in index order
    for (i, entry) in arr.iter().enumerate() {
        assert_eq!(
            entry.get("fan_out_index").and_then(|v| v.as_u64()),
            Some(i as u64),
            "result at position {i} should have fan_out_index={i}"
        );
    }

    Ok(())
}

// ===========================================================================
// AC-19: fan_out=false returns fail
// ===========================================================================

#[tokio::test]
async fn dynamic_fan_out_false_returns_fail() -> AttractorResult<()> {
    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let mut g = Graph::new("test_false");

    let mut fan = Node::new("fan_out_node");
    fan.attrs
        .insert("shape".into(), AttrValue::from("component"));
    fan.attrs
        .insert("fan_out".into(), AttrValue::Boolean(false));
    g.add_node(fan);

    g.add_node(Node::new("template"));
    g.add_edge(Edge::new("fan_out_node", "template"));

    let node = g.get_node("fan_out_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "fan_out_node".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Fail);
    assert!(outcome.failure_reason.contains("fan_out=false"));

    Ok(())
}

// ===========================================================================
// Single-item list
// ===========================================================================

#[tokio::test]
async fn dynamic_fan_out_single_item() -> AttractorResult<()> {
    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let g = dynamic_fan_out_graph();
    let node = g.get_node("fan_out_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "fan_out_node".into(),
        }
    })?;
    let ctx = Context::new();
    ctx.set("items", serde_json::json!(["only_one"]));

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert!(outcome.status.is_success());

    let results = ctx.get("parallel.results");
    let arr = results.as_ref().and_then(|v| v.as_array());
    assert_eq!(arr.map(Vec::len), Some(1));

    Ok(())
}

// ===========================================================================
// Dynamic fan-out with error_policy=ignore
// ===========================================================================

#[tokio::test]
async fn dynamic_fan_out_error_policy_ignore() -> AttractorResult<()> {
    let registry = mixed_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    // Build graph where the template node is a fail_node
    let mut g = Graph::new("test_ignore");

    let mut fan = Node::new("fan_out_node");
    fan.attrs
        .insert("shape".into(), AttrValue::from("component"));
    fan.attrs.insert("fan_out".into(), AttrValue::from("items"));
    fan.attrs
        .insert("error_policy".into(), AttrValue::from("ignore"));
    g.add_node(fan);

    let mut template = Node::new("template_node");
    template
        .attrs
        .insert("type".into(), AttrValue::from("fail_node"));
    g.add_node(template);
    g.add_edge(Edge::new("fan_out_node", "template_node"));

    let mut fi = Node::new("fan_in");
    fi.attrs
        .insert("shape".into(), AttrValue::from("tripleoctagon"));
    g.add_node(fi);
    g.add_edge(Edge::new("template_node", "fan_in"));

    let node = g.get_node("fan_out_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "fan_out_node".into(),
        }
    })?;
    let ctx = Context::new();
    ctx.set("items", serde_json::json!(["a", "b"]));

    let outcome = handler.execute(node, &ctx, &g).await?;

    // With ignore policy, failures are hidden → overall SUCCESS
    assert!(outcome.status.is_success());

    // parallel.results should be empty (all failures filtered)
    let results = ctx.get("parallel.results");
    let arr = results.as_ref().and_then(|v| v.as_array());
    assert_eq!(arr.map(Vec::len), Some(0));

    // parallel.outputs preserves all branch positions regardless of
    // error_policy filtering. This is intentional: outputs are built
    // before error filtering so that downstream consumers can correlate
    // outputs by fan_out_index. When error_policy=ignore,
    // parallel.results may be shorter than parallel.outputs.
    let outputs = ctx.get("parallel.outputs");
    let outputs_arr = outputs.as_ref().and_then(|v| v.as_array());
    assert_eq!(
        outputs_arr.map(Vec::len),
        Some(2),
        "parallel.outputs should contain all branch outputs even when error_policy=ignore"
    );

    Ok(())
}

// ===========================================================================
// Multiple edges on dynamic fan-out returns fail at runtime
// ===========================================================================

#[tokio::test]
async fn dynamic_fan_out_multiple_edges_runtime_fail() -> AttractorResult<()> {
    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let mut g = Graph::new("test_multi_edge");

    let mut fan = Node::new("fan_out_node");
    fan.attrs
        .insert("shape".into(), AttrValue::from("component"));
    fan.attrs.insert("fan_out".into(), AttrValue::from("items"));
    g.add_node(fan);

    g.add_node(Node::new("a"));
    g.add_node(Node::new("b"));
    g.add_edge(Edge::new("fan_out_node", "a"));
    g.add_edge(Edge::new("fan_out_node", "b"));

    let node = g.get_node("fan_out_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "fan_out_node".into(),
        }
    })?;
    let ctx = Context::new();
    ctx.set("items", serde_json::json!(["x"]));

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Fail);
    assert!(outcome.failure_reason.contains("2 outgoing edges"));

    Ok(())
}

// ===========================================================================
// Validation: no edges on dynamic fan-out
// ===========================================================================

#[test]
fn validation_dynamic_fan_out_zero_edges() {
    let mut g = Graph::new("test_validation_zero");

    let mut start = Node::new("Start");
    start
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);

    let mut fan = Node::new("fan_out_node");
    fan.attrs
        .insert("shape".into(), AttrValue::from("component"));
    fan.attrs.insert("fan_out".into(), AttrValue::from("items"));
    g.add_node(fan);

    let mut end = Node::new("End");
    end.attrs.insert("shape".into(), AttrValue::from("Msquare"));
    g.add_node(end);

    g.add_edge(Edge::new("Start", "fan_out_node"));
    // No edge from fan_out_node!
    g.add_edge(Edge::new("fan_out_node", "End"));

    // Remove that edge to make zero outgoing... wait, we need an actual
    // graph with zero edges from the fan_out_node to trigger the rule.
    // Let's just check with a modified graph:
    let mut g2 = Graph::new("test_validation_zero");

    let mut start2 = Node::new("Start");
    start2
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g2.add_node(start2);

    let mut fan2 = Node::new("fan_out_node");
    fan2.attrs
        .insert("shape".into(), AttrValue::from("component"));
    fan2.attrs
        .insert("fan_out".into(), AttrValue::from("items"));
    g2.add_node(fan2);

    let mut end2 = Node::new("End");
    end2.attrs
        .insert("shape".into(), AttrValue::from("Msquare"));
    g2.add_node(end2);

    g2.add_edge(Edge::new("Start", "fan_out_node"));
    // Zero outgoing edges from fan_out_node

    let diagnostics = validation::validate(&g2, &[]);
    let fan_out_errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.rule == "dynamic_fan_out")
        .collect();

    assert_eq!(fan_out_errors.len(), 1);
    assert!(fan_out_errors[0].message.contains("0 outgoing edge(s)"));
}

// ===========================================================================
// Static event has dynamic_item_count=None
// ===========================================================================

#[tokio::test]
async fn static_fan_out_event_has_no_item_count() -> AttractorResult<()> {
    let registry = default_registry();
    let emitter = Arc::new(CollectingEmitter::new());
    let handler = ParallelHandler::new(registry, Arc::clone(&emitter) as Arc<_>);

    let mut g = Graph::new("test_static_event");

    let mut par = Node::new("par");
    par.attrs
        .insert("shape".into(), AttrValue::from("component"));
    g.add_node(par);

    g.add_node(Node::new("a"));
    g.add_node(Node::new("b"));
    g.add_edge(Edge::new("par", "a"));
    g.add_edge(Edge::new("par", "b"));

    let node = g.get_node("par").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "par".into(),
        }
    })?;
    let ctx = Context::new();

    handler.execute(node, &ctx, &g).await?;

    let events = emitter.events();
    let started_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, PipelineEvent::ParallelStarted { .. }))
        .collect();

    assert_eq!(started_events.len(), 1);
    if let PipelineEvent::ParallelStarted {
        dynamic_item_count, ..
    } = &started_events[0]
    {
        assert_eq!(*dynamic_item_count, None);
    }

    Ok(())
}

// ===========================================================================
// Success/fail counts for dynamic fan-out
// ===========================================================================

#[tokio::test]
async fn dynamic_fan_out_success_fail_counts() -> AttractorResult<()> {
    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let g = dynamic_fan_out_graph();
    let node = g.get_node("fan_out_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "fan_out_node".into(),
        }
    })?;
    let ctx = Context::new();
    ctx.set("items", serde_json::json!(["a", "b", "c"]));

    let outcome = handler.execute(node, &ctx, &g).await?;

    let success_count = outcome
        .context_updates
        .get("parallel.success_count")
        .and_then(|v| v.as_u64());
    let fail_count = outcome
        .context_updates
        .get("parallel.fail_count")
        .and_then(|v| v.as_u64());
    assert_eq!(success_count, Some(3));
    assert_eq!(fail_count, Some(0));

    Ok(())
}

// ===========================================================================
// Empty-list bypass sets success/fail counts to 0
// ===========================================================================

#[tokio::test]
async fn dynamic_fan_out_empty_list_counts() -> AttractorResult<()> {
    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let g = dynamic_fan_out_graph();
    let node = g.get_node("fan_out_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "fan_out_node".into(),
        }
    })?;
    let ctx = Context::new();
    ctx.set("items", serde_json::json!([]));

    let outcome = handler.execute(node, &ctx, &g).await?;

    let success_count = outcome
        .context_updates
        .get("parallel.success_count")
        .and_then(|v| v.as_u64());
    let fail_count = outcome
        .context_updates
        .get("parallel.fail_count")
        .and_then(|v| v.as_u64());
    assert_eq!(success_count, Some(0));
    assert_eq!(fail_count, Some(0));

    Ok(())
}

// ===========================================================================
// Validation: missing fan-in warns
// ===========================================================================

#[test]
fn validation_dynamic_fan_out_missing_fan_in_warns() {
    let mut g = Graph::new("test_no_fan_in");

    let mut start = Node::new("Start");
    start
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);

    let mut fan = Node::new("fan_out_node");
    fan.attrs
        .insert("shape".into(), AttrValue::from("component"));
    fan.attrs.insert("fan_out".into(), AttrValue::from("items"));
    g.add_node(fan);

    // Template node but NO fan-in downstream
    g.add_node(Node::new("template"));

    let mut end = Node::new("End");
    end.attrs.insert("shape".into(), AttrValue::from("Msquare"));
    g.add_node(end);

    g.add_edge(Edge::new("Start", "fan_out_node"));
    g.add_edge(Edge::new("fan_out_node", "template"));
    g.add_edge(Edge::new("template", "End"));

    let diagnostics = validation::validate(&g, &[]);
    let missing_fan_in: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.rule == "dynamic_fan_out_missing_fan_in")
        .collect();

    assert_eq!(missing_fan_in.len(), 1);
    assert_eq!(missing_fan_in[0].severity, validation::Severity::Warning);
    assert!(missing_fan_in[0].message.contains("fan_out_node"));
}

// ===========================================================================
// Validation: present fan-in does NOT warn
// ===========================================================================

#[test]
fn validation_dynamic_fan_out_with_fan_in_no_warning() {
    let mut g = Graph::new("test_with_fan_in");

    let mut start = Node::new("Start");
    start
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);

    let mut fan = Node::new("fan_out_node");
    fan.attrs
        .insert("shape".into(), AttrValue::from("component"));
    fan.attrs.insert("fan_out".into(), AttrValue::from("items"));
    g.add_node(fan);

    g.add_node(Node::new("template"));

    let mut fi = Node::new("merge");
    fi.attrs
        .insert("shape".into(), AttrValue::from("tripleoctagon"));
    g.add_node(fi);

    let mut end = Node::new("End");
    end.attrs.insert("shape".into(), AttrValue::from("Msquare"));
    g.add_node(end);

    g.add_edge(Edge::new("Start", "fan_out_node"));
    g.add_edge(Edge::new("fan_out_node", "template"));
    g.add_edge(Edge::new("template", "merge"));
    g.add_edge(Edge::new("merge", "End"));

    let diagnostics = validation::validate(&g, &[]);
    let missing_fan_in: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.rule == "dynamic_fan_out_missing_fan_in")
        .collect();

    assert_eq!(
        missing_fan_in.len(),
        0,
        "should not warn when fan-in is present"
    );
}
