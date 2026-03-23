//! Tests for provider preference path, default path, and SessionOverrides
//! in `routing.rs` (Phase 3 / Slice 3).
//!
//! These tests verify:
//! - The `providers` selection path: iterates providers in order, selects
//!   first with credentials, uses default model. `selection_mechanism` is
//!   `ProviderPreference`.
//! - The default path: when nothing specified, existing behavior preserved.
//!   `selection_mechanism` is `Default`.
//! - `SessionOverrides` has `model_size` field, and model override produces
//!   the expected effective routing.

#![allow(clippy::result_large_err)]

use futures::future::BoxFuture;
use futures::stream::BoxStream;

use stencila_agents::convenience::SessionOverrides;
use stencila_agents::error::AgentResult;
use stencila_agents::routing::{SelectionMechanism, SessionRoute, route_session_explained};
use stencila_models3::error::SdkError;
use stencila_models3::provider::ProviderAdapter;
use stencila_models3::types::request::Request;
use stencila_models3::types::response::Response;
use stencila_models3::types::stream_event::StreamEvent;

// ── Helpers ──────────────────────────────────────────────────────────────

type SdkResult<T> = Result<T, SdkError>;

/// Minimal stub adapter for tests — just needs a name.
struct StubAdapter(String);

impl StubAdapter {
    fn new(name: &str) -> Self {
        Self(name.to_string())
    }
}

impl ProviderAdapter for StubAdapter {
    fn name(&self) -> &str {
        &self.0
    }

    fn complete(&self, _request: Request) -> BoxFuture<'_, SdkResult<Response>> {
        Box::pin(async {
            Err(SdkError::Configuration {
                message: "stub".into(),
            })
        })
    }

    fn stream(
        &self,
        _request: Request,
    ) -> BoxFuture<'_, SdkResult<BoxStream<'_, SdkResult<StreamEvent>>>> {
        Box::pin(async {
            Err(SdkError::Configuration {
                message: "stub".into(),
            })
        })
    }
}

/// Build a client with the given provider names registered (simulates having API keys).
fn client_with_providers(names: &[&str]) -> stencila_models3::client::Client {
    let mut builder = stencila_models3::client::Client::builder();
    for &name in names {
        builder = builder.add_provider_as(name, StubAdapter::new(name));
    }
    builder.build().expect("builder should succeed")
}

/// Build an empty client (no providers, no keys).
fn empty_client() -> stencila_models3::client::Client {
    stencila_models3::client::Client::builder()
        .build()
        .expect("empty builder should succeed")
}

// ── Test (a): providers [anthropic, openai] + only openai creds ──────────

#[test]
fn providers_path_selects_first_with_credentials() -> AgentResult<()> {
    // Only openai has credentials; anthropic does not.
    let client = client_with_providers(&["openai"]);
    let providers = vec!["anthropic".to_string(), "openai".to_string()];

    // No models, no model_size — only providers specified
    let decision = route_session_explained(None, Some(&providers), None, &client)?;

    // Should skip anthropic (no creds), select openai with resolved model
    match &decision.route {
        SessionRoute::Api { provider, model } => {
            assert_eq!(provider, "openai");
            assert!(
                model.starts_with("gpt-"),
                "resolved model should start with 'gpt-', got: {model}"
            );
        }
        other => panic!("expected Api route, got: {other:?}"),
    }
    assert_eq!(
        decision.selection_mechanism,
        SelectionMechanism::ProviderPreference,
    );
    Ok(())
}

#[test]
fn providers_path_any_falls_through_to_default() -> AgentResult<()> {
    let client = client_with_providers(&["anthropic"]);
    let providers = vec!["mistral".to_string(), "any".to_string()];

    let decision = route_session_explained(None, Some(&providers), None, &client)?;

    assert_eq!(decision.selection_mechanism, SelectionMechanism::Default);
    match &decision.route {
        SessionRoute::Api { provider, model } => {
            assert_eq!(provider, "anthropic");
            assert!(
                model.starts_with("claude-"),
                "resolved model should start with 'claude-', got: {model}"
            );
        }
        other => panic!("expected Api route, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn providers_path_without_any_remains_closed_set() -> AgentResult<()> {
    let client = client_with_providers(&["anthropic"]);
    let providers = vec!["mistral".to_string()];

    let result = route_session_explained(None, Some(&providers), None, &client);
    assert!(result.is_err());

    let err_msg = result.expect_err("should be an error").to_string();
    assert!(
        err_msg.contains("add 'any'"),
        "error should suggest using 'any': {err_msg}"
    );

    Ok(())
}

#[test]
fn model_size_uses_configured_provider_preferences_when_supplied() -> AgentResult<()> {
    let client = client_with_providers(&["openai", "anthropic"]);
    let configured_providers = vec!["openai".to_string(), "anthropic".to_string()];

    let decision =
        route_session_explained(None, Some(&configured_providers), Some("large"), &client)?;

    assert_eq!(
        decision.route,
        SessionRoute::Api {
            provider: "openai".into(),
            model: "gpt-5.4-pro".into(),
        }
    );
    assert_eq!(
        decision.selection_mechanism,
        SelectionMechanism::ModelSize {
            size: "large".to_string(),
        }
    );

    Ok(())
}

// ── Test (b): nothing specified → default behavior ───────────────────────

#[test]
fn default_path_with_api_key_selects_default_provider() -> AgentResult<()> {
    let client = client_with_providers(&["anthropic"]);

    // Nothing specified — no models, no providers, no model_size
    let decision = route_session_explained(None, None, None, &client)?;

    match &decision.route {
        SessionRoute::Api { provider, model } => {
            assert_eq!(provider, "anthropic");
            assert!(
                model.starts_with("claude-"),
                "resolved model should start with 'claude-', got: {model}"
            );
        }
        other => panic!("expected Api route, got: {other:?}"),
    }
    assert_eq!(decision.selection_mechanism, SelectionMechanism::Default);
    Ok(())
}

#[test]
fn default_path_no_keys_falls_back_to_cli_or_errors() -> AgentResult<()> {
    let client = empty_client();

    // Nothing specified and no API keys
    let result = route_session_explained(None, None, None, &client);

    match result {
        Ok(decision) => {
            // If a CLI tool is available, should use it with Default mechanism
            assert!(matches!(decision.route, SessionRoute::Cli { .. }));
            assert_eq!(decision.selection_mechanism, SelectionMechanism::Default);
        }
        Err(e) => {
            // If no CLI tools available, should error
            let msg = e.to_string();
            assert!(
                msg.contains("No API providers configured"),
                "error should mention missing providers: {msg}"
            );
        }
    }
    Ok(())
}

// ── Test (c): SessionOverrides model override → effective models routing ─

#[test]
fn session_overrides_model_produces_effective_models_routing() -> AgentResult<()> {
    let client = client_with_providers(&["openai"]);

    // Simulate what create_session_full does when SessionOverrides { model: Some("gpt") }
    // is applied to an agent that has models: [sonnet]. The override replaces the
    // agent's models list with [gpt].
    let overrides = SessionOverrides {
        model: Some("gpt".to_string()),
        ..Default::default()
    };

    // Compute effective_models the same way create_session_full does:
    // if override.model is set, wrap as vec; otherwise use agent's models.
    let agent_models: Option<Vec<String>> = Some(vec!["claude-sonnet-4-20250514".to_string()]);
    let effective_models: Option<Vec<String>> = if let Some(ref m) = overrides.model {
        Some(vec![m.clone()])
    } else {
        agent_models
    };

    let decision = route_session_explained(effective_models.as_deref(), None, None, &client)?;

    // Should route to openai with a concrete model ID (alias "gpt" is resolved)
    match &decision.route {
        SessionRoute::Api { provider, model } => {
            assert_eq!(provider, "openai");
            // The "gpt" alias should be resolved to a concrete model ID
            assert_ne!(
                model, "gpt",
                "alias should be resolved to a concrete model ID"
            );
            assert!(
                model.starts_with("gpt-"),
                "resolved model should start with 'gpt-', got: {model}"
            );
        }
        other => panic!("expected Api route, got: {other:?}"),
    }
    Ok(())
}

// ── SessionOverrides has model_size field ─────────────────────────────────

#[test]
fn session_overrides_has_model_size_field() -> AgentResult<()> {
    let overrides = SessionOverrides {
        model_size: Some("small".to_string()),
        ..Default::default()
    };

    assert_eq!(overrides.model_size.as_deref(), Some("small"));

    // Default should have model_size as None
    let default_overrides = SessionOverrides::default();
    assert_eq!(default_overrides.model_size, None);

    Ok(())
}

#[test]
fn session_overrides_model_size_used_for_routing() -> AgentResult<()> {
    let client = client_with_providers(&["anthropic"]);

    // Simulate create_session_full with model_size override
    let overrides = SessionOverrides {
        model_size: Some("small".to_string()),
        ..Default::default()
    };

    // No model override, no agent models → model_size should be used
    // The effective_model_size comes from the override
    let effective_model_size = overrides.model_size.as_deref();

    let decision = route_session_explained(None, None, effective_model_size, &client)?;

    // Should use ModelSize selection mechanism
    assert_eq!(
        decision.selection_mechanism,
        SelectionMechanism::ModelSize {
            size: "small".to_string(),
        }
    );
    Ok(())
}

// ── Providers path: providers [anthropic] with anthropic creds ───────────

#[test]
fn providers_path_first_provider_has_creds() -> AgentResult<()> {
    let client = client_with_providers(&["anthropic"]);
    let providers = vec!["anthropic".to_string()];

    let decision = route_session_explained(None, Some(&providers), None, &client)?;

    match &decision.route {
        SessionRoute::Api { provider, model } => {
            assert_eq!(provider, "anthropic");
            assert!(
                model.starts_with("claude-"),
                "resolved model should start with 'claude-', got: {model}"
            );
        }
        other => panic!("expected Api route, got: {other:?}"),
    }
    assert_eq!(
        decision.selection_mechanism,
        SelectionMechanism::ProviderPreference,
    );
    Ok(())
}
