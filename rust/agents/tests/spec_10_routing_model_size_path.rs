//! Tests for the `model_size` selection path in `routing.rs` (Phase 3 / Slice 2).
//!
//! These tests verify:
//! - `parse_model_size` helper: case-insensitive conversion of "large"/"medium"/"small"
//! - The `model_size` selection path in `route_session_explained`:
//!   when `models` is None and `model_size` is Some, uses catalog to find
//!   size-matched candidates, filters by providers, checks credentials,
//!   selects first available.
//! - Graceful fallthrough for unrecognized `model_size` values.

#![allow(clippy::result_large_err)]

use futures::future::BoxFuture;
use futures::stream::BoxStream;

use stencila_agents::error::AgentResult;
use stencila_agents::routing::{
    ModelSource, ProviderSource, SelectionMechanism, parse_model_size, route_session_explained,
};
use stencila_models3::catalog::ModelSize;
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

// ── parse_model_size tests ───────────────────────────────────────────────

#[test]
fn parse_model_size_large() -> AgentResult<()> {
    assert_eq!(parse_model_size("large"), Some(ModelSize::Large));
    Ok(())
}

#[test]
fn parse_model_size_medium() -> AgentResult<()> {
    assert_eq!(parse_model_size("medium"), Some(ModelSize::Medium));
    Ok(())
}

#[test]
fn parse_model_size_small() -> AgentResult<()> {
    assert_eq!(parse_model_size("small"), Some(ModelSize::Small));
    Ok(())
}

#[test]
fn parse_model_size_case_insensitive() -> AgentResult<()> {
    assert_eq!(parse_model_size("Large"), Some(ModelSize::Large));
    assert_eq!(parse_model_size("MEDIUM"), Some(ModelSize::Medium));
    assert_eq!(parse_model_size("Small"), Some(ModelSize::Small));
    assert_eq!(parse_model_size("LARGE"), Some(ModelSize::Large));
    Ok(())
}

#[test]
fn parse_model_size_unrecognized_returns_none() -> AgentResult<()> {
    assert_eq!(parse_model_size("invalid_value"), None);
    assert_eq!(parse_model_size("huge"), None);
    assert_eq!(parse_model_size(""), None);
    assert_eq!(parse_model_size("tiny"), None);
    Ok(())
}

// ── Test (a): model_size "small" + providers [anthropic] + anthropic creds

#[test]
fn model_size_small_with_anthropic_provider_selects_haiku() -> AgentResult<()> {
    let client = client_with_providers(&["anthropic"]);
    let providers = vec!["anthropic".to_string()];

    // models is None, model_size is "small", providers filters to anthropic
    let decision = route_session_explained(None, Some(&providers), Some("small"), &client)?;

    // Should select an anthropic small model (haiku variant)
    match &decision.route {
        stencila_agents::routing::SessionRoute::Api { provider, model } => {
            assert_eq!(provider, "anthropic", "provider should be anthropic");
            assert!(
                model.contains("haiku"),
                "small anthropic model should be a haiku variant, got: {model}"
            );
        }
        stencila_agents::routing::SessionRoute::Cli { .. } => {
            panic!("expected API route for model_size selection with credentials");
        }
    }

    // selection_mechanism should be ModelSize with size "small"
    assert_eq!(
        decision.selection_mechanism,
        SelectionMechanism::ModelSize {
            size: "small".to_string(),
        }
    );

    // provider and model sources should reflect catalog-driven selection
    assert_eq!(
        decision.provider_source,
        ProviderSource::CatalogModelSize,
        "model_size path should report CatalogModelSize provider source"
    );
    assert_eq!(
        decision.model_source,
        ModelSource::CatalogModelSize,
        "model_size path should report CatalogModelSize model source"
    );
    Ok(())
}

// ── Test (b): model_size "medium" + no providers + anthropic creds

#[test]
fn model_size_medium_no_provider_filter_selects_medium_model() -> AgentResult<()> {
    let client = client_with_providers(&["anthropic"]);

    // models is None, model_size is "medium", no provider filter
    let decision = route_session_explained(None, None, Some("medium"), &client)?;

    // Should select a medium model; with anthropic creds, likely a sonnet variant
    match &decision.route {
        stencila_agents::routing::SessionRoute::Api { provider, model } => {
            assert_eq!(
                provider, "anthropic",
                "should select from anthropic (only provider with creds)"
            );
            // Medium anthropic models are sonnet variants
            assert!(
                model.contains("sonnet"),
                "medium anthropic model should be a sonnet variant, got: {model}"
            );
        }
        stencila_agents::routing::SessionRoute::Cli { .. } => {
            panic!("expected API route for model_size selection with credentials");
        }
    }

    // selection_mechanism should be ModelSize with size "medium"
    assert_eq!(
        decision.selection_mechanism,
        SelectionMechanism::ModelSize {
            size: "medium".to_string(),
        }
    );

    // provider and model sources should reflect catalog-driven selection
    assert_eq!(
        decision.provider_source,
        ProviderSource::CatalogModelSize,
        "model_size path should report CatalogModelSize provider source"
    );
    assert_eq!(
        decision.model_source,
        ModelSource::CatalogModelSize,
        "model_size path should report CatalogModelSize model source"
    );
    Ok(())
}

// ── Test (c): model_size "invalid_value" → fallthrough to default ────────

#[test]
fn model_size_invalid_falls_through_to_default() -> AgentResult<()> {
    let client = client_with_providers(&["anthropic"]);

    // models is None, model_size is unrecognized → should fall through
    let decision = route_session_explained(None, None, Some("invalid_value"), &client)?;

    // The selection_mechanism should NOT be ModelSize — it should be Default
    // or ProviderPreference since the invalid size caused a fallthrough
    assert!(
        !matches!(
            decision.selection_mechanism,
            SelectionMechanism::ModelSize { .. }
        ),
        "invalid model_size should not produce ModelSize mechanism, got: {:?}",
        decision.selection_mechanism
    );
    assert!(
        matches!(
            decision.selection_mechanism,
            SelectionMechanism::Default | SelectionMechanism::ProviderPreference
        ),
        "should fall through to Default or ProviderPreference, got: {:?}",
        decision.selection_mechanism
    );
    Ok(())
}

#[test]
fn model_size_unsatisfied_falls_through_to_provider_preference() -> AgentResult<()> {
    let client = stencila_models3::client::Client::builder()
        .build()
        .expect("empty builder should succeed");
    let providers = vec!["anthropic".to_string()];

    // No anthropic credentials, but the provider preference path can still
    // produce the usual CLI fallback after model_size fails to find a match.
    let decision = route_session_explained(None, Some(&providers), Some("small"), &client)?;

    assert!(matches!(
        decision.route,
        stencila_agents::routing::SessionRoute::Cli { .. }
    ));
    assert_eq!(
        decision.selection_mechanism,
        SelectionMechanism::ProviderPreference
    );

    Ok(())
}
