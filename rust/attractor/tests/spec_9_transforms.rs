//! Tests for transforms (§9.1–9.3).

mod common;

use std::sync::Arc;

use async_trait::async_trait;

use stencila_attractor::context::Context;
use stencila_attractor::engine::{self, EngineConfig};
use stencila_attractor::error::AttractorResult;
use stencila_attractor::graph::{AttrValue, Edge, Graph, Node};
use stencila_attractor::handlers::{CodergenBackend, CodergenHandler, CodergenOutput};
use stencila_attractor::transform::{Transform, TransformRegistry};
use stencila_attractor::transforms::VariableExpansionTransform;
use stencila_attractor::types::StageStatus;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A test transform that appends a suffix to all node labels.
struct SuffixTransform {
    suffix: String,
}

impl SuffixTransform {
    fn new(suffix: &str) -> Self {
        Self {
            suffix: suffix.to_string(),
        }
    }
}

impl Transform for SuffixTransform {
    fn name(&self) -> &str {
        "suffix"
    }

    fn apply(&self, graph: &mut Graph) -> AttractorResult<()> {
        for node in graph.nodes.values_mut() {
            if let Some(AttrValue::String(label)) = node.attrs.get_mut("label") {
                label.push_str(&self.suffix);
            }
        }
        Ok(())
    }
}

/// A transform that records its name in a graph attribute for ordering tests.
struct RecordingTransform {
    tag: String,
}

impl RecordingTransform {
    fn new(tag: &str) -> Self {
        Self {
            tag: tag.to_string(),
        }
    }
}

impl Transform for RecordingTransform {
    fn name(&self) -> &str {
        &self.tag
    }

    fn apply(&self, graph: &mut Graph) -> AttractorResult<()> {
        // Append our tag to a graph attr to prove execution order
        let existing = graph
            .get_graph_attr("execution_order")
            .and_then(AttrValue::as_str)
            .unwrap_or("")
            .to_string();
        let updated = if existing.is_empty() {
            self.tag.clone()
        } else {
            format!("{existing},{}", self.tag)
        };
        graph
            .graph_attrs
            .insert("execution_order".into(), AttrValue::from(updated));
        Ok(())
    }
}

// ===========================================================================
// Transform trait basic
// ===========================================================================

#[test]
fn transform_trait_basic_apply() -> AttractorResult<()> {
    let mut g = Graph::new("test");
    let mut n = Node::new("task1");
    n.attrs.insert("label".into(), AttrValue::from("Hello"));
    g.add_node(n);

    let t = SuffixTransform::new("_done");
    t.apply(&mut g)?;

    let label = g
        .get_node("task1")
        .and_then(|n| n.get_str_attr("label"))
        .unwrap_or("");
    assert_eq!(label, "Hello_done");
    Ok(())
}

// ===========================================================================
// Registry ordering
// ===========================================================================

#[test]
fn registry_ordering_builtins_before_custom() -> AttractorResult<()> {
    let mut registry = TransformRegistry::new();
    registry.register_builtin(RecordingTransform::new("builtin1"));
    registry.register_builtin(RecordingTransform::new("builtin2"));
    registry.register_custom(RecordingTransform::new("custom1"));

    let mut g = Graph::new("test");
    registry.apply_all(&mut g)?;

    let order = g
        .get_graph_attr("execution_order")
        .and_then(AttrValue::as_str)
        .unwrap_or("");
    assert_eq!(order, "builtin1,builtin2,custom1");
    Ok(())
}

#[test]
fn registry_apply_all_executes_all() -> AttractorResult<()> {
    let mut registry = TransformRegistry::new();
    registry.register_builtin(RecordingTransform::new("A"));
    registry.register_custom(RecordingTransform::new("B"));
    registry.register_custom(RecordingTransform::new("C"));

    let mut g = Graph::new("test");
    registry.apply_all(&mut g)?;

    let order = g
        .get_graph_attr("execution_order")
        .and_then(AttrValue::as_str)
        .unwrap_or("");
    assert_eq!(order, "A,B,C");
    Ok(())
}

// ===========================================================================
// Variable expansion
// ===========================================================================

#[test]
fn variable_expansion_replaces_goal() -> AttractorResult<()> {
    let mut g = Graph::new("test");
    g.graph_attrs
        .insert("goal".into(), AttrValue::from("Build a widget"));

    let mut n = Node::new("task1");
    n.attrs
        .insert("prompt".into(), AttrValue::from("Please $goal for me"));
    g.add_node(n);

    VariableExpansionTransform.apply(&mut g)?;

    let prompt = g
        .get_node("task1")
        .and_then(|n| n.get_str_attr("prompt"))
        .unwrap_or("");
    assert_eq!(prompt, "Please Build a widget for me");
    Ok(())
}

#[test]
fn variable_expansion_no_goal_uses_empty() -> AttractorResult<()> {
    let mut g = Graph::new("test");
    // No graph-level "goal" attribute

    let mut n = Node::new("task1");
    n.attrs
        .insert("prompt".into(), AttrValue::from("Goal: $goal"));
    g.add_node(n);

    VariableExpansionTransform.apply(&mut g)?;

    let prompt = g
        .get_node("task1")
        .and_then(|n| n.get_str_attr("prompt"))
        .unwrap_or("");
    assert_eq!(prompt, "Goal: ");
    Ok(())
}

#[test]
fn variable_expansion_multiple_nodes() -> AttractorResult<()> {
    let mut g = Graph::new("test");
    g.graph_attrs.insert("goal".into(), AttrValue::from("test"));

    let mut n1 = Node::new("a");
    n1.attrs.insert("prompt".into(), AttrValue::from("$goal A"));
    g.add_node(n1);

    let mut n2 = Node::new("b");
    n2.attrs.insert("prompt".into(), AttrValue::from("$goal B"));
    g.add_node(n2);

    VariableExpansionTransform.apply(&mut g)?;

    let p1 = g
        .get_node("a")
        .and_then(|n| n.get_str_attr("prompt"))
        .unwrap_or("");
    let p2 = g
        .get_node("b")
        .and_then(|n| n.get_str_attr("prompt"))
        .unwrap_or("");
    assert_eq!(p1, "test A");
    assert_eq!(p2, "test B");
    Ok(())
}

#[test]
fn variable_expansion_no_prompt_attr_noop() -> AttractorResult<()> {
    let mut g = Graph::new("test");
    g.graph_attrs
        .insert("goal".into(), AttrValue::from("something"));

    let n = Node::new("task1"); // No prompt attribute
    g.add_node(n);

    // Should not error
    VariableExpansionTransform.apply(&mut g)?;

    // Node unchanged
    assert!(
        g.get_node("task1")
            .and_then(|n| n.get_str_attr("prompt"))
            .is_none()
    );
    Ok(())
}

#[test]
fn variable_expansion_multiple_occurrences() -> AttractorResult<()> {
    let mut g = Graph::new("test");
    g.graph_attrs.insert("goal".into(), AttrValue::from("X"));

    let mut n = Node::new("task1");
    n.attrs
        .insert("prompt".into(), AttrValue::from("$goal and $goal again"));
    g.add_node(n);

    VariableExpansionTransform.apply(&mut g)?;

    let prompt = g
        .get_node("task1")
        .and_then(|n| n.get_str_attr("prompt"))
        .unwrap_or("");
    assert_eq!(prompt, "X and X again");
    Ok(())
}

// ===========================================================================
// Engine integration: transforms applied before execution
// ===========================================================================

/// A backend that captures the prompt it receives.
struct CapturingBackend {
    captured: std::sync::Mutex<Vec<String>>,
}

impl CapturingBackend {
    fn new() -> Self {
        Self {
            captured: std::sync::Mutex::new(Vec::new()),
        }
    }

    fn prompts(&self) -> Vec<String> {
        self.captured.lock().map(|v| v.clone()).unwrap_or_default()
    }
}

#[async_trait]
impl CodergenBackend for CapturingBackend {
    async fn run(
        &self,
        _node: &Node,
        prompt: &str,
        _context: &Context,
        _emitter: std::sync::Arc<dyn stencila_attractor::events::EventEmitter>,
        _stage_index: usize,
    ) -> AttractorResult<CodergenOutput> {
        if let Ok(mut captured) = self.captured.lock() {
            captured.push(prompt.to_string());
        }
        Ok(CodergenOutput::Text("ok".to_string()))
    }
}

#[tokio::test]
async fn engine_applies_variable_expansion() -> AttractorResult<()> {
    let tmp = common::make_tempdir()?;

    let mut g = Graph::new("test");
    g.graph_attrs
        .insert("goal".into(), AttrValue::from("build a widget"));

    let mut start = Node::new("start");
    start
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);

    let mut task = Node::new("task");
    task.attrs
        .insert("prompt".into(), AttrValue::from("Please $goal now"));
    g.add_node(task);

    let mut exit = Node::new("exit");
    exit.attrs
        .insert("shape".into(), AttrValue::from("Msquare"));
    g.add_node(exit);

    g.add_edge(Edge::new("start", "task"));
    g.add_edge(Edge::new("task", "exit"));

    let backend = Arc::new(CapturingBackend::new());
    let handler = CodergenHandler::with_backend(backend.clone());

    let mut config = EngineConfig::new(tmp.path());
    config.registry.register("codergen", handler);

    let outcome = engine::run(&g, config).await?;
    assert_eq!(outcome.status, StageStatus::Success);

    // The backend should have received the expanded prompt, not "$goal"
    let prompts = backend.prompts();
    assert_eq!(prompts.len(), 1);
    assert_eq!(prompts[0], "Please build a widget now");
    Ok(())
}
