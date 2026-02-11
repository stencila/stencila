//! Shared helpers for env-gated live integration tests.
//!
//! Mirrors the pattern from `stencila-models3/tests/integration/helpers.rs`:
//! tests skip silently when API keys are absent — no CI failures.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use stencila_agents::error::{AgentError, AgentResult};
use stencila_agents::events::EventReceiver;
use stencila_agents::execution::LocalExecutionEnvironment;
use stencila_agents::profiles::AnthropicProfile;
use stencila_agents::profiles::GeminiProfile;
use stencila_agents::profiles::OpenAiProfile;
use stencila_agents::prompts;
use stencila_agents::session::{Models3Client, Session};
use stencila_agents::types::{EventKind, SessionConfig, SessionEvent};
use stencila_models3::error::SdkError;

// ---------------------------------------------------------------------------
// Provider availability
// ---------------------------------------------------------------------------

/// Check whether the API key env-var for `provider` is set.
pub fn has_provider(provider: &str) -> bool {
    use stencila_models3::secret::get_secret;

    match provider {
        "openai" => get_secret("OPENAI_API_KEY").is_some(),
        "anthropic" => get_secret("ANTHROPIC_API_KEY").is_some(),
        "gemini" => {
            get_secret("GEMINI_API_KEY").is_some() || get_secret("GOOGLE_API_KEY").is_some()
        }
        _ => false,
    }
}

/// Filter a list of provider names to those whose API keys are set.
pub fn available_providers<'a>(names: &'a [&'a str]) -> Vec<&'a str> {
    names.iter().copied().filter(|n| has_provider(n)).collect()
}

/// Return the model ID to use for a given provider in integration tests.
///
/// Uses affordable, fast models to keep costs and latency low.
pub fn test_model(provider: &str) -> &'static str {
    match provider {
        "openai" => "gpt-4.1-mini",
        "anthropic" => "claude-sonnet-4-5-20250929",
        "gemini" => "gemini-2.0-flash",
        _ => "unknown",
    }
}

// ---------------------------------------------------------------------------
// Error classification
// ---------------------------------------------------------------------------

/// Whether a live-provider integration error should skip the current provider.
///
/// Env-gated tests should avoid hard failures for temporary provider-side
/// rate limits or quota exhaustion.
pub fn should_skip_agent_error(error: &AgentError) -> bool {
    match error {
        AgentError::Sdk(sdk_err) => matches!(
            sdk_err,
            SdkError::RateLimit { .. } | SdkError::QuotaExceeded { .. }
        ),
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Live session factory
// ---------------------------------------------------------------------------

/// Create a live session against a real LLM provider.
///
/// Builds the appropriate profile, execution environment, models3 client,
/// system prompt, and wires them into a `Session`. Returns the session
/// and its event receiver.
///
/// # Errors
///
/// Returns `AgentError::Sdk` if the models3 client cannot be constructed
/// (e.g. missing API key after availability check — race condition).
pub async fn live_session(
    provider: &str,
    working_dir: &Path,
    config: SessionConfig,
) -> AgentResult<(Session, EventReceiver)> {
    let model = test_model(provider);

    // Build profile
    let mut profile: Box<dyn stencila_agents::profile::ProviderProfile> = match provider {
        "openai" => Box::new(OpenAiProfile::new(model, 600_000)?),
        "anthropic" => Box::new(AnthropicProfile::new(model, 600_000)?),
        "gemini" => Box::new(GeminiProfile::new(model, 600_000)?),
        _ => {
            return Err(AgentError::Io {
                message: format!("unknown provider: {provider}"),
            });
        }
    };

    // Build execution environment
    let exec_env = Arc::new(LocalExecutionEnvironment::new(working_dir));

    // Build models3 client
    let client = stencila_models3::client::Client::from_env().map_err(AgentError::Sdk)?;
    let llm_client = Arc::new(Models3Client::new(client));

    // Build system prompt
    let system_prompt = prompts::build_system_prompt(&mut *profile, &*exec_env, true).await?;

    // Create session (depth 0 = top-level)
    let (session, receiver) = Session::new(profile, exec_env, llm_client, config, system_prompt, 0);

    Ok((session, receiver))
}

// ---------------------------------------------------------------------------
// Tempdir helper
// ---------------------------------------------------------------------------

/// Create a temporary directory, mapping the IO error to `AgentError`.
pub fn make_tempdir() -> AgentResult<tempfile::TempDir> {
    tempfile::tempdir().map_err(|e| AgentError::Io {
        message: e.to_string(),
    })
}

// ---------------------------------------------------------------------------
// Submit + drain helper
// ---------------------------------------------------------------------------

/// Submit a prompt, close the session, and drain events.
///
/// Returns `Ok(None)` if the provider should be skipped (rate limit / quota).
/// Returns `Ok(Some(events))` on success.
pub async fn submit_and_drain(
    session: &mut Session,
    receiver: &mut EventReceiver,
    prompt: &str,
) -> AgentResult<Option<Vec<SessionEvent>>> {
    match session.submit(prompt).await {
        Ok(()) => {}
        Err(e) if should_skip_agent_error(&e) => return Ok(None),
        Err(e) => return Err(e),
    }
    session.close();
    Ok(Some(drain_events(receiver).await))
}

// ---------------------------------------------------------------------------
// Event helpers
// ---------------------------------------------------------------------------

/// Drain all pending events from the receiver with a timeout.
///
/// Waits up to `timeout` for each successive event. Returns all events
/// collected before the timeout fires. The session should be closed
/// before calling this so the emitter is dropped and `recv()` returns
/// `None` promptly.
pub async fn drain_events(receiver: &mut EventReceiver) -> Vec<SessionEvent> {
    let timeout = Duration::from_millis(100);
    let mut events = Vec::new();
    while let Ok(Some(event)) = tokio::time::timeout(timeout, receiver.recv()).await {
        events.push(event);
    }
    events
}

/// Check if any event in the list has the given kind.
pub fn has_event_kind(events: &[SessionEvent], kind: EventKind) -> bool {
    events.iter().any(|e| e.kind == kind)
}

/// Extract the names of tools that were called from `TOOL_CALL_START` events.
pub fn tool_names_used(events: &[SessionEvent]) -> Vec<String> {
    events
        .iter()
        .filter(|e| e.kind == EventKind::ToolCallStart)
        .filter_map(|e| {
            e.data
                .get("tool_name")
                .and_then(|v| v.as_str())
                .map(String::from)
        })
        .collect()
}

/// Check if a specific tool was called in the events.
pub fn was_tool_called(events: &[SessionEvent], name: &str) -> bool {
    tool_names_used(events).iter().any(|n| n == name)
}

/// Get the full untruncated output from `TOOL_CALL_END` events.
pub fn tool_outputs(events: &[SessionEvent]) -> Vec<String> {
    events
        .iter()
        .filter(|e| e.kind == EventKind::ToolCallEnd)
        .filter_map(|e| {
            e.data
                .get("output")
                .and_then(|v| v.as_str())
                .map(String::from)
        })
        .collect()
}
