//! Spec Section 2 conformance tests.
//!
//! Target areas:
//! - Provider adapter interface conformance
//! - Client routing and default provider resolution
//! - Middleware behavior for both `complete()` and `stream()`
//! - Default client initialization and override behavior
//! - Model catalog lookup/list/latest helpers

mod common;

use stencila_models3::catalog::{get_latest_model, get_model_info, list_models};
use stencila_models3::error::{SdkError, SdkResult};
use stencila_models3::provider::{BoxFuture, BoxStream, ProviderAdapter};
use stencila_models3::types::request::Request;
use stencila_models3::types::response::Response;
use stencila_models3::types::stream_event::StreamEvent;
use stencila_models3::types::tool::ToolChoice;

// ── Model catalog (Spec §2.9) ─────────────────────────────────────────

#[test]
fn catalog_contains_all_providers() -> Result<(), Box<dyn std::error::Error>> {
    let providers: Vec<&str> = list_models(None)?
        .iter()
        .map(|m| m.provider.as_str())
        .collect();
    assert!(providers.contains(&"anthropic"));
    assert!(providers.contains(&"openai"));
    assert!(providers.contains(&"gemini"));
    Ok(())
}

#[test]
fn catalog_lookup_by_id() -> Result<(), Box<dyn std::error::Error>> {
    // Use the first model in the catalog — no hardcoded model ID
    let all = list_models(None)?;
    let first = all.first().ok_or("catalog is empty")?;
    let info = get_model_info(&first.id)?.ok_or("lookup by id failed")?;
    assert_eq!(info.id, first.id);
    assert!(info.context_window > 0);
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
    let all = list_models(None)?;
    assert!(!all.is_empty(), "catalog should not be empty");
    for info in &all {
        assert!(!info.id.is_empty(), "id must be non-empty");
        assert!(!info.provider.is_empty(), "provider must be non-empty");
        assert!(
            !info.display_name.is_empty(),
            "display_name must be non-empty"
        );
        assert!(info.context_window > 0, "context_window must be positive");
    }
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

#[test]
fn adapter_is_object_safe() {
    // Verify ProviderAdapter can be used as a trait object
    let adapter: Box<dyn ProviderAdapter> = Box::new(StubAdapter);
    assert_eq!(adapter.name(), "stub");
}
