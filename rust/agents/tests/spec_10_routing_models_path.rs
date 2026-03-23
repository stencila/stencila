//! Tests for the `models` selection path in `routing.rs` (Phase 3 / Slice 1).
//!
//! These tests verify:
//! - New types: `SelectionMechanism`, `SkipReason`, `SkippedModel`
//! - Updated `RoutingDecision` with `selection_mechanism` and `skipped` fields
//! - New function signatures: `route_session(models, providers, model_size, client)`
//!   and `route_session_explained(models, providers, model_size, client)`
//! - The explicit model list selection path (FR4 highest priority)
//!   with credential checking, skip tracking, and CLI fallback

#![allow(clippy::result_large_err)]

use futures::future::BoxFuture;
use futures::stream::BoxStream;

use stencila_agents::error::AgentResult;
use stencila_agents::routing::{
    SelectionMechanism, SessionRoute, SkipReason, SkippedModel, route_session,
    route_session_explained,
};
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

#[test]
fn models_path_any_falls_through_to_model_size() -> AgentResult<()> {
    let client = client_with_providers(&["anthropic"]);
    let models = vec!["mistral-large-latest".to_string(), "any".to_string()];

    let decision = route_session_explained(Some(&models), None, Some("small"), &client)?;

    assert_eq!(
        decision.selection_mechanism,
        SelectionMechanism::ModelSize {
            size: "small".to_string(),
        }
    );
    Ok(())
}

#[test]
fn models_path_any_falls_through_to_providers() -> AgentResult<()> {
    let client = client_with_providers(&["openai"]);
    let models = vec!["mistral-large-latest".to_string(), "any".to_string()];
    let providers = vec!["openai".to_string()];

    let decision = route_session_explained(Some(&models), Some(&providers), None, &client)?;

    assert_eq!(
        decision.selection_mechanism,
        SelectionMechanism::ProviderPreference
    );
    match &decision.route {
        SessionRoute::Api { provider, model } => {
            assert_eq!(provider, "openai");
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

// ── New type existence tests ─────────────────────────────────────────────

#[test]
fn selection_mechanism_enum_has_explicit_list_variant() -> AgentResult<()> {
    // Verify the ExplicitList variant exists with index and id fields
    let mechanism = SelectionMechanism::ExplicitList {
        index: 0,
        id: "claude-sonnet-4-20250514".to_string(),
    };
    // Should be Debug + Clone + PartialEq
    let cloned = mechanism.clone();
    assert_eq!(mechanism, cloned);
    Ok(())
}

#[test]
fn selection_mechanism_enum_has_model_size_variant() -> AgentResult<()> {
    let mechanism = SelectionMechanism::ModelSize {
        size: "small".to_string(),
    };
    let cloned = mechanism.clone();
    assert_eq!(mechanism, cloned);
    Ok(())
}

#[test]
fn selection_mechanism_enum_has_provider_preference_variant() -> AgentResult<()> {
    let mechanism = SelectionMechanism::ProviderPreference;
    let cloned = mechanism.clone();
    assert_eq!(mechanism, cloned);
    Ok(())
}

#[test]
fn selection_mechanism_enum_has_default_variant() -> AgentResult<()> {
    let mechanism = SelectionMechanism::Default;
    let cloned = mechanism.clone();
    assert_eq!(mechanism, cloned);
    Ok(())
}

#[test]
fn skip_reason_no_credentials_variant() -> AgentResult<()> {
    let reason = SkipReason::NoCredentials {
        provider: "anthropic".to_string(),
    };
    let cloned = reason.clone();
    assert_eq!(reason, cloned);
    Ok(())
}

#[test]
fn skip_reason_not_in_catalog_variant() -> AgentResult<()> {
    let reason = SkipReason::NotInCatalog {
        model: "nonexistent-model".to_string(),
    };
    let cloned = reason.clone();
    assert_eq!(reason, cloned);
    Ok(())
}

#[test]
fn skip_reason_no_provider_inferred_variant() -> AgentResult<()> {
    let reason = SkipReason::NoProviderInferred {
        model: "some-unknown-model".to_string(),
    };
    let cloned = reason.clone();
    assert_eq!(reason, cloned);
    Ok(())
}

#[test]
fn skipped_model_struct_has_model_and_reason() -> AgentResult<()> {
    let skipped = SkippedModel {
        model: "claude-sonnet-4-20250514".to_string(),
        reason: SkipReason::NoCredentials {
            provider: "anthropic".to_string(),
        },
    };
    assert_eq!(skipped.model, "claude-sonnet-4-20250514");
    assert_eq!(
        skipped.reason,
        SkipReason::NoCredentials {
            provider: "anthropic".to_string()
        }
    );
    Ok(())
}

// ── New signature tests ──────────────────────────────────────────────────

#[test]
fn route_session_accepts_new_signature() -> AgentResult<()> {
    let client = client_with_providers(&["anthropic"]);
    let models = vec!["claude-sonnet-4-20250514".to_string()];

    // The new signature: (models, providers, model_size, client)
    let route = route_session(Some(&models), None, None, &client)?;
    assert!(matches!(
        route,
        SessionRoute::Api { .. } | SessionRoute::Cli { .. }
    ));
    Ok(())
}

#[test]
fn route_session_explained_accepts_new_signature() -> AgentResult<()> {
    let client = client_with_providers(&["anthropic"]);
    let models = vec!["claude-sonnet-4-20250514".to_string()];

    // The new signature: (models, providers, model_size, client)
    let decision = route_session_explained(Some(&models), None, None, &client)?;
    assert!(matches!(
        decision.route,
        SessionRoute::Api { .. } | SessionRoute::Cli { .. }
    ));
    Ok(())
}

// ── RoutingDecision field tests ──────────────────────────────────────────

#[test]
fn routing_decision_has_selection_mechanism_field() -> AgentResult<()> {
    let client = client_with_providers(&["anthropic"]);
    let models = vec!["claude-sonnet-4-20250514".to_string()];

    let decision = route_session_explained(Some(&models), None, None, &client)?;
    // The field should exist and be accessible
    let _mechanism: &SelectionMechanism = &decision.selection_mechanism;
    Ok(())
}

#[test]
fn routing_decision_has_skipped_field() -> AgentResult<()> {
    let client = client_with_providers(&["anthropic"]);
    let models = vec!["claude-sonnet-4-20250514".to_string()];

    let decision = route_session_explained(Some(&models), None, None, &client)?;
    // The skipped field should exist and be a Vec<SkippedModel>
    let _skipped: &Vec<SkippedModel> = &decision.skipped;
    Ok(())
}

// ── Test (a): models [sonnet, gpt] + anthropic creds → selects sonnet ───

#[test]
fn models_path_selects_first_available_model() -> AgentResult<()> {
    let client = client_with_providers(&["anthropic"]);
    let models = vec!["claude-sonnet-4-20250514".to_string(), "gpt-4o".to_string()];

    let decision = route_session_explained(Some(&models), None, None, &client)?;

    // Should select sonnet (first model) since anthropic credentials are available
    assert_eq!(
        decision.route,
        SessionRoute::Api {
            provider: "anthropic".into(),
            model: "claude-sonnet-4-20250514".into(),
        }
    );
    assert_eq!(
        decision.selection_mechanism,
        SelectionMechanism::ExplicitList {
            index: 0,
            id: "claude-sonnet-4-20250514".to_string(),
        }
    );
    // No models should be skipped
    assert!(decision.skipped.is_empty());
    Ok(())
}

// ── Test (b): models [sonnet, gpt] + openai creds → skips sonnet ────────

#[test]
fn models_path_skips_unavailable_selects_second() -> AgentResult<()> {
    let client = client_with_providers(&["openai"]);
    let models = vec!["claude-sonnet-4-20250514".to_string(), "gpt-4o".to_string()];

    let decision = route_session_explained(Some(&models), None, None, &client)?;

    // Should skip sonnet (no anthropic creds), select gpt-4o at index 1
    assert_eq!(
        decision.route,
        SessionRoute::Api {
            provider: "openai".into(),
            model: "gpt-4o".into(),
        }
    );
    assert_eq!(
        decision.selection_mechanism,
        SelectionMechanism::ExplicitList {
            index: 1,
            id: "gpt-4o".to_string(),
        }
    );
    // Skipped should contain sonnet with NoCredentials reason
    assert_eq!(decision.skipped.len(), 1);
    assert_eq!(decision.skipped[0].model, "claude-sonnet-4-20250514");
    assert!(matches!(
        &decision.skipped[0].reason,
        SkipReason::NoCredentials { provider } if provider == "anthropic"
    ));
    Ok(())
}

// ── Test (c): models [nonexistent, sonnet] + anthropic creds → skips nonexistent

#[test]
fn models_path_skips_not_in_catalog_model() -> AgentResult<()> {
    let client = client_with_providers(&["anthropic"]);
    let models = vec![
        "nonexistent-model-xyz".to_string(),
        "claude-sonnet-4-20250514".to_string(),
    ];

    let decision = route_session_explained(Some(&models), None, None, &client)?;

    // Should skip nonexistent (not in catalog / no provider inferred), select sonnet
    assert_eq!(
        decision.route,
        SessionRoute::Api {
            provider: "anthropic".into(),
            model: "claude-sonnet-4-20250514".into(),
        }
    );
    assert_eq!(
        decision.selection_mechanism,
        SelectionMechanism::ExplicitList {
            index: 1,
            id: "claude-sonnet-4-20250514".to_string(),
        }
    );
    // Skipped should contain the nonexistent model
    assert_eq!(decision.skipped.len(), 1);
    assert_eq!(decision.skipped[0].model, "nonexistent-model-xyz");
    assert!(matches!(
        &decision.skipped[0].reason,
        SkipReason::NotInCatalog { .. } | SkipReason::NoProviderInferred { .. }
    ));
    Ok(())
}

#[test]
fn models_path_accepts_uncatalogued_but_inferable_model() -> AgentResult<()> {
    let client = client_with_providers(&["openai"]);
    let models = vec!["gpt-unreleased-preview".to_string()];

    let decision = route_session_explained(Some(&models), None, None, &client)?;

    assert_eq!(
        decision.route,
        SessionRoute::Api {
            provider: "openai".into(),
            model: "gpt-unreleased-preview".into(),
        }
    );
    assert!(decision.skipped.is_empty());
    Ok(())
}

// ── Test (d): models [sonnet] + model_size small → models wins precedence

#[test]
fn models_path_takes_precedence_over_model_size() -> AgentResult<()> {
    let client = client_with_providers(&["anthropic"]);
    let models = vec!["claude-sonnet-4-20250514".to_string()];

    // Even with model_size specified, models takes highest precedence
    let decision = route_session_explained(Some(&models), None, Some("small"), &client)?;

    assert_eq!(
        decision.route,
        SessionRoute::Api {
            provider: "anthropic".into(),
            model: "claude-sonnet-4-20250514".into(),
        }
    );
    assert_eq!(
        decision.selection_mechanism,
        SelectionMechanism::ExplicitList {
            index: 0,
            id: "claude-sonnet-4-20250514".to_string(),
        }
    );
    Ok(())
}

// ── Test (e): models [sonnet] + providers [openai] → providers ignored ───

#[test]
fn models_path_takes_precedence_over_providers() -> AgentResult<()> {
    let client = client_with_providers(&["anthropic", "openai"]);
    let models = vec!["claude-sonnet-4-20250514".to_string(), "gpt-4o".to_string()];
    let providers = vec!["openai".to_string(), "anthropic".to_string()];

    // Even with providers specified, models takes highest precedence
    let decision = route_session_explained(Some(&models), Some(&providers), None, &client)?;

    // sonnet should route to anthropic, not openai
    assert_eq!(
        decision.route,
        SessionRoute::Api {
            provider: "anthropic".into(),
            model: "claude-sonnet-4-20250514".into(),
        }
    );
    assert_eq!(
        decision.selection_mechanism,
        SelectionMechanism::ExplicitList {
            index: 0,
            id: "claude-sonnet-4-20250514".to_string(),
        }
    );
    Ok(())
}

// ── Test (f): models [sonnet, gpt] + no API keys + claude-cli available → CLI fallback

#[test]
#[allow(clippy::print_stderr)]
fn models_path_cli_fallback_when_no_api_keys() -> AgentResult<()> {
    use stencila_agents::cli_providers::is_cli_available;

    // Skip if claude CLI is not installed
    if !is_cli_available("claude") {
        eprintln!("Skipping test: claude CLI not available on PATH");
        return Ok(());
    }

    let client = empty_client();
    let models = vec!["claude-sonnet-4-20250514".to_string(), "gpt-4o".to_string()];

    let decision = route_session_explained(Some(&models), None, None, &client)?;

    // Should fall back to CLI for the first model's inferred provider (anthropic → claude-cli)
    assert!(matches!(
        &decision.route,
        SessionRoute::Cli { provider, .. } if provider == "claude-cli"
    ));
    Ok(())
}

// ── Test (g): models [mistral-model] + no credentials + no CLI → error ───

#[test]
fn models_path_all_unavailable_returns_error() -> AgentResult<()> {
    let client = empty_client();
    // mistral has no CLI fallback
    let models = vec!["mistral-large-latest".to_string()];

    let result = route_session_explained(Some(&models), None, None, &client);

    assert!(
        result.is_err(),
        "should return error when no models are available"
    );
    let err_msg = result.expect_err("should be error").to_string();
    // Error should mention something about the model or provider being unavailable
    assert!(
        err_msg.contains("mistral") || err_msg.contains("no") || err_msg.contains("unavailable"),
        "error should be descriptive: {err_msg}"
    );
    assert!(
        err_msg.contains("add 'any'"),
        "error should suggest using 'any': {err_msg}"
    );
    Ok(())
}

// ── summary() and fallback_warning() include new info ────────────────────

#[test]
fn summary_includes_selection_mechanism_info() -> AgentResult<()> {
    let client = client_with_providers(&["anthropic"]);
    let models = vec!["claude-sonnet-4-20250514".to_string()];

    let decision = route_session_explained(Some(&models), None, None, &client)?;
    let summary = decision.summary();

    // Summary should mention the provider and model
    assert!(summary.contains("anthropic") || summary.contains("claude"));
    Ok(())
}

#[test]
#[allow(clippy::print_stderr)]
fn fallback_warning_includes_skipped_models_info() -> AgentResult<()> {
    use stencila_agents::cli_providers::is_cli_available;

    // Skip if claude CLI is not installed
    if !is_cli_available("claude") {
        eprintln!("Skipping test: claude CLI not available on PATH");
        return Ok(());
    }

    let client = empty_client();
    let models = vec!["claude-sonnet-4-20250514".to_string(), "gpt-4o".to_string()];

    let decision = route_session_explained(Some(&models), None, None, &client)?;

    // When falling back to CLI, the warning should mention skipped models
    if let Some(warning) = decision.fallback_warning() {
        assert!(
            warning.contains("skipped")
                || warning.contains("unavailable")
                || warning.contains("claude")
                || warning.contains("gpt"),
            "warning should reference skipped models: {warning}"
        );
    }
    Ok(())
}
