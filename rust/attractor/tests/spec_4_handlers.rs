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

use common::make_tempdir;

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
    let tmp = make_tempdir()?;
    let logs_root = tmp.path();
    std::fs::create_dir_all(logs_root.join("nodes"))?;

    let handler = CodergenHandler::simulation();
    let node = Node::new("task1");
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g, logs_root).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    assert!(outcome.notes.contains("task1"));

    // Check the output file contains simulated text
    let output = std::fs::read_to_string(logs_root.join("nodes/task1/output.md"))?;
    assert!(output.contains("[Simulated]"));
    assert!(output.contains("task1"));
    Ok(())
}

#[tokio::test]
async fn codergen_prompt_from_attr() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let logs_root = tmp.path();
    std::fs::create_dir_all(logs_root.join("nodes"))?;

    let handler = CodergenHandler::simulation();
    let mut node = Node::new("task1");
    node.attrs
        .insert("prompt".into(), AttrValue::from("Write a function"));
    let ctx = Context::new();
    let g = Graph::new("test");

    handler.execute(&node, &ctx, &g, logs_root).await?;

    let input = std::fs::read_to_string(logs_root.join("nodes/task1/input.md"))?;
    assert_eq!(input, "Write a function");
    Ok(())
}

#[tokio::test]
async fn codergen_prompt_fallback_to_label() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let logs_root = tmp.path();
    std::fs::create_dir_all(logs_root.join("nodes"))?;

    let handler = CodergenHandler::simulation();
    let mut node = Node::new("task1");
    node.attrs
        .insert("label".into(), AttrValue::from("Generate Code"));
    // No "prompt" attribute
    let ctx = Context::new();
    let g = Graph::new("test");

    handler.execute(&node, &ctx, &g, logs_root).await?;

    let input = std::fs::read_to_string(logs_root.join("nodes/task1/input.md"))?;
    assert_eq!(input, "Generate Code");
    Ok(())
}

#[tokio::test]
async fn codergen_backend_returns_text() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let logs_root = tmp.path();
    std::fs::create_dir_all(logs_root.join("nodes"))?;

    let backend = Arc::new(MockBackend::text("Hello from LLM"));
    let handler = CodergenHandler::with_backend(backend);
    let node = Node::new("task1");
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g, logs_root).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    let output = std::fs::read_to_string(logs_root.join("nodes/task1/output.md"))?;
    assert_eq!(output, "Hello from LLM");
    Ok(())
}

#[tokio::test]
async fn codergen_backend_returns_outcome() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let logs_root = tmp.path();
    std::fs::create_dir_all(logs_root.join("nodes"))?;

    let custom_outcome = Outcome::fail("custom failure");
    let backend = Arc::new(MockBackend::full_outcome(custom_outcome.clone()));
    let handler = CodergenHandler::with_backend(backend);
    let node = Node::new("task1");
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g, logs_root).await?;

    assert_eq!(outcome.status, StageStatus::Fail);
    assert_eq!(outcome.failure_reason, "custom failure");
    Ok(())
}

#[tokio::test]
async fn codergen_backend_error() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let logs_root = tmp.path();
    std::fs::create_dir_all(logs_root.join("nodes"))?;

    let backend = Arc::new(ErrorBackend::new("LLM is down"));
    let handler = CodergenHandler::with_backend(backend);
    let node = Node::new("task1");
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g, logs_root).await?;

    // Backend errors are caught and returned as FAIL (not propagated as Err)
    assert_eq!(outcome.status, StageStatus::Fail);
    assert!(outcome.failure_reason.contains("Backend error"));
    Ok(())
}

#[tokio::test]
async fn codergen_writes_input_md() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let logs_root = tmp.path();
    std::fs::create_dir_all(logs_root.join("nodes"))?;

    let handler = CodergenHandler::simulation();
    let mut node = Node::new("task1");
    node.attrs
        .insert("prompt".into(), AttrValue::from("My custom prompt"));
    let ctx = Context::new();
    let g = Graph::new("test");

    handler.execute(&node, &ctx, &g, logs_root).await?;

    let path = logs_root.join("nodes/task1/input.md");
    assert!(path.exists());
    let content = std::fs::read_to_string(path)?;
    assert_eq!(content, "My custom prompt");
    Ok(())
}

#[tokio::test]
async fn codergen_writes_output_md() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let logs_root = tmp.path();
    std::fs::create_dir_all(logs_root.join("nodes"))?;

    let backend = Arc::new(MockBackend::text("LLM output text"));
    let handler = CodergenHandler::with_backend(backend);
    let node = Node::new("task1");
    let ctx = Context::new();
    let g = Graph::new("test");

    handler.execute(&node, &ctx, &g, logs_root).await?;

    let path = logs_root.join("nodes/task1/output.md");
    assert!(path.exists());
    let content = std::fs::read_to_string(path)?;
    assert_eq!(content, "LLM output text");
    Ok(())
}

#[tokio::test]
async fn codergen_writes_status_json() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let logs_root = tmp.path();
    std::fs::create_dir_all(logs_root.join("nodes"))?;

    let handler = CodergenHandler::simulation();
    let node = Node::new("task1");
    let ctx = Context::new();
    let g = Graph::new("test");

    handler.execute(&node, &ctx, &g, logs_root).await?;

    let path = logs_root.join("nodes/task1/status.json");
    assert!(path.exists());
    let content = std::fs::read_to_string(path)?;
    let parsed: Outcome = serde_json::from_str(&content)?;
    assert_eq!(parsed.status, StageStatus::Success);
    Ok(())
}

#[tokio::test]
async fn codergen_context_updates() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let logs_root = tmp.path();
    std::fs::create_dir_all(logs_root.join("nodes"))?;

    let backend = Arc::new(MockBackend::text("some output"));
    let handler = CodergenHandler::with_backend(backend);
    let node = Node::new("task1");
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g, logs_root).await?;

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
    let tmp = make_tempdir()?;
    let logs_root = tmp.path();
    std::fs::create_dir_all(logs_root.join("nodes"))?;

    // Create an output longer than 200 chars
    let long_output = "x".repeat(300);
    let backend = Arc::new(MockBackend::text(&long_output));
    let handler = CodergenHandler::with_backend(backend);
    let node = Node::new("task1");
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g, logs_root).await?;

    let last_output = outcome
        .context_updates
        .get("last_output")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    // Truncated to 200 + "..."
    assert_eq!(last_output.len(), 203);
    assert!(last_output.ends_with("..."));

    // Full output is still written to file
    let full = std::fs::read_to_string(logs_root.join("nodes/task1/output.md"))?;
    assert_eq!(full.len(), 300);
    Ok(())
}

#[tokio::test]
async fn codergen_end_to_end() -> AttractorResult<()> {
    let tmp = make_tempdir()?;

    let middle = Node::new("middle"); // default shape "box" → "codergen"
    let g = pipeline_with_middle(middle);

    let config = EngineConfig::new(tmp.path());
    // with_defaults() now includes codergen simulation handler
    let outcome = engine::run(&g, config).await?;
    assert_eq!(outcome.status, StageStatus::Success);
    Ok(())
}

#[tokio::test]
async fn codergen_truncation_non_ascii_safe() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let logs_root = tmp.path();
    std::fs::create_dir_all(logs_root.join("nodes"))?;

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

    let outcome = handler.execute(&node, &ctx, &g, logs_root).await?;

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
// Shell handler tests
// ===========================================================================

#[tokio::test]
async fn shell_executes_command() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let handler = ShellHandler;
    let mut node = Node::new("tool1");
    node.attrs
        .insert("shell_command".into(), AttrValue::from("echo hello"));
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g, tmp.path()).await?;
    assert_eq!(outcome.status, StageStatus::Success);
    Ok(())
}

#[tokio::test]
async fn shell_captures_stdout() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let handler = ShellHandler;
    let mut node = Node::new("tool1");
    node.attrs
        .insert("shell_command".into(), AttrValue::from("echo hello"));
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g, tmp.path()).await?;

    let output = outcome
        .context_updates
        .get("shell.output")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(output.trim(), "hello");
    Ok(())
}

#[tokio::test]
async fn shell_missing_command() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let handler = ShellHandler;
    let node = Node::new("tool1"); // No shell_command
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g, tmp.path()).await?;
    assert_eq!(outcome.status, StageStatus::Fail);
    assert!(outcome.failure_reason.contains("shell_command"));
    Ok(())
}

#[tokio::test]
async fn shell_nonzero_exit() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let handler = ShellHandler;
    let mut node = Node::new("tool1");
    node.attrs.insert(
        "shell_command".into(),
        AttrValue::from("echo fail >&2 && exit 1"),
    );
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g, tmp.path()).await?;
    assert_eq!(outcome.status, StageStatus::Fail);
    assert!(outcome.failure_reason.contains("non-zero"));
    Ok(())
}

#[tokio::test]
async fn shell_timeout_expires() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let handler = ShellHandler;
    let g = Graph::new("test");

    // Duration variant (unquoted in DOT: timeout=100ms)
    let mut node = Node::new("tool1");
    node.attrs
        .insert("shell_command".into(), AttrValue::from("sleep 10"));
    node.attrs.insert(
        "timeout".into(),
        AttrValue::Duration(Duration::from_spec_str("100ms")?),
    );
    let outcome = handler
        .execute(&node, &Context::new(), &g, tmp.path())
        .await?;
    assert_eq!(outcome.status, StageStatus::Fail);
    assert!(outcome.failure_reason.contains("timed out"));

    // String variant (quoted in DOT: timeout="100ms")
    let mut node = Node::new("tool2");
    node.attrs
        .insert("shell_command".into(), AttrValue::from("sleep 10"));
    node.attrs
        .insert("timeout".into(), AttrValue::from("100ms"));
    let outcome = handler
        .execute(&node, &Context::new(), &g, tmp.path())
        .await?;
    assert_eq!(outcome.status, StageStatus::Fail);
    assert!(outcome.failure_reason.contains("timed out"));

    Ok(())
}

#[tokio::test]
async fn shell_empty_stdout() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let handler = ShellHandler;
    let mut node = Node::new("tool1");
    node.attrs
        .insert("shell_command".into(), AttrValue::from("true"));
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g, tmp.path()).await?;
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
    let tmp = make_tempdir()?;

    let mut middle = Node::new("middle");
    middle
        .attrs
        .insert("shape".into(), AttrValue::from("parallelogram"));
    middle
        .attrs
        .insert("shell_command".into(), AttrValue::from("echo done"));
    let g = pipeline_with_middle(middle);

    let config = EngineConfig::new(tmp.path());
    let outcome = engine::run(&g, config).await?;
    assert_eq!(outcome.status, StageStatus::Success);
    Ok(())
}
