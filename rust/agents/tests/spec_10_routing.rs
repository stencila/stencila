//! Tests for session routing logic (`routing` module).
//!
//! Covers the `route_session` decision helper and the API↔CLI mapping
//! functions independently of real environments.

#![allow(clippy::result_large_err)]

use futures::future::BoxFuture;
use futures::stream::BoxStream;

use stencila_agents::error::AgentResult;
use stencila_agents::routing::{
    SessionRoute, api_to_cli, cli_to_api, is_cli_provider, route_session,
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

/// Build a client with the given provider names registered.
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

// ── Mapping helper tests ─────────────────────────────────────────────────

#[test]
fn cli_provider_mapping_roundtrip() -> AgentResult<()> {
    // API → CLI
    assert_eq!(api_to_cli("anthropic"), Some("claude-cli"));
    assert_eq!(api_to_cli("openai"), Some("codex-cli"));
    assert_eq!(api_to_cli("gemini"), Some("gemini-cli"));
    assert_eq!(api_to_cli("google"), Some("gemini-cli"));
    assert_eq!(api_to_cli("mistral"), None);
    assert_eq!(api_to_cli("deepseek"), None);

    // CLI → API
    assert_eq!(cli_to_api("claude-cli"), Some("anthropic"));
    assert_eq!(cli_to_api("codex-cli"), Some("openai"));
    assert_eq!(cli_to_api("gemini-cli"), Some("gemini"));
    assert_eq!(cli_to_api("anthropic"), None);

    Ok(())
}

#[test]
fn is_cli_provider_detects_all_variants() -> AgentResult<()> {
    assert!(is_cli_provider("claude-cli"));
    assert!(is_cli_provider("codex-cli"));
    assert!(is_cli_provider("gemini-cli"));
    assert!(!is_cli_provider("anthropic"));
    assert!(!is_cli_provider("openai"));
    assert!(!is_cli_provider("gemini"));
    assert!(!is_cli_provider(""));
    Ok(())
}

// ── route_session tests ──────────────────────────────────────────────────

#[test]
fn explicit_cli_provider_routes_to_cli() -> AgentResult<()> {
    let client = empty_client();

    let route = route_session(Some("claude-cli"), Some("claude-sonnet-4-5"), &client)?;
    assert_eq!(
        route,
        SessionRoute::Cli {
            provider: "claude-cli".into(),
            model: Some("claude-sonnet-4-5".into()),
        }
    );

    // Without model
    let route = route_session(Some("codex-cli"), None, &client)?;
    assert_eq!(
        route,
        SessionRoute::Cli {
            provider: "codex-cli".into(),
            model: None,
        }
    );

    let route = route_session(Some("gemini-cli"), None, &client)?;
    assert_eq!(
        route,
        SessionRoute::Cli {
            provider: "gemini-cli".into(),
            model: None,
        }
    );

    Ok(())
}

#[test]
fn explicit_api_provider_with_auth_routes_to_api() -> AgentResult<()> {
    let client = client_with_providers(&["anthropic"]);

    let route = route_session(Some("anthropic"), Some("claude-sonnet-4-5"), &client)?;
    assert_eq!(
        route,
        SessionRoute::Api {
            provider: "anthropic".into(),
            model: "claude-sonnet-4-5".into(),
        }
    );

    Ok(())
}

#[test]
fn explicit_api_provider_with_auth_default_model() -> AgentResult<()> {
    let client = client_with_providers(&["openai"]);

    let route = route_session(Some("openai"), None, &client)?;
    assert_eq!(
        route,
        SessionRoute::Api {
            provider: "openai".into(),
            model: "gpt".into(),
        }
    );

    Ok(())
}

#[test]
fn explicit_api_provider_without_auth_falls_back_to_cli() -> AgentResult<()> {
    // Client has no "anthropic" provider registered
    let client = empty_client();

    let route = route_session(Some("anthropic"), Some("claude-sonnet-4-5"), &client)?;
    assert_eq!(
        route,
        SessionRoute::Cli {
            provider: "claude-cli".into(),
            model: Some("claude-sonnet-4-5".into()),
        }
    );

    // openai → codex-cli
    let route = route_session(Some("openai"), None, &client)?;
    assert_eq!(
        route,
        SessionRoute::Cli {
            provider: "codex-cli".into(),
            model: Some("gpt".into()),
        }
    );

    // gemini → gemini-cli
    let route = route_session(Some("gemini"), None, &client)?;
    assert_eq!(
        route,
        SessionRoute::Cli {
            provider: "gemini-cli".into(),
            model: Some("gemini".into()),
        }
    );

    Ok(())
}

#[test]
fn no_provider_no_model_no_keys_falls_back_to_first_cli() -> AgentResult<()> {
    let client = empty_client();

    let route = route_session(None, None, &client)?;
    assert_eq!(
        route,
        SessionRoute::Cli {
            provider: "claude-cli".into(),
            model: None,
        }
    );

    Ok(())
}

#[test]
fn no_provider_no_model_with_api_key_routes_to_api() -> AgentResult<()> {
    let client = client_with_providers(&["anthropic"]);

    let route = route_session(None, None, &client)?;
    assert_eq!(
        route,
        SessionRoute::Api {
            provider: "anthropic".into(),
            model: "claude".into(),
        }
    );

    Ok(())
}

#[test]
fn unmapped_api_provider_without_auth_returns_error() -> AgentResult<()> {
    let client = empty_client();

    let result = route_session(Some("mistral"), Some("mistral-large"), &client);
    assert!(result.is_err());

    let err = result.expect_err("should be an error");
    let msg = err.to_string();
    assert!(
        msg.contains("mistral"),
        "error should mention the provider: {msg}"
    );

    Ok(())
}

#[test]
fn explicit_api_provider_unknown_default_model_returns_error() -> AgentResult<()> {
    let client = empty_client();

    // "deepseek" has no default_model mapping and no CLI mapping
    let result = route_session(Some("deepseek"), None, &client);
    assert!(result.is_err());

    let err = result.expect_err("should be an error");
    let msg = err.to_string();
    assert!(
        msg.contains("No default model"),
        "error should mention missing default model: {msg}"
    );

    Ok(())
}

#[test]
fn cli_provider_ignores_api_auth() -> AgentResult<()> {
    // Even if the API provider is registered, explicit CLI routes to CLI
    let client = client_with_providers(&["anthropic"]);

    let route = route_session(Some("claude-cli"), Some("claude-sonnet-4-5"), &client)?;
    assert_eq!(
        route,
        SessionRoute::Cli {
            provider: "claude-cli".into(),
            model: Some("claude-sonnet-4-5".into()),
        }
    );

    Ok(())
}
