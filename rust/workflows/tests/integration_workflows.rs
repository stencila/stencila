//! Integration tests that run the `test-*` workflows from
//! `.stencila/workflows/` against a real LLM backend.
//!
//! These tests are `#[ignore]`d by default because they require:
//!   - Model API keys (e.g. `ANTHROPIC_API_KEY`)
//!   - Network access
//!   - Non-trivial wall-clock time
//!
//! Run them explicitly with:
//!
//! ```sh
//! cargo test -p stencila-workflows --test integration_workflows -- --ignored
//! ```
//!
//! Or run a single test:
//!
//! ```sh
//! cargo test -p stencila-workflows --test integration_workflows test_shell_nodes -- --ignored
//! ```

use std::sync::Arc;

use stencila_attractor::events::NoOpEmitter;
use stencila_attractor::interviewer::Interviewer;
use stencila_interviews::interviewers::AutoApproveInterviewer;

use stencila_workflows::{RunOptions, WorkflowInstance, get_by_name, run_workflow_with_options};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Resolve the repository root so `get_by_name` finds `.stencila/workflows/`.
///
/// Integration tests run with `cwd` set to `rust/workflows/`, but workflow
/// discovery looks for `.stencila/` upwards. We walk up until we find the
/// `.stencila/workflows` directory.
fn repo_root() -> std::path::PathBuf {
    let mut dir = std::env::current_dir().expect("current_dir");
    loop {
        if dir.join(".stencila/workflows").is_dir() {
            return dir;
        }
        if !dir.pop() {
            panic!("Could not find repository root with .stencila/workflows/");
        }
    }
}

async fn load_workflow(name: &str) -> WorkflowInstance {
    let root = repo_root();
    get_by_name(&root, name)
        .await
        .unwrap_or_else(|e| panic!("Failed to load workflow `{name}`: {e}"))
}

fn run_options() -> RunOptions {
    RunOptions {
        emitter: Arc::new(NoOpEmitter),
        interviewer: Some(Arc::new(AutoApproveInterviewer) as Arc<dyn Interviewer>),
        run_id_out: None,
        gate_timeout: stencila_workflows::GateTimeoutConfig::default(),
    }
}

/// Run a workflow and assert it succeeds.
async fn run_and_assert_success(name: &str) {
    let wf = load_workflow(name).await;
    let outcome = run_workflow_with_options(
        &wf,
        RunOptions {
            emitter: stencila_workflows::stderr_event_emitter_for_testing(),
            interviewer: run_options().interviewer,
            run_id_out: None,
            gate_timeout: stencila_workflows::GateTimeoutConfig::default(),
        },
    )
    .await
    .unwrap_or_else(|e| panic!("Workflow `{name}` execution error: {e}"));
    assert!(
        outcome.status.is_success(),
        "Workflow `{name}` failed: status={}, reason={}",
        outcome.status.as_str(),
        outcome.failure_reason,
    );
}

// ===========================================================================
// Tests that require NO model API key (no LLM calls)
// ===========================================================================

/// Start → End, no LLM calls.
#[tokio::test]
async fn test_no_op() {
    run_and_assert_success("test-no-op").await;
}

/// Shell command nodes only, no LLM calls.
#[tokio::test]
async fn test_shell_nodes() {
    run_and_assert_success("test-shell-nodes").await;
}

/// Parent workflow composing a child workflow via `workflow=` sugar.
#[tokio::test]
async fn test_workflow_composition() {
    run_and_assert_success("test-compose").await;
}

/// goal_gate=true with retryTarget loopback, shell only.
#[tokio::test]
async fn test_goal_gates() {
    run_and_assert_success("test-goal-gates").await;
}

/// max_retries=N node attribute and defaultMaxRetry frontmatter, shell only.
#[tokio::test]
async fn test_max_retries() {
    run_and_assert_success("test-max-retries").await;
}

/// Edge weight-based routing priority, shell only.
#[tokio::test]
async fn test_edge_weights() {
    run_and_assert_success("test-edge-weights").await;
}

/// Multi-way context.* edge conditions with fallback edge, shell only.
#[tokio::test]
async fn test_context_conditions() {
    run_and_assert_success("test-context-conditions").await;
}

/// Human-in-the-loop gates, no LLM calls. AutoApproveInterviewer selects
/// the first option at each gate.
#[tokio::test]
async fn test_human_gates() {
    run_and_assert_success("test-human-gates").await;
}

/// Dynamic fan-out over a JSON array produced by a shell node, no LLM calls.
#[tokio::test]
async fn test_fan_out_dynamic_shell() {
    run_and_assert_success("test-fan-out-dynamic-shell").await;
}

// ===========================================================================
// Tests that require a model API key
// ===========================================================================

/// Linear chain with $last_output and $goal expansion.
#[tokio::test]
#[ignore]
async fn test_count_to_three() {
    run_and_assert_success("test-count-to-three").await;
}

/// Looping with conditional branching on context.last_output.
#[tokio::test]
#[ignore]
async fn test_count_to_goal() {
    let wf = {
        let mut wf = load_workflow("test-count-to-goal").await;
        // Use a small goal so the loop terminates quickly.
        wf.inner.goal = Some("3".to_string());
        wf
    };
    let outcome = run_workflow_with_options(&wf, run_options())
        .await
        .unwrap_or_else(|e| panic!("Workflow execution error: {e}"));
    assert!(
        outcome.status.is_success(),
        "test-count-to-goal failed: status={}, reason={}",
        outcome.status.as_str(),
        outcome.failure_reason,
    );
}

/// Parallel fan-out/fan-in.
#[tokio::test]
#[ignore]
async fn test_fan_out_fan_in() {
    run_and_assert_success("test-fan-out-fan-in").await;
}

/// Check* diamond node with outcome-based edge conditions.
#[tokio::test]
#[ignore]
async fn test_conditional_branching() {
    run_and_assert_success("test-conditional-branching").await;
}

/// overrides frontmatter with *, .class, and #id selectors.
#[tokio::test]
#[ignore]
async fn test_overrides() {
    run_and_assert_success("test-overrides").await;
}

/// Subgraph scoping of node defaults.
#[tokio::test]
#[ignore]
async fn test_subgraph_defaults() {
    run_and_assert_success("test-subgraph-defaults").await;
}

/// agent= node attribute for named agent resolution.
#[tokio::test]
#[ignore]
async fn test_agent_reference() {
    run_and_assert_success("test-agent-reference").await;
}

/// Dynamic fan-out over a JSON array produced by an agent via workflow_set_context.
#[tokio::test]
#[ignore]
async fn test_fan_out_dynamic_agent() {
    run_and_assert_success("test-fan-out-dynamic-agent").await;
}

/// Combined pipeline with 10+ nodes: shell, parallel, conditional,
/// human gate, max_retries, model stylesheet.
#[tokio::test]
#[ignore]
async fn test_kitchen_sink() {
    run_and_assert_success("test-kitchen-sink").await;
}
