//! Integration tests for the full FR4 routing precedence chain
//! (Phase 4 — integration verification).
//!
//! These tests verify the complete routing precedence ordering works
//! end-to-end: models > modelSize > providers > defaults. Each test
//! exercises one priority path and confirms it takes precedence over
//! lower-priority parameters.
//!
//! Also verifies that the schema-generated `Agent` type includes the
//! `models`, `providers`, and `model_size` fields.

#![allow(clippy::result_large_err)]

use futures::future::BoxFuture;
use futures::stream::BoxStream;

use stencila_agents::convenience::SessionOverrides;
use stencila_agents::error::AgentResult;
use stencila_agents::routing::{
    SelectionMechanism, SessionRoute, route_session, route_session_explained,
};
use stencila_models3::error::SdkError;
use stencila_models3::provider::ProviderAdapter;
use stencila_models3::types::request::Request;
use stencila_models3::types::response::Response;
use stencila_models3::types::stream_event::StreamEvent;
use stencila_schema::Agent;

// ── Helpers ──────────────────────────────────────────────────────────────

type SdkResult<T> = Result<T, SdkError>;

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

fn client_with_providers(names: &[&str]) -> stencila_models3::client::Client {
    let mut builder = stencila_models3::client::Client::builder();
    for &name in names {
        builder = builder.add_provider_as(name, StubAdapter::new(name));
    }
    builder.build().expect("builder should succeed")
}

fn empty_client() -> stencila_models3::client::Client {
    stencila_models3::client::Client::builder()
        .build()
        .expect("empty builder should succeed")
}

// ── Schema-generated type field verification ─────────────────────────────

#[test]
fn agent_type_has_models_field() -> AgentResult<()> {
    let agent = Agent {
        models: Some(vec!["claude-sonnet-4-20250514".to_string()]),
        ..Default::default()
    };
    assert_eq!(
        agent.models,
        Some(vec!["claude-sonnet-4-20250514".to_string()])
    );
    Ok(())
}

#[test]
fn agent_type_has_providers_field() -> AgentResult<()> {
    let agent = Agent {
        providers: Some(vec!["anthropic".to_string(), "openai".to_string()]),
        ..Default::default()
    };
    assert_eq!(
        agent.providers,
        Some(vec!["anthropic".to_string(), "openai".to_string()])
    );
    Ok(())
}

#[test]
fn agent_type_has_model_size_field() -> AgentResult<()> {
    let agent = Agent {
        model_size: Some("medium".to_string()),
        ..Default::default()
    };
    assert_eq!(agent.model_size, Some("medium".to_string()));
    Ok(())
}

// ── Full precedence chain: models > modelSize > providers > defaults ─────

#[test]
fn precedence_models_beats_model_size_and_providers() -> AgentResult<()> {
    let client = client_with_providers(&["anthropic", "openai"]);
    let models = vec!["gpt-4o".to_string(), "claude-sonnet-4-20250514".to_string()];
    let providers = vec!["anthropic".to_string(), "openai".to_string()];

    // All three are specified: the explicit models list should win over lower-priority
    // model_size and providers preferences.
    let decision =
        route_session_explained(Some(&models), Some(&providers), Some("small"), &client)?;

    assert_eq!(
        decision.selection_mechanism,
        SelectionMechanism::ExplicitList {
            index: 0,
            id: "gpt-4o".to_string(),
        }
    );
    assert_eq!(
        decision.route,
        SessionRoute::Api {
            provider: "openai".into(),
            model: "gpt-4o".into(),
        }
    );
    Ok(())
}

#[test]
fn precedence_model_size_beats_providers_when_no_models() -> AgentResult<()> {
    let client = client_with_providers(&["anthropic", "openai"]);
    let providers = vec!["openai".to_string()];

    // No models, but model_size and providers both specified — model_size should win
    let decision = route_session_explained(None, Some(&providers), Some("small"), &client)?;

    assert_eq!(
        decision.selection_mechanism,
        SelectionMechanism::ModelSize {
            size: "small".to_string(),
        }
    );
    Ok(())
}

#[test]
fn precedence_providers_beats_default_when_no_models_or_size() -> AgentResult<()> {
    let client = client_with_providers(&["anthropic", "openai"]);
    let providers = vec!["openai".to_string()];

    // No models, no model_size — providers should take effect
    let decision = route_session_explained(None, Some(&providers), None, &client)?;

    assert_eq!(
        decision.selection_mechanism,
        SelectionMechanism::ProviderPreference,
    );
    assert_eq!(
        decision.route,
        SessionRoute::Api {
            provider: "openai".into(),
            model: "gpt".into(),
        }
    );
    Ok(())
}

#[test]
fn precedence_default_when_nothing_specified() -> AgentResult<()> {
    let client = client_with_providers(&["anthropic"]);

    // Nothing specified — default path
    let decision = route_session_explained(None, None, None, &client)?;

    assert_eq!(decision.selection_mechanism, SelectionMechanism::Default);
    Ok(())
}

// ── route_session returns just the route (thin wrapper) ──────────────────

#[test]
fn route_session_thin_wrapper_matches_explained() -> AgentResult<()> {
    let client = client_with_providers(&["openai"]);
    let models = vec!["gpt-4o".to_string()];

    let route = route_session(Some(&models), None, None, &client)?;
    let decision = route_session_explained(Some(&models), None, None, &client)?;

    assert_eq!(route, decision.route);
    Ok(())
}

// ── SessionOverrides integration ─────────────────────────────────────────

#[test]
fn session_overrides_model_replaces_agent_models_for_routing() -> AgentResult<()> {
    let client = client_with_providers(&["openai"]);

    // Simulate: agent has models=[sonnet], override has model=gpt-4o
    let overrides = SessionOverrides {
        model: Some("gpt-4o".to_string()),
        ..Default::default()
    };

    // Compute effective parameters the same way create_session_full does
    let agent_models = Some(vec!["claude-sonnet-4-20250514".to_string()]);
    let effective_models = if let Some(ref m) = overrides.model {
        Some(vec![m.clone()])
    } else {
        agent_models
    };

    let decision = route_session_explained(effective_models.as_deref(), None, None, &client)?;

    // Override should win: gpt-4o via openai, not sonnet via anthropic
    assert_eq!(
        decision.route,
        SessionRoute::Api {
            provider: "openai".into(),
            model: "gpt-4o".into(),
        }
    );
    Ok(())
}

#[test]
fn single_model_and_provider_preserve_legacy_pairing() -> AgentResult<()> {
    let client = client_with_providers(&["anthropic", "openai"]);
    let models = vec!["gpt-4o".to_string()];
    let providers = vec!["anthropic".to_string()];

    let decision = route_session_explained(Some(&models), Some(&providers), None, &client)?;

    assert_eq!(
        decision.route,
        SessionRoute::Api {
            provider: "anthropic".into(),
            model: "gpt-4o".into(),
        }
    );
    assert_eq!(decision.selection_mechanism, SelectionMechanism::Default);
    Ok(())
}

#[test]
fn session_overrides_model_size_used_when_no_model_override() -> AgentResult<()> {
    let client = client_with_providers(&["anthropic"]);

    let overrides = SessionOverrides {
        model_size: Some("small".to_string()),
        ..Default::default()
    };

    // No agent models, no model override — model_size should be used
    let effective_model_size = overrides.model_size.as_deref();

    let decision = route_session_explained(None, None, effective_model_size, &client)?;

    assert_eq!(
        decision.selection_mechanism,
        SelectionMechanism::ModelSize {
            size: "small".to_string(),
        }
    );
    Ok(())
}

// ── Fallback behavior end-to-end ─────────────────────────────────────────

#[test]
fn models_fallback_across_providers() -> AgentResult<()> {
    // Only openai has creds; models list has sonnet first, then gpt
    let client = client_with_providers(&["openai"]);
    let models = vec!["claude-sonnet-4-20250514".to_string(), "gpt-4o".to_string()];

    let decision = route_session_explained(Some(&models), None, None, &client)?;

    // Should skip sonnet (no anthropic creds), select gpt-4o
    assert_eq!(
        decision.route,
        SessionRoute::Api {
            provider: "openai".into(),
            model: "gpt-4o".into(),
        }
    );
    assert_eq!(decision.skipped.len(), 1);
    assert_eq!(decision.skipped[0].model, "claude-sonnet-4-20250514");
    Ok(())
}

#[test]
fn all_models_unavailable_no_cli_returns_error() -> AgentResult<()> {
    let client = empty_client();
    // mistral has no CLI fallback
    let models = vec!["mistral-large-latest".to_string()];

    let result = route_session(Some(&models), None, None, &client);
    assert!(result.is_err());
    Ok(())
}
