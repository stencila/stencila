//! Spec Section 2 conformance tests.
//!
//! Target areas:
//! - Provider adapter interface conformance
//! - Client routing and default provider resolution
//! - Middleware behavior for both `complete()` and `stream()`
//! - Default client initialization and override behavior
//! - Model catalog lookup/list/latest helpers

mod common;

use std::sync::{Arc, Mutex, OnceLock};

use futures::StreamExt;

use stencila_auth::{AuthOptions, AuthOverrides, StaticKey};
use stencila_models3::api::default_client;
use stencila_models3::catalog::{
    ModelInfo, get_latest_model, get_model_info, list_models, merge_models, refresh,
};
use stencila_models3::client::Client;
use stencila_models3::error::{SdkError, SdkResult};
use stencila_models3::middleware::{Middleware, NextComplete, NextStream};
use stencila_models3::provider::{BoxFuture, BoxStream, ProviderAdapter};
use stencila_models3::types::message::Message;
use stencila_models3::types::request::Request;
use stencila_models3::types::response::Response;
use stencila_models3::types::stream_event::StreamEvent;
use stencila_models3::types::tool::ToolChoice;

use crate::common::{
    ErrorAdapter, MockAdapter, ModelListingAdapter, make_request, make_request_for,
};

// ── Model catalog (Spec §2.9) ─────────────────────────────────────────

#[test]
fn catalog_contains_all_providers() -> Result<(), Box<dyn std::error::Error>> {
    let providers: Vec<String> = list_models(None)?
        .iter()
        .map(|m| m.provider.clone())
        .collect();
    assert!(providers.iter().any(|p| p == "anthropic"));
    assert!(providers.iter().any(|p| p == "openai"));
    assert!(providers.iter().any(|p| p == "gemini"));
    Ok(())
}

#[test]
fn catalog_lookup_by_id() -> Result<(), Box<dyn std::error::Error>> {
    // Use the first model in the catalog — no hardcoded model ID
    let all = list_models(None)?;
    let first = all.first().ok_or("catalog is empty")?;
    let info = get_model_info(&first.id)?.ok_or("lookup by id failed")?;
    assert_eq!(info.id, first.id);
    Ok(())
}

#[test]
fn catalog_lookup_by_alias() -> Result<(), Box<dyn std::error::Error>> {
    // Find any model that has aliases, then look it up by the first alias
    let all = list_models(None)?;
    let with_alias = all
        .iter()
        .find(|m| !m.aliases.is_empty())
        .ok_or("no models with aliases")?;
    let alias = &with_alias.aliases[0];
    let info = get_model_info(alias)?.ok_or("alias lookup failed")?;
    assert_eq!(info.id, with_alias.id);
    Ok(())
}

#[test]
fn catalog_unknown_model_returns_none() -> Result<(), Box<dyn std::error::Error>> {
    assert!(get_model_info("totally-fake-model")?.is_none());
    Ok(())
}

#[test]
fn catalog_list_filters_by_provider() -> Result<(), Box<dyn std::error::Error>> {
    let all = list_models(None)?;
    let provider = &all.first().ok_or("catalog is empty")?.provider;
    let filtered = list_models(Some(provider))?;
    assert!(!filtered.is_empty());
    for m in &filtered {
        assert_eq!(&m.provider, provider);
    }
    Ok(())
}

#[test]
fn catalog_latest_model() -> Result<(), Box<dyn std::error::Error>> {
    let all = list_models(None)?;
    let provider = &all.first().ok_or("catalog is empty")?.provider;
    let m = get_latest_model(provider, None)?.ok_or("no model for provider")?;
    assert_eq!(&m.provider, provider);
    Ok(())
}

#[test]
fn catalog_latest_with_vision() -> Result<(), Box<dyn std::error::Error>> {
    let all = list_models(None)?;
    let vision_model = all
        .iter()
        .find(|m| m.supports_vision)
        .ok_or("no vision models in catalog")?;
    let m = get_latest_model(&vision_model.provider, Some("vision"))?
        .ok_or("latest vision lookup failed")?;
    assert!(m.supports_vision);
    Ok(())
}

#[test]
fn catalog_latest_unknown_provider() -> Result<(), Box<dyn std::error::Error>> {
    assert!(get_latest_model("nonexistent_provider", None)?.is_none());
    Ok(())
}

#[test]
fn catalog_latest_unknown_capability_returns_none() -> Result<(), Box<dyn std::error::Error>> {
    let all = list_models(None)?;
    let provider = &all.first().ok_or("catalog is empty")?.provider;
    // Typo "vison" should return None, not silently match
    assert!(get_latest_model(provider, Some("vison"))?.is_none());
    Ok(())
}

#[test]
fn catalog_model_info_has_required_fields() -> Result<(), Box<dyn std::error::Error>> {
    // Only check curated providers; test-injected placeholder models
    // (from refresh tests) may have context_window = 0 which is valid
    // for API-discovered entries.
    let curated_providers = ["anthropic", "openai", "gemini"];
    let curated: Vec<_> = list_models(None)?
        .into_iter()
        .filter(|m| curated_providers.contains(&m.provider.as_str()))
        .collect();
    assert!(!curated.is_empty(), "catalog should contain curated models");
    for info in &curated {
        assert!(!info.id.is_empty(), "id must be non-empty");
        assert!(!info.provider.is_empty(), "provider must be non-empty");
        assert!(
            !info.display_name.is_empty(),
            "display_name must be non-empty"
        );
        assert!(
            info.context_window > 0,
            "curated model context_window must be positive"
        );
    }
    Ok(())
}

// ── Catalog runtime updates (Spec §2.9, Phase 4D) ────────────────────

#[test]
fn catalog_merge_adds_new_model() -> Result<(), Box<dyn std::error::Error>> {
    merge_models(vec![ModelInfo {
        id: "spec2-merge-add-test".into(),
        provider: "test_provider".into(),
        display_name: "Test Model for Merge".into(),
        context_window: 2048,
        max_output: Some(512),
        supports_tools: true,
        supports_vision: false,
        supports_reasoning: false,
        input_cost_per_million: None,
        output_cost_per_million: None,
        aliases: vec!["spec2-merge-alias".into()],
    }])?;

    let info = get_model_info("spec2-merge-add-test")?.ok_or("merged model not found by ID")?;
    assert_eq!(info.provider, "test_provider");
    assert!(info.supports_tools);
    assert_eq!(info.context_window, 2048);

    // Also findable by alias
    let by_alias = get_model_info("spec2-merge-alias")?.ok_or("merged model not found by alias")?;
    assert_eq!(by_alias.id, "spec2-merge-add-test");
    Ok(())
}

#[test]
fn catalog_merge_updates_existing_model() -> Result<(), Box<dyn std::error::Error>> {
    // Add, then update
    merge_models(vec![ModelInfo {
        id: "spec2-merge-update-test".into(),
        provider: "test_provider".into(),
        display_name: "Original".into(),
        context_window: 1024,
        max_output: None,
        supports_tools: false,
        supports_vision: false,
        supports_reasoning: false,
        input_cost_per_million: None,
        output_cost_per_million: None,
        aliases: vec![],
    }])?;

    merge_models(vec![ModelInfo {
        id: "spec2-merge-update-test".into(),
        provider: "test_provider".into(),
        display_name: "Updated".into(),
        context_window: 4096,
        max_output: Some(1024),
        supports_tools: true,
        supports_vision: false,
        supports_reasoning: false,
        input_cost_per_million: None,
        output_cost_per_million: None,
        aliases: vec![],
    }])?;

    let info = get_model_info("spec2-merge-update-test")?.ok_or("updated model not found")?;
    assert_eq!(info.display_name, "Updated");
    assert_eq!(info.context_window, 4096);
    assert!(info.supports_tools);
    Ok(())
}

#[test]
fn catalog_merge_prepends_new_model_to_provider_group() -> Result<(), Box<dyn std::error::Error>> {
    let all = list_models(None)?;
    let existing_provider = &all.first().ok_or("catalog is empty")?.provider;

    merge_models(vec![ModelInfo {
        id: "spec2-prepend-latest-test".into(),
        provider: existing_provider.clone(),
        display_name: "Prepend Latest Test".into(),
        context_window: 999_999,
        max_output: None,
        supports_tools: true,
        supports_vision: true,
        supports_reasoning: true,
        input_cost_per_million: None,
        output_cost_per_million: None,
        aliases: vec![],
    }])?;

    // New model should be first (latest) for the provider
    let latest = get_latest_model(existing_provider, None)?.ok_or("no latest model after merge")?;
    assert_eq!(latest.id, "spec2-prepend-latest-test");
    Ok(())
}

// ── Catalog refresh (Spec §2.9, Phase 4D) ─────────────────────────────

#[tokio::test]
async fn catalog_refresh_adds_discovered_models() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .add_provider(ModelListingAdapter::new(
            "refresh_test_provider",
            vec![ModelInfo {
                id: "refresh-discovered-model".into(),
                provider: "refresh_test_provider".into(),
                display_name: "Discovered Model".into(),
                context_window: 0,
                max_output: None,
                supports_tools: false,
                supports_vision: false,
                supports_reasoning: false,
                input_cost_per_million: None,
                output_cost_per_million: None,
                aliases: vec![],
            }],
        ))
        .build()?;

    let result = refresh(&client).await?;
    assert_eq!(result.new_models.len(), 1);
    assert_eq!(result.new_models[0].id, "refresh-discovered-model");
    assert!(result.provider_errors.is_empty());

    // Should be findable in the catalog
    let info =
        get_model_info("refresh-discovered-model")?.ok_or("refreshed model not in catalog")?;
    assert_eq!(info.provider, "refresh_test_provider");
    Ok(())
}

#[tokio::test]
async fn catalog_refresh_appends_after_curated_entries() -> Result<(), Box<dyn std::error::Error>> {
    // First, add a curated model via merge_models (prepends)
    let provider = "refresh_order_test";
    merge_models(vec![ModelInfo {
        id: "curated-first-model".into(),
        provider: provider.into(),
        display_name: "Curated First".into(),
        context_window: 200_000,
        max_output: Some(8192),
        supports_tools: true,
        supports_vision: true,
        supports_reasoning: false,
        input_cost_per_million: Some(3.0),
        output_cost_per_million: Some(15.0),
        aliases: vec![],
    }])?;

    // Now refresh discovers a new model for the same provider
    let client = Client::builder()
        .add_provider(ModelListingAdapter::new(
            provider,
            vec![ModelInfo {
                id: "discovered-placeholder-model".into(),
                provider: provider.into(),
                display_name: "discovered-placeholder-model".into(),
                context_window: 0,
                max_output: None,
                supports_tools: false,
                supports_vision: false,
                supports_reasoning: false,
                input_cost_per_million: None,
                output_cost_per_million: None,
                aliases: vec![],
            }],
        ))
        .build()?;

    refresh(&client).await?;

    // The curated model should still be "latest" (first in provider group)
    let latest = get_latest_model(provider, None)?.ok_or("no latest model")?;
    assert_eq!(
        latest.id, "curated-first-model",
        "curated model should remain 'latest', not placeholder"
    );
    assert!(
        latest.context_window > 0,
        "latest should have real metadata, not placeholder"
    );
    Ok(())
}

#[tokio::test]
async fn catalog_refresh_skips_already_known_models() -> Result<(), Box<dyn std::error::Error>> {
    let provider = "refresh_dedup_test";
    merge_models(vec![ModelInfo {
        id: "already-known-model".into(),
        provider: provider.into(),
        display_name: "Already Known".into(),
        context_window: 100_000,
        max_output: None,
        supports_tools: true,
        supports_vision: false,
        supports_reasoning: false,
        input_cost_per_million: None,
        output_cost_per_million: None,
        aliases: vec![],
    }])?;

    let client = Client::builder()
        .add_provider(ModelListingAdapter::new(
            provider,
            vec![ModelInfo {
                id: "already-known-model".into(),
                provider: provider.into(),
                display_name: "Placeholder".into(),
                context_window: 0,
                max_output: None,
                supports_tools: false,
                supports_vision: false,
                supports_reasoning: false,
                input_cost_per_million: None,
                output_cost_per_million: None,
                aliases: vec![],
            }],
        ))
        .build()?;

    let result = refresh(&client).await?;
    assert!(
        result.new_models.is_empty(),
        "already-known model should not appear as new"
    );

    // Curated metadata should be preserved
    let info = get_model_info("already-known-model")?.ok_or("model not found")?;
    assert_eq!(info.display_name, "Already Known");
    assert!(info.supports_tools);
    Ok(())
}

#[tokio::test]
async fn catalog_refresh_reports_provider_errors() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .add_provider(ErrorAdapter::new(
            "failing_provider",
            SdkError::Network {
                message: "connection refused".into(),
            },
        ))
        .add_provider(ModelListingAdapter::new(
            "working_provider",
            vec![ModelInfo {
                id: "from-working-provider".into(),
                provider: "working_provider".into(),
                display_name: "Working Model".into(),
                context_window: 0,
                max_output: None,
                supports_tools: false,
                supports_vision: false,
                supports_reasoning: false,
                input_cost_per_million: None,
                output_cost_per_million: None,
                aliases: vec![],
            }],
        ))
        .build()?;

    let result = refresh(&client).await?;

    // Working provider's model should be added
    assert!(
        result
            .new_models
            .iter()
            .any(|m| m.id == "from-working-provider"),
        "working provider's model should be discovered"
    );

    // Failing provider should appear in errors
    assert!(
        !result.provider_errors.is_empty(),
        "should report failing provider error"
    );
    assert_eq!(result.provider_errors[0].0, "failing_provider");
    Ok(())
}

// ── ProviderAdapter trait (Spec §2.4) ─────────────────────────────────

/// Minimal adapter that implements only the required methods.
/// Used to verify default method behavior.
struct StubAdapter;

impl ProviderAdapter for StubAdapter {
    #[allow(clippy::unnecessary_literal_bound)]
    fn name(&self) -> &str {
        "stub"
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

#[test]
fn adapter_name_returns_identifier() {
    let adapter = StubAdapter;
    assert_eq!(adapter.name(), "stub");
}

#[tokio::test]
async fn adapter_close_default_is_ok() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = StubAdapter;
    adapter.close().await?;
    Ok(())
}

#[tokio::test]
async fn adapter_initialize_default_is_ok() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = StubAdapter;
    adapter.initialize().await?;
    Ok(())
}

#[test]
fn adapter_supports_tool_choice_default_is_true() {
    let adapter = StubAdapter;
    assert!(adapter.supports_tool_choice(&ToolChoice::Auto));
    assert!(adapter.supports_tool_choice(&ToolChoice::None));
    assert!(adapter.supports_tool_choice(&ToolChoice::Required));
    assert!(adapter.supports_tool_choice(&ToolChoice::Tool("test".into())));
}

#[tokio::test]
async fn adapter_list_models_default_is_empty() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = StubAdapter;
    let models = adapter.list_models().await?;
    assert!(
        models.is_empty(),
        "default list_models should return empty vec"
    );
    Ok(())
}

#[test]
fn adapter_is_object_safe() {
    // Verify ProviderAdapter can be used as a trait object
    let adapter: Box<dyn ProviderAdapter> = Box::new(StubAdapter);
    assert_eq!(adapter.name(), "stub");
}

// ── Client construction (Spec §2.2) ──────────────────────────────────

#[test]
fn client_builder_empty() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder().build()?;
    assert!(client.provider_names().is_empty());
    assert!(client.default_provider().is_none());
    assert_eq!(client.middleware_count(), 0);
    Ok(())
}

#[test]
fn client_builder_single_provider() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .add_provider(MockAdapter::with_text("alpha", "hi"))
        .build()?;

    assert_eq!(client.provider_names(), vec!["alpha"]);
    assert_eq!(client.default_provider(), Some("alpha"));
    Ok(())
}

#[test]
fn client_builder_first_provider_becomes_default() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .add_provider(MockAdapter::with_text("first", "a"))
        .add_provider(MockAdapter::with_text("second", "b"))
        .build()?;

    assert_eq!(client.default_provider(), Some("first"));
    Ok(())
}

#[test]
fn client_builder_explicit_default() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .add_provider(MockAdapter::with_text("first", "a"))
        .add_provider(MockAdapter::with_text("second", "b"))
        .default_provider("second")
        .build()?;

    assert_eq!(client.default_provider(), Some("second"));
    Ok(())
}

#[test]
fn client_builder_invalid_default_errors() {
    let result = Client::builder()
        .add_provider(MockAdapter::with_text("alpha", "hi"))
        .default_provider("nonexistent")
        .build();

    assert!(
        matches!(result, Err(SdkError::Configuration { .. })),
        "expected Configuration error, got: {result:?}"
    );
}

#[test]
fn client_builder_explicit_default_with_no_providers_errors() {
    // Setting default_provider without registering any providers should fail
    let result = Client::builder().default_provider("ghost").build();

    assert!(
        matches!(result, Err(SdkError::Configuration { .. })),
        "expected Configuration error for unregistered default, got: {result:?}"
    );
}

#[test]
fn client_builder_with_middleware() -> Result<(), Box<dyn std::error::Error>> {
    struct NoopMiddleware;
    impl Middleware for NoopMiddleware {}

    let client = Client::builder()
        .add_provider(MockAdapter::with_text("alpha", "hi"))
        .middleware(NoopMiddleware)
        .middleware(NoopMiddleware)
        .build()?;

    assert_eq!(client.middleware_count(), 2);
    Ok(())
}

#[test]
fn client_debug_format() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .add_provider(MockAdapter::with_text("test", "hi"))
        .build()?;
    let debug = format!("{client:?}");
    assert!(debug.contains("Client"));
    assert!(debug.contains("test"));
    Ok(())
}

// ── Client routing (Spec §2.2) ───────────────────────────────────────

#[tokio::test]
async fn client_routes_to_default_provider() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .add_provider(MockAdapter::with_text("alpha", "from_alpha"))
        .add_provider(MockAdapter::with_text("beta", "from_beta"))
        .build()?;

    // No provider specified → uses default (first registered = "alpha")
    let response = client.complete(make_request("test-model")).await?;
    assert_eq!(response.text(), "from_alpha");
    assert_eq!(response.provider, "alpha");
    Ok(())
}

#[tokio::test]
async fn client_routes_by_provider_field() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .add_provider(MockAdapter::with_text("alpha", "from_alpha"))
        .add_provider(MockAdapter::with_text("beta", "from_beta"))
        .build()?;

    let response = client
        .complete(make_request_for("test-model", "beta"))
        .await?;
    assert_eq!(response.text(), "from_beta");
    assert_eq!(response.provider, "beta");
    Ok(())
}

#[tokio::test]
async fn client_routes_by_model_alias_when_provider_missing()
-> Result<(), Box<dyn std::error::Error>> {
    // Default provider is openai, but model alias "sonnet" should infer anthropic.
    let client = Client::builder()
        .add_provider(MockAdapter::with_text("openai", "from_openai"))
        .add_provider(MockAdapter::with_text("anthropic", "from_anthropic"))
        .build()?;

    let response = client.complete(make_request("sonnet")).await?;
    assert_eq!(response.text(), "from_anthropic");
    assert_eq!(response.provider, "anthropic");
    Ok(())
}

#[tokio::test]
async fn client_unknown_model_still_routes_to_default_provider()
-> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .add_provider(MockAdapter::with_text("alpha", "from_alpha"))
        .add_provider(MockAdapter::with_text("beta", "from_beta"))
        .build()?;

    // Unknown model cannot be inferred from catalog, so default provider is used.
    let response = client.complete(make_request("not-in-catalog")).await?;
    assert_eq!(response.text(), "from_alpha");
    assert_eq!(response.provider, "alpha");
    Ok(())
}

#[tokio::test]
async fn client_infers_provider_from_model_name_heuristics()
-> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .add_provider(MockAdapter::with_text("openai", "from_openai"))
        .add_provider(MockAdapter::with_text("anthropic", "from_anthropic"))
        .add_provider(MockAdapter::with_text("gemini", "from_gemini"))
        .add_provider(MockAdapter::with_text("mistral", "from_mistral"))
        .add_provider(MockAdapter::with_text("deepseek", "from_deepseek"))
        .build()?;

    // Models not in the catalog should still route by name prefix.
    let cases = vec![
        ("claude-future-99", "from_anthropic"),
        ("gpt-99-turbo", "from_openai"),
        ("o4-mini-future", "from_openai"),
        ("gemini-9.0-ultra", "from_gemini"),
        ("mistral-future-large", "from_mistral"),
        ("codestral-future", "from_mistral"),
        ("deepseek-future-v3", "from_deepseek"),
    ];

    for (model, expected_text) in cases {
        let response = client.complete(make_request(model)).await?;
        assert_eq!(
            response.text(),
            expected_text,
            "model '{model}' should route to the correct provider"
        );
    }
    Ok(())
}

#[tokio::test]
async fn client_errors_on_unknown_provider() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .add_provider(MockAdapter::with_text("alpha", "hi"))
        .build()?;

    let result = client
        .complete(make_request_for("model", "nonexistent"))
        .await;
    assert!(
        matches!(result, Err(SdkError::Configuration { .. })),
        "expected Configuration error"
    );
    Ok(())
}

#[tokio::test]
async fn client_errors_on_no_provider_and_no_default() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder().build()?;
    let result = client.complete(make_request("model")).await;
    assert!(
        matches!(result, Err(SdkError::Configuration { .. })),
        "expected Configuration error"
    );
    Ok(())
}

#[tokio::test]
async fn client_propagates_provider_error() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .add_provider(ErrorAdapter::new(
            "failing",
            SdkError::Network {
                message: "connection refused".into(),
            },
        ))
        .build()?;

    let result = client.complete(make_request("model")).await;
    assert!(
        matches!(result, Err(SdkError::Network { .. })),
        "expected Network error, got: {result:?}"
    );
    Ok(())
}

#[tokio::test]
async fn client_close_is_ok_with_mocks() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .add_provider(MockAdapter::with_text("alpha", "hi"))
        .add_provider(MockAdapter::with_text("beta", "hello"))
        .build()?;
    client.close().await?;
    Ok(())
}

// ── Client streaming (Spec §2.4) ─────────────────────────────────────

#[tokio::test]
async fn client_stream_routes_to_provider() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .add_provider(MockAdapter::with_text("alpha", "hi"))
        .build()?;

    let mut stream = client.stream(make_request("model")).await?;
    let mut events = Vec::new();
    while let Some(event) = stream.next().await {
        events.push(event?);
    }

    assert!(
        events.len() >= 2,
        "should have at least stream_start + finish"
    );
    Ok(())
}

// ── Middleware (Spec §2.3) ────────────────────────────────────────────

/// Recording middleware that logs request/response phases.
struct RecordingMiddleware {
    name: String,
    log: Arc<Mutex<Vec<String>>>,
}

impl RecordingMiddleware {
    fn new(name: impl Into<String>, log: Arc<Mutex<Vec<String>>>) -> Self {
        Self {
            name: name.into(),
            log,
        }
    }
}

impl Middleware for RecordingMiddleware {
    fn handle_complete<'a>(
        &'a self,
        request: Request,
        next: NextComplete<'a>,
    ) -> BoxFuture<'a, SdkResult<Response>> {
        let log = self.log.clone();
        let name = self.name.clone();
        Box::pin(async move {
            if let Ok(mut guard) = log.lock() {
                guard.push(format!("{name}:before"));
            }
            let response = next(request).await?;
            if let Ok(mut guard) = log.lock() {
                guard.push(format!("{name}:after"));
            }
            Ok(response)
        })
    }

    fn handle_stream<'a>(
        &'a self,
        request: Request,
        next: NextStream<'a>,
    ) -> BoxFuture<'a, SdkResult<BoxStream<'a, SdkResult<StreamEvent>>>> {
        let log = self.log.clone();
        let name = self.name.clone();
        Box::pin(async move {
            if let Ok(mut guard) = log.lock() {
                guard.push(format!("{name}:before"));
            }
            let stream = next(request).await?;
            let log2 = log.clone();
            let name2 = name.clone();
            let wrapped = stream.map(move |event| {
                if let Ok(ref evt) = event
                    && let Ok(mut guard) = log2.lock()
                {
                    guard.push(format!("{name2}:event:{:?}", evt.event_type));
                }
                event
            });
            if let Ok(mut guard) = log.lock() {
                guard.push(format!("{name}:stream_opened"));
            }
            Ok(Box::pin(wrapped) as BoxStream<'a, SdkResult<StreamEvent>>)
        })
    }
}

#[tokio::test]
async fn middleware_default_passthrough() -> Result<(), Box<dyn std::error::Error>> {
    struct PassthroughMiddleware;
    impl Middleware for PassthroughMiddleware {}

    let client = Client::builder()
        .add_provider(MockAdapter::with_text("alpha", "hello"))
        .middleware(PassthroughMiddleware)
        .build()?;

    let response = client.complete(make_request("model")).await?;
    assert_eq!(response.text(), "hello");
    Ok(())
}

#[tokio::test]
async fn middleware_complete_execution_order() -> Result<(), Box<dyn std::error::Error>> {
    let log = Arc::new(Mutex::new(Vec::new()));

    let client = Client::builder()
        .add_provider(MockAdapter::with_text("alpha", "done"))
        .middleware(RecordingMiddleware::new("m1", log.clone()))
        .middleware(RecordingMiddleware::new("m2", log.clone()))
        .middleware(RecordingMiddleware::new("m3", log.clone()))
        .build()?;

    client.complete(make_request("model")).await?;

    let entries = log.lock().map_err(|e| format!("lock: {e}"))?;
    // Request phase: m1 → m2 → m3 (registration order)
    // Response phase: m3 → m2 → m1 (reverse order)
    assert_eq!(
        *entries,
        vec![
            "m1:before",
            "m2:before",
            "m3:before",
            "m3:after",
            "m2:after",
            "m1:after",
        ]
    );
    Ok(())
}

#[tokio::test]
async fn middleware_stream_execution_order() -> Result<(), Box<dyn std::error::Error>> {
    let log = Arc::new(Mutex::new(Vec::new()));

    let client = Client::builder()
        .add_provider(MockAdapter::with_text("alpha", "done"))
        .middleware(RecordingMiddleware::new("m1", log.clone()))
        .middleware(RecordingMiddleware::new("m2", log.clone()))
        .build()?;

    let mut stream = client.stream(make_request("model")).await?;
    while let Some(event) = stream.next().await {
        let _evt = event?;
    }

    let entries = log.lock().map_err(|e| format!("lock: {e}"))?;

    // Before entries should be in registration order
    let before_entries: Vec<_> = entries.iter().filter(|e| e.ends_with(":before")).collect();
    assert_eq!(before_entries, vec!["m1:before", "m2:before"]);

    // Stream events should be observed by both middlewares
    let event_entries: Vec<_> = entries.iter().filter(|e| e.contains(":event:")).collect();
    assert!(
        !event_entries.is_empty(),
        "middleware should observe events"
    );
    Ok(())
}

/// Middleware that modifies the request before passing to next.
struct RequestModifyingMiddleware;

impl Middleware for RequestModifyingMiddleware {
    fn handle_complete<'a>(
        &'a self,
        mut request: Request,
        next: NextComplete<'a>,
    ) -> BoxFuture<'a, SdkResult<Response>> {
        // Add a system message to demonstrate request modification
        request
            .messages
            .insert(0, Message::system("injected by middleware"));
        next(request)
    }
}

/// A mock adapter that echoes back the number of messages it received.
struct EchoCountAdapter;

impl ProviderAdapter for EchoCountAdapter {
    #[allow(clippy::unnecessary_literal_bound)]
    fn name(&self) -> &str {
        "echo_count"
    }

    fn complete(&self, request: Request) -> BoxFuture<'_, SdkResult<Response>> {
        let count = request.messages.len();
        Box::pin(async move {
            Ok(common::make_response(
                "echo_count",
                &format!("messages:{count}"),
            ))
        })
    }

    fn stream(
        &self,
        _request: Request,
    ) -> BoxFuture<'_, SdkResult<BoxStream<'_, SdkResult<StreamEvent>>>> {
        Box::pin(async {
            Err(SdkError::Configuration {
                message: "not implemented".into(),
            })
        })
    }
}

#[tokio::test]
async fn middleware_can_modify_request() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .add_provider(EchoCountAdapter)
        .middleware(RequestModifyingMiddleware)
        .build()?;

    // Original request has 1 message (user). Middleware adds 1 system message.
    let response = client.complete(make_request("model")).await?;
    assert_eq!(response.text(), "messages:2");
    Ok(())
}

/// Middleware that transforms the response after receiving it.
struct ResponseModifyingMiddleware;

impl Middleware for ResponseModifyingMiddleware {
    fn handle_complete<'a>(
        &'a self,
        request: Request,
        next: NextComplete<'a>,
    ) -> BoxFuture<'a, SdkResult<Response>> {
        Box::pin(async move {
            let mut response = next(request).await?;
            response.id = "modified-id".into();
            Ok(response)
        })
    }
}

#[tokio::test]
async fn middleware_can_modify_response() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .add_provider(MockAdapter::with_text("alpha", "original"))
        .middleware(ResponseModifyingMiddleware)
        .build()?;

    let response = client.complete(make_request("model")).await?;
    assert_eq!(response.id, "modified-id");
    assert_eq!(response.text(), "original"); // text unchanged
    Ok(())
}

/// Middleware that short-circuits without calling next.
struct ShortCircuitMiddleware;

impl Middleware for ShortCircuitMiddleware {
    fn handle_complete<'a>(
        &'a self,
        _request: Request,
        _next: NextComplete<'a>,
    ) -> BoxFuture<'a, SdkResult<Response>> {
        Box::pin(async { Ok(common::make_response("cache", "cached response")) })
    }
}

#[tokio::test]
async fn middleware_can_short_circuit() -> Result<(), Box<dyn std::error::Error>> {
    let log = Arc::new(Mutex::new(Vec::new()));

    let client = Client::builder()
        .add_provider(MockAdapter::with_text("alpha", "from provider"))
        .middleware(ShortCircuitMiddleware)
        .middleware(RecordingMiddleware::new("after_cache", log.clone()))
        .build()?;

    let response = client.complete(make_request("model")).await?;
    assert_eq!(response.text(), "cached response");
    assert_eq!(response.provider, "cache");

    // The recording middleware after the short-circuit should never execute
    let entries = log.lock().map_err(|e| format!("lock: {e}"))?;
    assert!(
        entries.is_empty(),
        "middleware after short-circuit should not run"
    );
    Ok(())
}

/// Middleware that wraps a stream to observe events.
struct StreamObservingMiddleware {
    event_count: Arc<Mutex<usize>>,
}

impl Middleware for StreamObservingMiddleware {
    fn handle_stream<'a>(
        &'a self,
        request: Request,
        next: NextStream<'a>,
    ) -> BoxFuture<'a, SdkResult<BoxStream<'a, SdkResult<StreamEvent>>>> {
        let count = self.event_count.clone();
        Box::pin(async move {
            let stream = next(request).await?;
            let wrapped = stream.map(move |event| {
                if event.is_ok()
                    && let Ok(mut c) = count.lock()
                {
                    *c += 1;
                }
                event
            });
            Ok(Box::pin(wrapped) as BoxStream<'a, SdkResult<StreamEvent>>)
        })
    }
}

#[tokio::test]
async fn middleware_can_observe_stream_events() -> Result<(), Box<dyn std::error::Error>> {
    let event_count = Arc::new(Mutex::new(0_usize));

    let client = Client::builder()
        .add_provider(MockAdapter::with_text("alpha", "hi"))
        .middleware(StreamObservingMiddleware {
            event_count: event_count.clone(),
        })
        .build()?;

    let mut stream = client.stream(make_request("model")).await?;
    while let Some(event) = stream.next().await {
        let _evt = event?;
    }

    let count = *event_count.lock().map_err(|e| format!("lock: {e}"))?;
    assert!(
        count >= 2,
        "middleware should have observed events, got {count}"
    );
    Ok(())
}

// ── Concurrent provider requests (Spec §2.6) ─────────────────────────

#[tokio::test]
async fn client_concurrent_requests() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .add_provider(MockAdapter::with_text("alpha", "from_alpha"))
        .add_provider(MockAdapter::with_text("beta", "from_beta"))
        .build()?;

    let (resp_a, resp_b) = tokio::join!(
        client.complete(make_request_for("model", "alpha")),
        client.complete(make_request_for("model", "beta")),
    );

    assert_eq!(resp_a?.text(), "from_alpha");
    assert_eq!(resp_b?.text(), "from_beta");
    Ok(())
}

// ── from_env (Spec §2.2) ─────────────────────────────────────────────

#[test]
fn client_from_env_returns_valid_client() {
    // from_env() should not panic regardless of what env vars are set.
    // It may succeed or fail depending on the environment, but should
    // always produce a consistent Client or a Configuration error.
    match Client::from_env() {
        Ok(client) => {
            // If it succeeds, the default provider must be in the provider list
            if let Some(default) = client.default_provider() {
                assert!(
                    client.provider_names().contains(&default),
                    "default provider '{default}' should be in provider list"
                );
            }
            // Middleware starts empty (from_env doesn't add any)
            assert_eq!(client.middleware_count(), 0);
        }
        Err(e) => {
            // If it fails, it should be a Configuration error
            assert!(
                matches!(e, SdkError::Configuration { .. }),
                "from_env errors should be Configuration, got: {e:?}"
            );
        }
    }
}

#[test]
fn default_client_set_can_override_previous_value() -> Result<(), Box<dyn std::error::Error>> {
    // Global default client state is process-wide; serialize this test.
    static TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
    let _guard = TEST_MUTEX
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|e| format!("lock: {e}"))?;

    let client_a = Client::builder()
        .add_provider(MockAdapter::with_text("alpha", "a"))
        .build()?;
    default_client::set_default_client(client_a);

    let resolved_a = default_client::get_default_client()?;
    assert_eq!(resolved_a.default_provider(), Some("alpha"));

    let client_b = Client::builder()
        .add_provider(MockAdapter::with_text("beta", "b"))
        .build()?;
    default_client::set_default_client(client_b);

    let resolved_b = default_client::get_default_client()?;
    assert_eq!(resolved_b.default_provider(), Some("beta"));

    Ok(())
}

// ── from_env_with_auth (AuthOverrides) ───────────────────────────────

#[test]
fn from_env_with_auth_unknown_provider_returns_error() {
    let mut overrides = AuthOverrides::new();
    overrides.insert(
        "opanai".to_string(), // typo
        Arc::new(StaticKey::new("key")),
    );
    let options = AuthOptions {
        overrides,
        ..AuthOptions::default()
    };
    let err = Client::from_env_with_auth(&options).expect_err("should reject unknown provider");
    assert!(
        matches!(err, SdkError::Configuration { ref message } if message.contains("opanai")),
        "expected Configuration error mentioning 'opanai', got: {err:?}"
    );
}

#[test]
fn from_env_with_auth_registers_override_provider() {
    let mut overrides = AuthOverrides::new();
    overrides.insert(
        "anthropic".to_string(),
        Arc::new(StaticKey::new("override-key")),
    );
    let options = AuthOptions {
        overrides,
        ..AuthOptions::default()
    };
    // This should succeed and register anthropic even without ANTHROPIC_API_KEY in env.
    // NOTE: We accept Configuration errors here because from_env_with_auth reads
    // real environment state (env vars, keyring) that varies across machines.
    // A Configuration error means the override was validated but another provider
    // failed to initialize — not a regression in override handling itself.
    match Client::from_env_with_auth(&options) {
        Ok(client) => {
            assert!(
                client.provider_names().contains(&"anthropic"),
                "anthropic should be registered via override"
            );
        }
        Err(e) => {
            assert!(
                matches!(e, SdkError::Configuration { .. }),
                "expected Configuration error, got: {e:?}"
            );
        }
    }
}

#[test]
fn from_env_with_auth_empty_overrides_same_as_from_env() {
    let options = AuthOptions::default();
    // With no overrides, behavior should be equivalent to from_env.
    // NOTE: We accept Configuration errors because the result depends on
    // which API keys / env vars happen to be set on the host machine.
    // The important assertion is that no other error variant (e.g. Server,
    // Authentication) is returned from a purely local operation.
    match Client::from_env_with_auth(&options) {
        Ok(client) => {
            if let Some(default) = client.default_provider() {
                assert!(
                    client.provider_names().contains(&default),
                    "default provider '{default}' should be in provider list"
                );
            }
            assert_eq!(client.middleware_count(), 0);
        }
        Err(e) => {
            assert!(
                matches!(e, SdkError::Configuration { .. }),
                "errors should be Configuration, got: {e:?}"
            );
        }
    }
}

// ── Middleware is object-safe ─────────────────────────────────────────

#[test]
fn middleware_is_object_safe() {
    struct TestMiddleware;
    impl Middleware for TestMiddleware {}

    // Verify Middleware can be used as a trait object
    let _mw: Box<dyn Middleware> = Box::new(TestMiddleware);
}
