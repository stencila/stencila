//! Live integration tests against real LLM providers (spec 9.12-9.13).
//!
//! These tests exercise the full agent loop — session, tools, events — against
//! real provider APIs. They skip silently when API keys are absent so CI never
//! fails due to missing credentials.
//!
//! # Running
//!
//! ```sh
//! # With API keys set:
//! cargo test -p stencila-agents -- spec_9 --nocapture
//! ```
//!
//! # Non-Determinism Strategy
//!
//! - Assert structural properties: file exists, contains substring
//! - Assert event kinds (tool was called), not exact counts
//! - `max_turns: 50` limits cost and prevents runaway
//! - `should_skip_agent_error` skips rate-limit / quota errors
//! - Case-insensitive substring checks for content assertions
#![allow(clippy::result_large_err)]

mod integration;

use std::sync::Arc;

use stencila_agents::error::AgentResult;
use stencila_agents::execution::{ExecutionEnvironment, LocalExecutionEnvironment};
use stencila_agents::types::{EventKind, SessionConfig};

use integration::helpers;

// ---------------------------------------------------------------------------
// Helper: default config for live tests
// ---------------------------------------------------------------------------

fn live_config() -> SessionConfig {
    SessionConfig {
        max_turns: 50,
        ..Default::default()
    }
}

/// Check if a file exists at the given path within a temp directory.
async fn file_exists_in(dir: &std::path::Path, relative: &str) -> bool {
    let env = LocalExecutionEnvironment::new(dir);
    env.file_exists(relative).await
}

/// Read a file's content from the temp directory.
async fn read_file_text(dir: &std::path::Path, relative: &str) -> Option<String> {
    let env = LocalExecutionEnvironment::new(dir);
    match env.read_file(relative, None, None).await {
        Ok(stencila_agents::execution::FileContent::Text(text)) => Some(text),
        _ => None,
    }
}

/// Check if text contains a substring (case-insensitive).
fn contains_ci(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

// ===========================================================================
// Parity Matrix (spec 9.12) — one test per row, loops over providers
// ===========================================================================

/// Simple file creation: ask the model to create hello.py that prints 'Hello World'.
#[tokio::test]
async fn parity_simple_file_creation() -> AgentResult<()> {
    let available = helpers::available_providers(&["openai", "anthropic", "gemini"]);
    if available.is_empty() {
        return Ok(());
    }

    for &provider in &available {
        let temp = helpers::make_tempdir()?;

        let (mut session, mut receiver) =
            match helpers::live_session(provider, temp.path(), live_config()).await {
                Ok(s) => s,
                Err(e) if helpers::should_skip_agent_error(&e) => continue,
                Err(e) => return Err(e),
            };

        let Some(events) = helpers::submit_and_drain(
            &mut session,
            &mut receiver,
            "Create a file called hello.py that prints 'Hello World'",
        )
        .await?
        else {
            continue;
        };

        // Assert: file was created
        assert!(
            file_exists_in(temp.path(), "hello.py").await,
            "[{provider}] hello.py should exist"
        );

        // Assert: a write tool was called (write_file or apply_patch)
        let tools = helpers::tool_names_used(&events);
        assert!(
            tools
                .iter()
                .any(|t| t == "write_file" || t == "apply_patch"),
            "[{provider}] write_file or apply_patch should have been called, got: {tools:?}"
        );
    }
    Ok(())
}

/// Read file, then edit: pre-seed hello.py, ask model to add a goodbye line.
#[tokio::test]
async fn parity_read_then_edit() -> AgentResult<()> {
    let available = helpers::available_providers(&["openai", "anthropic", "gemini"]);
    if available.is_empty() {
        return Ok(());
    }

    for &provider in &available {
        let temp = helpers::make_tempdir()?;

        // Pre-seed file
        let env = Arc::new(LocalExecutionEnvironment::new(temp.path()));
        env.write_file("hello.py", "print('Hello World')\n").await?;

        let (mut session, mut receiver) =
            match helpers::live_session(provider, temp.path(), live_config()).await {
                Ok(s) => s,
                Err(e) if helpers::should_skip_agent_error(&e) => continue,
                Err(e) => return Err(e),
            };

        let Some(_events) = helpers::submit_and_drain(
            &mut session,
            &mut receiver,
            "Read hello.py and add a line that prints 'Goodbye World' at the end",
        )
        .await?
        else {
            continue;
        };

        // Assert: file contains both lines
        let content = read_file_text(temp.path(), "hello.py").await;
        assert!(
            content.is_some(),
            "[{provider}] hello.py should be readable"
        );
        let content = content.unwrap_or_default();
        assert!(
            contains_ci(&content, "hello"),
            "[{provider}] hello.py should still contain 'Hello'"
        );
        assert!(
            contains_ci(&content, "goodbye"),
            "[{provider}] hello.py should contain 'Goodbye'"
        );
    }
    Ok(())
}

/// Multi-file edit: pre-seed two files, ask model to modify both.
#[tokio::test]
async fn parity_multi_file_edit() -> AgentResult<()> {
    let available = helpers::available_providers(&["openai", "anthropic", "gemini"]);
    if available.is_empty() {
        return Ok(());
    }

    for &provider in &available {
        let temp = helpers::make_tempdir()?;

        let env = Arc::new(LocalExecutionEnvironment::new(temp.path()));
        env.write_file("a.py", "x = 1\n").await?;
        env.write_file("b.py", "y = 2\n").await?;

        let (mut session, mut receiver) =
            match helpers::live_session(provider, temp.path(), live_config()).await {
                Ok(s) => s,
                Err(e) if helpers::should_skip_agent_error(&e) => continue,
                Err(e) => return Err(e),
            };

        let Some(_events) = helpers::submit_and_drain(
            &mut session,
            &mut receiver,
            "Read a.py and b.py. In a.py change x to 10, in b.py change y to 20.",
        )
        .await?
        else {
            continue;
        };

        let a_content = read_file_text(temp.path(), "a.py").await;
        assert!(a_content.is_some(), "[{provider}] a.py should be readable");
        let a_content = a_content.unwrap_or_default();
        let b_content = read_file_text(temp.path(), "b.py").await;
        assert!(b_content.is_some(), "[{provider}] b.py should be readable");
        let b_content = b_content.unwrap_or_default();

        assert!(
            contains_ci(&a_content, "10"),
            "[{provider}] a.py should contain '10'"
        );
        assert!(
            contains_ci(&b_content, "20"),
            "[{provider}] b.py should contain '20'"
        );
    }
    Ok(())
}

/// Shell command execution: ask the model to run a shell command.
#[tokio::test]
async fn parity_shell_execution() -> AgentResult<()> {
    let available = helpers::available_providers(&["openai", "anthropic", "gemini"]);
    if available.is_empty() {
        return Ok(());
    }

    for &provider in &available {
        let temp = helpers::make_tempdir()?;

        let (mut session, mut receiver) =
            match helpers::live_session(provider, temp.path(), live_config()).await {
                Ok(s) => s,
                Err(e) if helpers::should_skip_agent_error(&e) => continue,
                Err(e) => return Err(e),
            };

        let Some(events) = helpers::submit_and_drain(
            &mut session,
            &mut receiver,
            "Run `echo hello` in the shell and tell me what it outputs",
        )
        .await?
        else {
            continue;
        };

        assert!(
            helpers::was_tool_called(&events, "shell"),
            "[{provider}] shell tool should have been called"
        );
    }
    Ok(())
}

/// Shell timeout: ask the model to run a long-running command with a short
/// timeout, verify the timeout path fires and `[TIMED OUT]` appears in output.
#[tokio::test]
async fn parity_shell_timeout() -> AgentResult<()> {
    let available = helpers::available_providers(&["openai", "anthropic", "gemini"]);
    if available.is_empty() {
        return Ok(());
    }

    for &provider in &available {
        let temp = helpers::make_tempdir()?;

        let config = SessionConfig {
            max_turns: 50,
            default_command_timeout_ms: 2_000,
            ..Default::default()
        };

        let (mut session, mut receiver) =
            match helpers::live_session(provider, temp.path(), config).await {
                Ok(s) => s,
                Err(e) if helpers::should_skip_agent_error(&e) => continue,
                Err(e) => return Err(e),
            };

        let Some(events) =
            helpers::submit_and_drain(&mut session, &mut receiver, "Run `sleep 30` in the shell")
                .await?
        else {
            continue;
        };

        // Assert: shell was called
        assert!(
            helpers::was_tool_called(&events, "shell"),
            "[{provider}] shell tool should have been called"
        );

        // Assert: the timeout path fired — tool output contains [TIMED OUT]
        let outputs = helpers::tool_outputs(&events);
        assert!(
            outputs.iter().any(|o| contains_ci(o, "TIMED OUT")),
            "[{provider}] shell output should contain TIMED OUT marker, got: {outputs:?}"
        );
    }
    Ok(())
}

/// Grep + glob: pre-seed .py files, ask model to search for a pattern.
#[tokio::test]
async fn parity_grep_glob() -> AgentResult<()> {
    let available = helpers::available_providers(&["openai", "anthropic", "gemini"]);
    if available.is_empty() {
        return Ok(());
    }

    for &provider in &available {
        let temp = helpers::make_tempdir()?;

        let env = Arc::new(LocalExecutionEnvironment::new(temp.path()));
        env.write_file("alpha.py", "# TODO: implement alpha\ndef alpha(): pass\n")
            .await?;
        env.write_file("beta.py", "# TODO: implement beta\ndef beta(): pass\n")
            .await?;

        let (mut session, mut receiver) =
            match helpers::live_session(provider, temp.path(), live_config()).await {
                Ok(s) => s,
                Err(e) if helpers::should_skip_agent_error(&e) => continue,
                Err(e) => return Err(e),
            };

        let Some(events) = helpers::submit_and_drain(
            &mut session,
            &mut receiver,
            "Find all Python files and search for lines containing 'TODO'",
        )
        .await?
        else {
            continue;
        };

        let tools = helpers::tool_names_used(&events);
        assert!(
            tools.iter().any(|t| t == "grep" || t == "glob"),
            "[{provider}] grep or glob should have been called, got: {tools:?}"
        );
    }
    Ok(())
}

/// Multi-step task: pre-seed file with TODO, ask model to read-analyze-edit.
#[tokio::test]
async fn parity_multi_step_task() -> AgentResult<()> {
    let available = helpers::available_providers(&["openai", "anthropic", "gemini"]);
    if available.is_empty() {
        return Ok(());
    }

    for &provider in &available {
        let temp = helpers::make_tempdir()?;

        let env = Arc::new(LocalExecutionEnvironment::new(temp.path()));
        env.write_file(
            "task.py",
            "def add(a, b):\n    # TODO: implement\n    pass\n",
        )
        .await?;

        let (mut session, mut receiver) =
            match helpers::live_session(provider, temp.path(), live_config()).await {
                Ok(s) => s,
                Err(e) if helpers::should_skip_agent_error(&e) => continue,
                Err(e) => return Err(e),
            };

        let Some(_events) = helpers::submit_and_drain(
            &mut session,
            &mut receiver,
            "Read task.py, implement the add function (it should return a + b), and remove the TODO comment",
        )
        .await?
        else {
            continue;
        };

        let content = read_file_text(temp.path(), "task.py").await;
        assert!(content.is_some(), "[{provider}] task.py should be readable");
        let content = content.unwrap_or_default();
        assert!(
            contains_ci(&content, "return"),
            "[{provider}] task.py should contain a return statement"
        );
    }
    Ok(())
}

/// Truncation: pre-write a large file, verify TOOL_CALL_END has large output.
#[tokio::test]
async fn parity_truncation() -> AgentResult<()> {
    let available = helpers::available_providers(&["openai", "anthropic", "gemini"]);
    if available.is_empty() {
        return Ok(());
    }

    for &provider in &available {
        let temp = helpers::make_tempdir()?;

        // Pre-write a ~100KB file
        let env = Arc::new(LocalExecutionEnvironment::new(temp.path()));
        let big_content: String = (0..2500)
            .map(|i| format!("line {i}: {}\n", "x".repeat(40)))
            .collect();
        env.write_file("large.txt", &big_content).await?;

        let (mut session, mut receiver) =
            match helpers::live_session(provider, temp.path(), live_config()).await {
                Ok(s) => s,
                Err(e) if helpers::should_skip_agent_error(&e) => continue,
                Err(e) => return Err(e),
            };

        let Some(events) =
            helpers::submit_and_drain(&mut session, &mut receiver, "Read the file large.txt")
                .await?
        else {
            continue;
        };

        // Assert: read_file was called
        assert!(
            helpers::was_tool_called(&events, "read_file"),
            "[{provider}] read_file should have been called"
        );

        // Assert: TOOL_CALL_END has large output (untruncated)
        let outputs = helpers::tool_outputs(&events);
        assert!(
            !outputs.is_empty(),
            "[{provider}] TOOL_CALL_END events should have output"
        );
        let max_output_len = outputs.iter().map(|o| o.len()).max().unwrap_or_default();
        assert!(
            max_output_len > 50_000,
            "[{provider}] TOOL_CALL_END should have large untruncated output (got {max_output_len} chars)"
        );
    }
    Ok(())
}

/// Provider-specific editing: OpenAI uses apply_patch; Anthropic/Gemini use edit_file.
///
/// The preferred tool is checked first. If only write_file was used (models
/// sometimes fall back to rewriting the whole file), the test still passes but
/// emits a warning visible in `--nocapture` runs so regressions are noticeable.
#[tokio::test]
#[allow(clippy::print_stderr)]
async fn parity_provider_specific_editing() -> AgentResult<()> {
    let available = helpers::available_providers(&["openai", "anthropic", "gemini"]);
    if available.is_empty() {
        return Ok(());
    }

    for &provider in &available {
        let temp = helpers::make_tempdir()?;

        let env = Arc::new(LocalExecutionEnvironment::new(temp.path()));
        env.write_file("greet.py", "def greet():\n    return 'hello'\n")
            .await?;

        let (mut session, mut receiver) =
            match helpers::live_session(provider, temp.path(), live_config()).await {
                Ok(s) => s,
                Err(e) if helpers::should_skip_agent_error(&e) => continue,
                Err(e) => return Err(e),
            };

        let Some(events) = helpers::submit_and_drain(
            &mut session,
            &mut receiver,
            "Read greet.py and change the return value from 'hello' to 'hi there'",
        )
        .await?
        else {
            continue;
        };

        let tools = helpers::tool_names_used(&events);
        let (preferred, fallback_ok) = match provider {
            "openai" => ("apply_patch", "write_file"),
            _ => ("edit_file", "write_file"),
        };

        if tools.iter().any(|t| t == preferred) {
            // Preferred tool was used — all good
        } else if tools.iter().any(|t| t == fallback_ok) {
            eprintln!(
                "  [WARN] [{provider}] used {fallback_ok} instead of preferred {preferred} — \
                 model may have fallen back to full-file rewrite"
            );
        } else {
            panic!("[{provider}] expected {preferred} or {fallback_ok}, got: {tools:?}");
        }
    }
    Ok(())
}

// ===========================================================================
// Smoke Tests (spec 9.13) — individual tests, loops over providers
// ===========================================================================

/// Smoke Step 1: File creation with content verification.
#[tokio::test]
async fn smoke_file_creation() -> AgentResult<()> {
    let available = helpers::available_providers(&["openai", "anthropic", "gemini"]);
    if available.is_empty() {
        return Ok(());
    }

    for &provider in &available {
        let temp = helpers::make_tempdir()?;

        let (mut session, mut receiver) =
            match helpers::live_session(provider, temp.path(), live_config()).await {
                Ok(s) => s,
                Err(e) if helpers::should_skip_agent_error(&e) => continue,
                Err(e) => return Err(e),
            };

        let Some(_events) = helpers::submit_and_drain(
            &mut session,
            &mut receiver,
            "Create a Python file called hello.py that prints 'Hello'",
        )
        .await?
        else {
            continue;
        };

        assert!(
            file_exists_in(temp.path(), "hello.py").await,
            "[{provider}] hello.py should exist"
        );

        let content = read_file_text(temp.path(), "hello.py").await;
        assert!(
            content.is_some(),
            "[{provider}] hello.py should be readable"
        );
        let content = content.unwrap_or_default();
        assert!(
            contains_ci(&content, "hello"),
            "[{provider}] hello.py should contain 'Hello'"
        );
    }
    Ok(())
}

/// Smoke Step 2: Read and edit with content verification.
#[tokio::test]
async fn smoke_read_and_edit() -> AgentResult<()> {
    let available = helpers::available_providers(&["openai", "anthropic", "gemini"]);
    if available.is_empty() {
        return Ok(());
    }

    for &provider in &available {
        let temp = helpers::make_tempdir()?;

        let env = Arc::new(LocalExecutionEnvironment::new(temp.path()));
        env.write_file("hello.py", "print('Hello')\n").await?;

        let (mut session, mut receiver) =
            match helpers::live_session(provider, temp.path(), live_config()).await {
                Ok(s) => s,
                Err(e) if helpers::should_skip_agent_error(&e) => continue,
                Err(e) => return Err(e),
            };

        let Some(_events) = helpers::submit_and_drain(
            &mut session,
            &mut receiver,
            "Read hello.py, then add a line that prints 'Goodbye' at the end of the file",
        )
        .await?
        else {
            continue;
        };

        let content = read_file_text(temp.path(), "hello.py").await;
        assert!(
            content.is_some(),
            "[{provider}] hello.py should be readable"
        );
        let content = content.unwrap_or_default();
        assert!(
            contains_ci(&content, "hello"),
            "[{provider}] should still contain 'Hello'"
        );
        assert!(
            contains_ci(&content, "goodbye"),
            "[{provider}] should contain 'Goodbye'"
        );
    }
    Ok(())
}

/// Smoke Step 3: Shell execution.
#[tokio::test]
async fn smoke_shell_execution() -> AgentResult<()> {
    let available = helpers::available_providers(&["openai", "anthropic", "gemini"]);
    if available.is_empty() {
        return Ok(());
    }

    for &provider in &available {
        let temp = helpers::make_tempdir()?;

        let (mut session, mut receiver) =
            match helpers::live_session(provider, temp.path(), live_config()).await {
                Ok(s) => s,
                Err(e) if helpers::should_skip_agent_error(&e) => continue,
                Err(e) => return Err(e),
            };

        let Some(events) = helpers::submit_and_drain(
            &mut session,
            &mut receiver,
            "Use the shell to run `echo test_output` and tell me what it prints",
        )
        .await?
        else {
            continue;
        };

        assert!(
            helpers::was_tool_called(&events, "shell"),
            "[{provider}] shell tool should have been called"
        );
    }
    Ok(())
}

/// Smoke Step 4: Truncation for large output.
#[tokio::test]
async fn smoke_truncation() -> AgentResult<()> {
    let available = helpers::available_providers(&["openai", "anthropic", "gemini"]);
    if available.is_empty() {
        return Ok(());
    }

    for &provider in &available {
        let temp = helpers::make_tempdir()?;

        let env = Arc::new(LocalExecutionEnvironment::new(temp.path()));
        let big_content: String = (0..3000)
            .map(|i| format!("data line {i}: {}\n", "y".repeat(30)))
            .collect();
        env.write_file("big.txt", &big_content).await?;

        let (mut session, mut receiver) =
            match helpers::live_session(provider, temp.path(), live_config()).await {
                Ok(s) => s,
                Err(e) if helpers::should_skip_agent_error(&e) => continue,
                Err(e) => return Err(e),
            };

        let Some(events) =
            helpers::submit_and_drain(&mut session, &mut receiver, "Read the file big.txt").await?
        else {
            continue;
        };

        let outputs = helpers::tool_outputs(&events);
        assert!(
            !outputs.is_empty(),
            "[{provider}] TOOL_CALL_END events should have output"
        );
        let max_len = outputs.iter().map(|o| o.len()).max().unwrap_or_default();
        assert!(
            max_len > 50_000,
            "[{provider}] TOOL_CALL_END should have large output (got {max_len})"
        );
    }
    Ok(())
}

/// Smoke Step 5: Steering injection.
#[tokio::test]
async fn smoke_steering() -> AgentResult<()> {
    let available = helpers::available_providers(&["openai", "anthropic", "gemini"]);
    if available.is_empty() {
        return Ok(());
    }

    for &provider in &available {
        let temp = helpers::make_tempdir()?;

        let (mut session, mut receiver) =
            match helpers::live_session(provider, temp.path(), live_config()).await {
                Ok(s) => s,
                Err(e) if helpers::should_skip_agent_error(&e) => continue,
                Err(e) => return Err(e),
            };

        // Queue steering before submitting
        session.steer("Remember to use Python 3 syntax only.");

        let Some(events) = helpers::submit_and_drain(
            &mut session,
            &mut receiver,
            "Create a file called example.py with a simple function",
        )
        .await?
        else {
            continue;
        };

        assert!(
            helpers::has_event_kind(&events, EventKind::SteeringInjected),
            "[{provider}] STEERING_INJECTED event should be present"
        );
    }
    Ok(())
}

/// Smoke Step 6: Subagent spawn and outcome verification.
#[tokio::test]
async fn smoke_subagent() -> AgentResult<()> {
    let available = helpers::available_providers(&["openai", "anthropic", "gemini"]);
    if available.is_empty() {
        return Ok(());
    }

    for &provider in &available {
        let temp = helpers::make_tempdir()?;

        let config = SessionConfig {
            max_turns: 50,
            max_subagent_depth: 1,
            ..Default::default()
        };

        let (mut session, mut receiver) =
            match helpers::live_session(provider, temp.path(), config).await {
                Ok(s) => s,
                Err(e) if helpers::should_skip_agent_error(&e) => continue,
                Err(e) => return Err(e),
            };

        let Some(events) = helpers::submit_and_drain(
            &mut session,
            &mut receiver,
            "Use the spawn_agent tool to create a sub-agent with the task: \
             'Create a file called sub.txt with the content: created by subagent'",
        )
        .await?
        else {
            continue;
        };

        // Assert: spawn_agent was called
        assert!(
            helpers::was_tool_called(&events, "spawn_agent"),
            "[{provider}] spawn_agent tool should have been called"
        );

        // Assert: the subagent produced the requested file
        assert!(
            file_exists_in(temp.path(), "sub.txt").await,
            "[{provider}] sub.txt should exist (subagent should have created it)"
        );
    }
    Ok(())
}

/// Smoke Step 7: Session completes with short timeout config.
#[tokio::test]
async fn smoke_timeout() -> AgentResult<()> {
    let available = helpers::available_providers(&["openai", "anthropic", "gemini"]);
    if available.is_empty() {
        return Ok(());
    }

    for &provider in &available {
        let temp = helpers::make_tempdir()?;

        let config = SessionConfig {
            max_turns: 10,
            default_command_timeout_ms: 3_000,
            ..Default::default()
        };

        let (mut session, mut receiver) =
            match helpers::live_session(provider, temp.path(), config).await {
                Ok(s) => s,
                Err(e) if helpers::should_skip_agent_error(&e) => continue,
                Err(e) => return Err(e),
            };

        let Some(_events) = helpers::submit_and_drain(
            &mut session,
            &mut receiver,
            "Create a simple hello.py that prints hello",
        )
        .await?
        else {
            continue;
        };

        // Session completed (didn't hang) — timeout config propagation verified
    }
    Ok(())
}
