//! Tests for codergen handler (§4.5) and shell handler (§4.10).

mod common;

use std::sync::Arc;

use async_trait::async_trait;

use stencila_attractor::context::Context;
use stencila_attractor::engine::{self, EngineConfig};
use stencila_attractor::error::{AttractorError, AttractorResult};
use stencila_attractor::graph::{AttrValue, Edge, Graph, Node};
use stencila_attractor::handler::Handler;
use stencila_attractor::handlers::{
    CodergenBackend, CodergenHandler, CodergenOutput, ShellHandler,
};
use stencila_attractor::types::{Duration, Outcome, StageStatus};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// A backend that returns a preconfigured output.
struct MockBackend {
    output: CodergenOutput,
}

impl MockBackend {
    fn text(s: &str) -> Self {
        Self {
            output: CodergenOutput::Text(s.to_string()),
        }
    }

    fn full_outcome(outcome: Outcome) -> Self {
        Self {
            output: CodergenOutput::FullOutcome(outcome),
        }
    }
}

#[async_trait]
impl CodergenBackend for MockBackend {
    async fn run(
        &self,
        _node: &Node,
        _prompt: &str,
        _context: &Context,
        _emitter: std::sync::Arc<dyn stencila_attractor::events::EventEmitter>,
        _stage_index: usize,
    ) -> AttractorResult<CodergenOutput> {
        // Clone the inner data manually since CodergenOutput doesn't impl Clone
        match &self.output {
            CodergenOutput::Text(s) => Ok(CodergenOutput::Text(s.clone())),
            CodergenOutput::FullOutcome(o) => Ok(CodergenOutput::FullOutcome(o.clone())),
        }
    }
}

/// A backend that always returns an error.
struct ErrorBackend {
    message: String,
}

impl ErrorBackend {
    fn new(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
        }
    }
}

#[async_trait]
impl CodergenBackend for ErrorBackend {
    async fn run(
        &self,
        _node: &Node,
        _prompt: &str,
        _context: &Context,
        _emitter: std::sync::Arc<dyn stencila_attractor::events::EventEmitter>,
        _stage_index: usize,
    ) -> AttractorResult<CodergenOutput> {
        Err(AttractorError::HandlerFailed {
            node_id: "test".into(),
            reason: self.message.clone(),
        })
    }
}

/// Build a minimal start→middle→exit graph for end-to-end tests.
fn pipeline_with_middle(middle: Node) -> Graph {
    let mut g = Graph::new("test");

    let mut start = Node::new("start");
    start
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);

    g.add_node(middle);

    let mut exit = Node::new("exit");
    exit.attrs
        .insert("shape".into(), AttrValue::from("Msquare"));
    g.add_node(exit);

    g.add_edge(Edge::new("start", "middle"));
    g.add_edge(Edge::new("middle", "exit"));
    g
}

// ===========================================================================
// Codergen handler tests
// ===========================================================================

#[tokio::test]
async fn codergen_simulation_mode() -> AttractorResult<()> {
    let handler = CodergenHandler::simulation();
    let node = Node::new("task1");
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    assert!(outcome.notes.contains("task1"));
    Ok(())
}

#[tokio::test]
async fn codergen_prompt_from_attr() -> AttractorResult<()> {
    let handler = CodergenHandler::simulation();
    let mut node = Node::new("task1");
    node.attrs
        .insert("prompt".into(), AttrValue::from("Write a function"));
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g).await?;
    assert_eq!(outcome.status, StageStatus::Success);
    Ok(())
}

#[tokio::test]
async fn codergen_prompt_fallback_to_label() -> AttractorResult<()> {
    let handler = CodergenHandler::simulation();
    let mut node = Node::new("task1");
    node.attrs
        .insert("label".into(), AttrValue::from("Generate Code"));
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g).await?;
    assert_eq!(outcome.status, StageStatus::Success);
    Ok(())
}

#[tokio::test]
async fn codergen_backend_returns_text() -> AttractorResult<()> {
    let backend = Arc::new(MockBackend::text("Hello from LLM"));
    let handler = CodergenHandler::with_backend(backend);
    let node = Node::new("task1");
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    Ok(())
}

#[tokio::test]
async fn codergen_backend_returns_outcome() -> AttractorResult<()> {
    let custom_outcome = Outcome::fail("custom failure");
    let backend = Arc::new(MockBackend::full_outcome(custom_outcome.clone()));
    let handler = CodergenHandler::with_backend(backend);
    let node = Node::new("task1");
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Fail);
    assert_eq!(outcome.failure_reason, "custom failure");
    Ok(())
}

#[tokio::test]
async fn codergen_backend_error() -> AttractorResult<()> {
    let backend = Arc::new(ErrorBackend::new("LLM is down"));
    let handler = CodergenHandler::with_backend(backend);
    let node = Node::new("task1");
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g).await?;

    // Backend errors are caught and returned as FAIL (not propagated as Err)
    assert_eq!(outcome.status, StageStatus::Fail);
    assert!(outcome.failure_reason.contains("Backend error"));
    Ok(())
}

#[tokio::test]
async fn codergen_context_updates() -> AttractorResult<()> {
    let backend = Arc::new(MockBackend::text("some output"));
    let handler = CodergenHandler::with_backend(backend);
    let node = Node::new("task1");
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g).await?;

    let last_stage = outcome
        .context_updates
        .get("last_stage")
        .and_then(|v| v.as_str());
    assert_eq!(last_stage, Some("task1"));

    let last_output = outcome
        .context_updates
        .get("last_output")
        .and_then(|v| v.as_str());
    assert_eq!(last_output, Some("some output"));
    Ok(())
}

#[tokio::test]
async fn codergen_output_truncation() -> AttractorResult<()> {
    // Create an output longer than 200 chars
    let long_output = "x".repeat(300);
    let backend = Arc::new(MockBackend::text(&long_output));
    let handler = CodergenHandler::with_backend(backend);
    let node = Node::new("task1");
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g).await?;

    let last_output = outcome
        .context_updates
        .get("last_output")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    // Truncated to 200 + "..."
    assert_eq!(last_output.len(), 203);
    assert!(last_output.ends_with("..."));
    Ok(())
}

#[tokio::test]
async fn codergen_end_to_end() -> AttractorResult<()> {
    let middle = Node::new("middle"); // default shape "box" → "codergen"
    let g = pipeline_with_middle(middle);

    let config = EngineConfig::new();
    // with_defaults() now includes codergen simulation handler
    let outcome = engine::run(&g, config).await?;
    assert_eq!(outcome.status, StageStatus::Success);
    Ok(())
}

#[tokio::test]
async fn codergen_truncation_non_ascii_safe() -> AttractorResult<()> {
    // Build a string with multi-byte chars that would panic if sliced at byte 200.
    // Each emoji is 4 bytes; 50 emojis = 200 bytes. Add one more so byte 200
    // falls in the middle of the 51st emoji.
    let output: String = std::iter::repeat_n('\u{1F600}', 51).collect(); // 204 bytes
    assert!(output.len() > 200);

    let backend = Arc::new(MockBackend::text(&output));
    let handler = CodergenHandler::with_backend(backend);
    let node = Node::new("task1");
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g).await?;

    let last_output = outcome
        .context_updates
        .get("last_output")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    // Should be truncated without panicking, at a char boundary
    assert!(last_output.ends_with("..."));
    // 50 emojis (200 bytes) + "..." = valid truncation
    assert!(last_output.is_char_boundary(last_output.len()));
    Ok(())
}

// ===========================================================================
// $KEY context variable expansion tests
// ===========================================================================

/// A backend that captures the expanded prompt for verification.
struct CapturingBackend {
    captured: std::sync::Mutex<Vec<String>>,
}

impl CapturingBackend {
    fn new() -> Self {
        Self {
            captured: std::sync::Mutex::new(Vec::new()),
        }
    }

    fn captured(&self) -> Vec<String> {
        self.captured
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
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
        self.captured
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(prompt.to_string());
        Ok(CodergenOutput::Text("ok".to_string()))
    }
}

#[tokio::test]
async fn context_variable_expansion_basic() -> AttractorResult<()> {
    let backend = Arc::new(CapturingBackend::new());
    let handler = CodergenHandler::with_backend(backend.clone());

    let mut node = Node::new("task1");
    node.attrs.insert(
        "prompt".into(),
        AttrValue::from("Fix this: $human.feedback"),
    );
    let ctx = Context::new();
    ctx.set(
        "human.feedback",
        serde_json::Value::String("Add error handling".into()),
    );
    let g = Graph::new("test");

    handler.execute(&node, &ctx, &g).await?;

    let prompts = backend.captured();
    assert_eq!(prompts.len(), 1);
    assert_eq!(prompts[0], "Fix this: Add error handling");
    Ok(())
}

#[tokio::test]
async fn context_variable_expansion_missing_key_is_preserved() -> AttractorResult<()> {
    let backend = Arc::new(CapturingBackend::new());
    let handler = CodergenHandler::with_backend(backend.clone());

    let mut node = Node::new("task1");
    node.attrs.insert(
        "prompt".into(),
        AttrValue::from("Feedback: $human.feedback"),
    );
    let ctx = Context::new();
    // Do not set human.feedback — should remain unchanged
    let g = Graph::new("test");

    handler.execute(&node, &ctx, &g).await?;

    let prompts = backend.captured();
    assert_eq!(prompts[0], "Feedback: $human.feedback");
    Ok(())
}

#[tokio::test]
async fn shell_preserves_unknown_shell_variables() -> AttractorResult<()> {
    let handler = ShellHandler::new();
    let mut node = Node::new("tool1");
    node.attrs.insert(
        "shell_command".into(),
        AttrValue::from("COUNT=1; echo $COUNT"),
    );
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g).await?;
    assert_eq!(outcome.status, StageStatus::Success);

    let output = outcome
        .context_updates
        .get("last_output")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(output, "1");

    Ok(())
}

#[tokio::test]
async fn context_variable_expansion_multiple() -> AttractorResult<()> {
    let backend = Arc::new(CapturingBackend::new());
    let handler = CodergenHandler::with_backend(backend.clone());

    let mut node = Node::new("task1");
    node.attrs.insert(
        "prompt".into(),
        AttrValue::from("Goal: $goal_text, Feedback: $human.feedback"),
    );
    let ctx = Context::new();
    ctx.set(
        "goal_text",
        serde_json::Value::String("build a widget".into()),
    );
    ctx.set(
        "human.feedback",
        serde_json::Value::String("make it blue".into()),
    );
    let g = Graph::new("test");

    handler.execute(&node, &ctx, &g).await?;

    let prompts = backend.captured();
    assert_eq!(prompts[0], "Goal: build a widget, Feedback: make it blue");
    Ok(())
}

#[tokio::test]
async fn context_variable_expansion_no_dollar_passthrough() -> AttractorResult<()> {
    let backend = Arc::new(CapturingBackend::new());
    let handler = CodergenHandler::with_backend(backend.clone());

    let mut node = Node::new("task1");
    node.attrs
        .insert("prompt".into(), AttrValue::from("Just a normal prompt"));
    let ctx = Context::new();
    let g = Graph::new("test");

    handler.execute(&node, &ctx, &g).await?;

    let prompts = backend.captured();
    assert_eq!(prompts[0], "Just a normal prompt");
    Ok(())
}

#[tokio::test]
async fn context_variable_expansion_non_ident_dollar_passthrough() -> AttractorResult<()> {
    let backend = Arc::new(CapturingBackend::new());
    let handler = CodergenHandler::with_backend(backend.clone());

    let mut node = Node::new("task1");
    // "$50" and trailing "$" should pass through literally
    node.attrs
        .insert("prompt".into(), AttrValue::from("costs $50 and $"));
    let ctx = Context::new();
    let g = Graph::new("test");

    handler.execute(&node, &ctx, &g).await?;

    let prompts = backend.captured();
    assert_eq!(prompts[0], "costs $50 and $");
    Ok(())
}

#[tokio::test]
async fn context_variable_expansion_underscore_key() -> AttractorResult<()> {
    let backend = Arc::new(CapturingBackend::new());
    let handler = CodergenHandler::with_backend(backend.clone());

    let mut node = Node::new("task1");
    node.attrs
        .insert("prompt".into(), AttrValue::from("val: $_my_key"));
    let ctx = Context::new();
    ctx.set("_my_key", serde_json::Value::String("works".into()));
    let g = Graph::new("test");

    handler.execute(&node, &ctx, &g).await?;

    let prompts = backend.captured();
    assert_eq!(prompts[0], "val: works");
    Ok(())
}

// ===========================================================================
// Shell handler tests
// ===========================================================================

#[tokio::test]
async fn shell_executes_command() -> AttractorResult<()> {
    let handler = ShellHandler::new();
    let mut node = Node::new("tool1");
    node.attrs
        .insert("shell_command".into(), AttrValue::from("echo hello"));
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g).await?;
    assert_eq!(outcome.status, StageStatus::Success);
    Ok(())
}

#[tokio::test]
async fn shell_captures_stdout() -> AttractorResult<()> {
    let handler = ShellHandler::new();
    let mut node = Node::new("tool1");
    node.attrs
        .insert("shell_command".into(), AttrValue::from("echo hello"));
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g).await?;

    let output = outcome
        .context_updates
        .get("shell.output")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(output, "hello");

    // last_output and last_output_full should also be set
    let last_output = outcome
        .context_updates
        .get("last_output")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(last_output, "hello");

    let last_output_full = outcome
        .context_updates
        .get("last_output_full")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(last_output_full, "hello");

    // last_stage should be the node id
    let last_stage = outcome
        .context_updates
        .get("last_stage")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(last_stage, "tool1");

    Ok(())
}

#[tokio::test]
async fn shell_expands_context_variables() -> AttractorResult<()> {
    let handler = ShellHandler::new();
    let mut node = Node::new("tool1");
    node.attrs
        .insert("shell_command".into(), AttrValue::from("echo $last_output"));
    let ctx = Context::new();
    ctx.set(
        "last_output_full",
        serde_json::Value::String("world".into()),
    );
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g).await?;
    assert_eq!(outcome.status, StageStatus::Success);

    let output = outcome
        .context_updates
        .get("last_output")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(output, "world");
    Ok(())
}

#[tokio::test]
async fn shell_missing_command() -> AttractorResult<()> {
    let handler = ShellHandler::new();
    let node = Node::new("tool1"); // No shell_command
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g).await?;
    assert_eq!(outcome.status, StageStatus::Fail);
    assert!(outcome.failure_reason.contains("shell_command"));
    Ok(())
}

#[tokio::test]
async fn shell_nonzero_exit() -> AttractorResult<()> {
    let handler = ShellHandler::new();
    let mut node = Node::new("tool1");
    node.attrs.insert(
        "shell_command".into(),
        AttrValue::from("echo fail >&2 && exit 1"),
    );
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g).await?;
    assert_eq!(outcome.status, StageStatus::Fail);
    assert!(outcome.failure_reason.contains("non-zero"));
    Ok(())
}

#[tokio::test]
async fn shell_timeout_expires() -> AttractorResult<()> {
    let handler = ShellHandler::new();
    let g = Graph::new("test");

    // Duration variant (unquoted in DOT: timeout=100ms)
    let mut node = Node::new("tool1");
    node.attrs
        .insert("shell_command".into(), AttrValue::from("sleep 10"));
    node.attrs.insert(
        "timeout".into(),
        AttrValue::Duration(Duration::from_spec_str("100ms")?),
    );
    let outcome = handler.execute(&node, &Context::new(), &g).await?;
    assert_eq!(outcome.status, StageStatus::Fail);
    assert!(outcome.failure_reason.contains("timed out"));

    // String variant (quoted in DOT: timeout="100ms")
    let mut node = Node::new("tool2");
    node.attrs
        .insert("shell_command".into(), AttrValue::from("sleep 10"));
    node.attrs
        .insert("timeout".into(), AttrValue::from("100ms"));
    let outcome = handler.execute(&node, &Context::new(), &g).await?;
    assert_eq!(outcome.status, StageStatus::Fail);
    assert!(outcome.failure_reason.contains("timed out"));

    Ok(())
}

#[tokio::test]
async fn shell_empty_stdout() -> AttractorResult<()> {
    let handler = ShellHandler::new();
    let mut node = Node::new("tool1");
    node.attrs
        .insert("shell_command".into(), AttrValue::from("true"));
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g).await?;
    assert_eq!(outcome.status, StageStatus::Success);

    let output = outcome
        .context_updates
        .get("shell.output")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(output.is_empty());
    Ok(())
}

#[tokio::test]
async fn shell_end_to_end() -> AttractorResult<()> {
    let mut middle = Node::new("middle");
    middle
        .attrs
        .insert("shape".into(), AttrValue::from("parallelogram"));
    middle
        .attrs
        .insert("shell_command".into(), AttrValue::from("echo done"));
    let g = pipeline_with_middle(middle);

    let config = EngineConfig::new();
    let outcome = engine::run(&g, config).await?;
    assert_eq!(outcome.status, StageStatus::Success);
    Ok(())
}

// ===========================================================================
// Shell handler: store / store_as attribute
// ===========================================================================

#[tokio::test]
async fn shell_store_json_array_auto_parsed() -> AttractorResult<()> {
    let handler = ShellHandler::new();
    let mut node = Node::new("seed");
    node.attrs.insert(
        "shell_command".into(),
        AttrValue::from(r#"echo '["alpha","beta","gamma"]'"#),
    );
    node.attrs
        .insert("store".into(), AttrValue::from("my_items"));
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g).await?;
    assert_eq!(outcome.status, StageStatus::Success);

    let stored = outcome.context_updates.get("my_items");
    assert!(stored.is_some(), "store key should be in context_updates");
    let arr = stored.and_then(|v| v.as_array());
    assert!(arr.is_some(), "value should be a JSON array");
    assert_eq!(arr.map(Vec::len), Some(3));
    Ok(())
}

#[tokio::test]
async fn shell_store_plain_text_fallback() -> AttractorResult<()> {
    let handler = ShellHandler::new();
    let mut node = Node::new("greet");
    node.attrs
        .insert("shell_command".into(), AttrValue::from("echo hello world"));
    node.attrs
        .insert("store".into(), AttrValue::from("greeting"));
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g).await?;
    assert_eq!(outcome.status, StageStatus::Success);

    let stored = outcome.context_updates.get("greeting");
    assert_eq!(
        stored,
        Some(&serde_json::Value::String("hello world".into())),
    );
    Ok(())
}

#[tokio::test]
async fn shell_store_as_json_valid() -> AttractorResult<()> {
    let handler = ShellHandler::new();
    let mut node = Node::new("seed");
    node.attrs.insert(
        "shell_command".into(),
        AttrValue::from(r#"echo '{"name":"rust"}'"#),
    );
    node.attrs
        .insert("store".into(), AttrValue::from("item"));
    node.attrs
        .insert("store_as".into(), AttrValue::from("json"));
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g).await?;
    assert_eq!(outcome.status, StageStatus::Success);

    let stored = outcome.context_updates.get("item");
    assert!(stored.is_some());
    let obj = stored.and_then(|v| v.as_object());
    assert!(obj.is_some(), "value should be a JSON object");
    assert_eq!(
        obj.and_then(|o| o.get("name")).and_then(|v| v.as_str()),
        Some("rust"),
    );
    Ok(())
}

#[tokio::test]
async fn shell_store_as_json_invalid_fails() -> AttractorResult<()> {
    let handler = ShellHandler::new();
    let mut node = Node::new("bad");
    node.attrs
        .insert("shell_command".into(), AttrValue::from("echo not-json"));
    node.attrs
        .insert("store".into(), AttrValue::from("item"));
    node.attrs
        .insert("store_as".into(), AttrValue::from("json"));
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g).await?;
    assert_eq!(outcome.status, StageStatus::Fail);
    assert!(outcome.failure_reason.contains("store_as=json"));
    Ok(())
}

#[tokio::test]
async fn shell_store_as_string_skips_parsing() -> AttractorResult<()> {
    let handler = ShellHandler::new();
    let mut node = Node::new("num");
    node.attrs
        .insert("shell_command".into(), AttrValue::from("echo 42"));
    node.attrs
        .insert("store".into(), AttrValue::from("count"));
    node.attrs
        .insert("store_as".into(), AttrValue::from("string"));
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g).await?;
    assert_eq!(outcome.status, StageStatus::Success);

    let stored = outcome.context_updates.get("count");
    assert_eq!(stored, Some(&serde_json::Value::String("42".into())));
    Ok(())
}

#[tokio::test]
async fn shell_store_auto_parses_number() -> AttractorResult<()> {
    let handler = ShellHandler::new();
    let mut node = Node::new("num");
    node.attrs
        .insert("shell_command".into(), AttrValue::from("echo 42"));
    node.attrs
        .insert("store".into(), AttrValue::from("count"));
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g).await?;
    assert_eq!(outcome.status, StageStatus::Success);

    let stored = outcome.context_updates.get("count");
    assert_eq!(
        stored,
        Some(&serde_json::Value::Number(serde_json::Number::from(42))),
    );
    Ok(())
}
