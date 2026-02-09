//! Shared helpers for env-gated integration tests.
//!
//! All real-provider tests should call [`require_provider`] at the top.
//! Tests skip (return `Ok(())`) rather than fail when keys are absent.

use stencila_models3::client::Client;
use stencila_models3::error::{SdkError, SdkResult};

/// Check whether the API key env-var for `provider` is set.
/// Returns `true` when the provider is available for testing.
pub fn has_provider(provider: &str) -> bool {
    match provider {
        "openai" => std::env::var("OPENAI_API_KEY").is_ok(),
        "anthropic" => std::env::var("ANTHROPIC_API_KEY").is_ok(),
        "gemini" => {
            std::env::var("GEMINI_API_KEY").is_ok() || std::env::var("GOOGLE_API_KEY").is_ok()
        }
        _ => false,
    }
}

/// Build a client that only includes providers whose keys are present.
///
/// # Errors
///
/// Returns `SdkError::Configuration` if adapter construction fails.
pub fn client_from_env() -> SdkResult<Client> {
    Client::from_env()
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

/// Return a model with image-input support for vision acceptance tests.
pub fn vision_test_model(provider: &str) -> &'static str {
    match provider {
        "openai" => "gpt-4o-mini",
        "anthropic" => "claude-sonnet-4-5-20250929",
        "gemini" => "gemini-2.0-flash",
        _ => "unknown",
    }
}

/// Normalize requested max tokens per provider for acceptance tests.
///
/// OpenAI Responses currently enforces a minimum `max_output_tokens` of 16.
#[must_use]
pub fn provider_max_tokens(provider: &str, requested: u64) -> u64 {
    match provider {
        "openai" => requested.max(16),
        _ => requested,
    }
}

/// Whether a live-provider integration error should skip the current provider.
///
/// Env-gated tests should avoid hard failures for temporary provider-side limits.
#[must_use]
pub fn should_skip_live_provider_error(error: &SdkError) -> bool {
    matches!(
        error,
        SdkError::RateLimit { .. } | SdkError::QuotaExceeded { .. }
    )
}

/// A simple weather tool definition for tool-call tests.
pub fn weather_tool_definition() -> stencila_models3::types::tool::ToolDefinition {
    stencila_models3::types::tool::ToolDefinition {
        name: "get_weather".into(),
        description: "Get the current weather for a city.".into(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "city": {
                    "type": "string",
                    "description": "The city name"
                }
            },
            "required": ["city"]
        }),
        strict: false,
    }
}

/// A mock weather handler that returns deterministic data.
pub fn weather_handler(
    args: serde_json::Value,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = SdkResult<serde_json::Value>> + Send>> {
    Box::pin(async move {
        let city = args
            .get("city")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");
        Ok(serde_json::json!({
            "city": city,
            "temperature": "72°F",
            "condition": "sunny"
        }))
    })
}

/// JSON schema for structured output tests (person extraction).
pub fn person_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "name": { "type": "string" },
            "age": { "type": "integer" }
        },
        "required": ["name", "age"]
    })
}

/// Provider-specific schema for structured-output acceptance tests.
pub fn person_schema_for(provider: &str) -> serde_json::Value {
    match provider {
        // OpenAI strict json_schema requires explicit additionalProperties=false.
        "openai" => serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "integer" }
            },
            "required": ["name", "age"],
            "additionalProperties": false
        }),
        _ => person_schema(),
    }
}
