//! Spec Section 8 acceptance tests.
//!
//! These are env-gated integration tests that exercise real provider APIs.
//! They implement the cross-provider parity matrix (§8.9) and the
//! integration smoke test (§8.10).
//!
//! Skip automatically when the required API keys are absent.
//!
//! Run with:
//! ```sh
//! OPENAI_API_KEY=... ANTHROPIC_API_KEY=... GEMINI_API_KEY=... MISTRAL_API_KEY=... DEEPSEEK_API_KEY=... \
//!   cargo test -p stencila-models3 -- spec_8
//! ```
#![allow(clippy::result_large_err)]

mod integration;

use base64::Engine;
use stencila_models3::api::cancel::AbortController;
use stencila_models3::api::generate::{GenerateOptions, generate};
use stencila_models3::api::generate_object::{GenerateObjectOptions, generate_object};
use stencila_models3::api::stream::{StreamOptions, stream_generate};
use stencila_models3::api::types::Tool;
use stencila_models3::client::Client;
use stencila_models3::error::{SdkError, SdkResult};
use stencila_models3::types::content::ContentPart;
use stencila_models3::types::finish_reason::Reason;
use stencila_models3::types::message::Message;
use stencila_models3::types::stream_event::StreamEventType;

use integration::helpers;

/// Collect available providers (those with API keys set).
fn available_providers(names: &[&'static str]) -> Vec<&'static str> {
    names
        .iter()
        .copied()
        .filter(|p| helpers::has_provider(p))
        .collect()
}

fn should_skip_provider_error(provider: &str, error: &SdkError) -> bool {
    let _ = provider;
    helpers::should_skip_live_provider_error(error)
}

fn is_openai_invalid_image_error(error: &SdkError) -> bool {
    matches!(
        error,
        SdkError::InvalidRequest { details, .. }
            if details.provider.as_deref() == Some("openai")
                && details.error_code.as_deref() == Some("invalid_value")
    )
}

// ---------------------------------------------------------------------------
// §8.10 — Integration Smoke Tests
// ---------------------------------------------------------------------------

/// §8.10 test 1: Basic generation across all available providers.
#[tokio::test]
async fn smoke_basic_generation() -> SdkResult<()> {
    let available = available_providers(&["openai", "anthropic", "gemini", "mistral", "deepseek"]);
    if available.is_empty() {
        return Ok(());
    }

    let client = helpers::client_from_env()?;

    for &provider in &available {
        let model = helpers::test_model(provider);
        let opts = GenerateOptions::new(model)
            .prompt("Say hello in one sentence.")
            .max_tokens(helpers::provider_max_tokens(provider, 100))
            .provider(provider)
            .client(&client);

        let result = match generate(opts).await {
            Ok(result) => result,
            Err(error) if should_skip_provider_error(provider, &error) => continue,
            Err(error) => return Err(error),
        };

        assert!(
            !result.text.is_empty(),
            "{provider}: expected non-empty text"
        );
        assert!(
            result.usage.input_tokens > 0,
            "{provider}: expected input_tokens > 0"
        );
        assert!(
            result.usage.output_tokens > 0,
            "{provider}: expected output_tokens > 0"
        );
        assert_eq!(
            result.finish_reason.reason,
            Reason::Stop,
            "{provider}: expected stop finish reason"
        );
    }

    Ok(())
}

/// §8.10 test 2: Streaming text generation.
///
/// Verifies that concatenated TEXT_DELTA events match the final response text.
#[tokio::test]
async fn smoke_streaming() -> SdkResult<()> {
    let available = available_providers(&["openai", "anthropic", "gemini", "mistral", "deepseek"]);
    if available.is_empty() {
        return Ok(());
    }

    let client = helpers::client_from_env()?;

    for &provider in &available {
        let model = helpers::test_model(provider);
        let opts = StreamOptions::new(model)
            .prompt("Write a haiku about code.")
            .max_tokens(helpers::provider_max_tokens(provider, 100))
            .provider(provider)
            .client(&client);

        let mut stream_result = match stream_generate(opts).await {
            Ok(stream_result) => stream_result,
            Err(error) if should_skip_provider_error(provider, &error) => continue,
            Err(error) => return Err(error),
        };

        let mut text_chunks = Vec::new();
        let mut saw_start = false;
        let mut saw_finish = false;
        let mut skip_provider = false;

        while let Some(event) = stream_result.next_event().await {
            let event = match event {
                Ok(event) => event,
                Err(error) if should_skip_provider_error(provider, &error) => {
                    skip_provider = true;
                    break;
                }
                Err(error) => return Err(error),
            };
            match event.event_type {
                StreamEventType::StreamStart => saw_start = true,
                StreamEventType::TextDelta => {
                    if let Some(delta) = event.delta {
                        text_chunks.push(delta);
                    }
                }
                StreamEventType::Finish => saw_finish = true,
                _ => {}
            }
        }
        if skip_provider {
            continue;
        }

        let concatenated: String = text_chunks.concat();
        assert!(
            !concatenated.is_empty(),
            "{provider}: expected non-empty streamed text"
        );
        assert!(saw_start, "{provider}: expected StreamStart event");
        assert!(saw_finish, "{provider}: expected Finish event");

        // The concatenated deltas should match the accumulated response
        let final_resp = stream_result
            .response()
            .unwrap_or_else(|| stream_result.partial_response());
        assert_eq!(
            concatenated,
            final_resp.text(),
            "{provider}: concatenated deltas should match response.text()"
        );
    }

    Ok(())
}

/// §8.10 test 3: Tool calling with active tools.
///
/// Uses a weather tool to verify the model can call tools and the SDK
/// executes them, feeding results back for a multi-step loop.
#[tokio::test]
async fn smoke_tool_calling() -> SdkResult<()> {
    let available = available_providers(&["openai", "anthropic", "gemini", "mistral", "deepseek"]);
    if available.is_empty() {
        return Ok(());
    }

    let client = helpers::client_from_env()?;

    for &provider in &available {
        let model = helpers::test_model(provider);
        let tool = Tool::active(helpers::weather_tool_definition(), helpers::weather_handler);

        let opts = GenerateOptions::new(model)
            .prompt("What is the weather in San Francisco? Use the get_weather tool.")
            .tools(vec![tool])
            .max_tool_rounds(3)
            .max_tokens(helpers::provider_max_tokens(provider, 300))
            .provider(provider)
            .client(&client);

        let result = match generate(opts).await {
            Ok(result) => result,
            Err(error) if should_skip_provider_error(provider, &error) => continue,
            Err(error) => return Err(error),
        };

        // Should have at least 2 steps: initial call + after tool results
        assert!(
            result.steps.len() >= 2,
            "{provider}: expected at least 2 steps, got {}",
            result.steps.len()
        );

        // Final text should reference the weather data
        assert!(
            !result.text.is_empty(),
            "{provider}: expected non-empty final text"
        );
    }

    Ok(())
}

/// §8.10 test 5: Structured output via `generate_object()`.
///
/// Note: Anthropic does not natively support json_schema response format,
/// so we only test OpenAI and Gemini here.
#[tokio::test]
async fn smoke_structured_output() -> SdkResult<()> {
    let available = available_providers(&["openai", "gemini", "mistral", "deepseek"]);
    if available.is_empty() {
        return Ok(());
    }

    let client = helpers::client_from_env()?;

    for &provider in &available {
        let model = helpers::test_model(provider);
        let opts = GenerateObjectOptions::new(model, helpers::person_schema_for(provider))
            .prompt("Extract: Alice is 30 years old")
            .max_tokens(helpers::provider_max_tokens(provider, 200))
            .provider(provider)
            .client(&client);

        let result = match generate_object(opts).await {
            Ok(result) => result,
            Err(error) if should_skip_provider_error(provider, &error) => continue,
            Err(error) => return Err(error),
        };

        let output = result
            .output
            .as_ref()
            .ok_or_else(|| SdkError::Configuration {
                message: format!("{provider}: expected output to be populated"),
            })?;

        assert_eq!(
            output.get("name").and_then(|v| v.as_str()),
            Some("Alice"),
            "{provider}: expected name=Alice"
        );
        assert_eq!(
            output.get("age").and_then(|v| v.as_i64()),
            Some(30),
            "{provider}: expected age=30"
        );
    }

    Ok(())
}

/// §8.10 test 6: Error handling — nonexistent model.
///
/// Verifies that a request for a model that doesn't exist returns
/// an appropriate error (NotFound, InvalidRequest, or Server error).
#[tokio::test]
async fn smoke_error_handling_nonexistent_model() -> SdkResult<()> {
    let available = available_providers(&["openai", "anthropic", "gemini", "mistral", "deepseek"]);
    if available.is_empty() {
        return Ok(());
    }

    let client = helpers::client_from_env()?;

    for &provider in &available {
        let opts = GenerateOptions::new("nonexistent-model-xyz-12345")
            .prompt("test")
            .max_tokens(helpers::provider_max_tokens(provider, 16))
            .max_retries(0)
            .provider(provider)
            .client(&client);

        let result = generate(opts).await;
        let err = match result {
            Ok(_) => {
                return Err(SdkError::Configuration {
                    message: format!("{provider}: expected error for nonexistent model"),
                });
            }
            Err(error) if should_skip_provider_error(provider, &error) => continue,
            Err(error) => error,
        };

        // The error should be a provider error (not a configuration error)
        let is_provider_error = matches!(
            err,
            SdkError::NotFound { .. } | SdkError::InvalidRequest { .. } | SdkError::Server { .. }
        );
        assert!(
            is_provider_error,
            "{provider}: expected NotFound/InvalidRequest/Server, got: {err}"
        );
    }

    Ok(())
}

/// §8.10 test 4: Cancellation — pre-aborted signal prevents the request.
///
/// Verifies that `generate()` and `stream_generate()` immediately return
/// `SdkError::Abort` when the abort signal is already triggered.
#[tokio::test]
async fn smoke_cancellation() -> SdkResult<()> {
    let available = available_providers(&["openai", "anthropic", "gemini", "mistral", "deepseek"]);
    if available.is_empty() {
        return Ok(());
    }

    let client = helpers::client_from_env()?;

    for &provider in &available {
        let model = helpers::test_model(provider);

        // generate() with a pre-aborted signal
        let controller = AbortController::new();
        controller.abort();

        let opts = GenerateOptions::new(model)
            .prompt("Say hello.")
            .max_tokens(helpers::provider_max_tokens(provider, 16))
            .abort_signal(controller.signal())
            .provider(provider)
            .client(&client);

        let result = generate(opts).await;
        assert!(
            matches!(result, Err(SdkError::Abort { .. })),
            "{provider}: generate() should return Abort error, got: {result:?}"
        );

        // stream_generate() with a pre-aborted signal
        let controller = AbortController::new();
        controller.abort();

        let opts = StreamOptions::new(model)
            .prompt("Say hello.")
            .max_tokens(helpers::provider_max_tokens(provider, 16))
            .abort_signal(controller.signal())
            .provider(provider)
            .client(&client);

        let result = stream_generate(opts).await;
        assert!(
            matches!(result, Err(SdkError::Abort { .. })),
            "{provider}: stream_generate() should return Abort error"
        );
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// §8.9 — Cross-Provider Parity Matrix
// ---------------------------------------------------------------------------

/// §8.9 parity: Simple text generation across all providers.
#[tokio::test]
async fn parity_simple_text_generation() -> SdkResult<()> {
    let available = available_providers(&["openai", "anthropic", "gemini", "mistral", "deepseek"]);
    if available.is_empty() {
        return Ok(());
    }

    let client = helpers::client_from_env()?;

    for &provider in &available {
        let model = helpers::test_model(provider);
        let opts = GenerateOptions::new(model)
            .prompt("What is 2+2? Answer with just the number.")
            .max_tokens(helpers::provider_max_tokens(provider, 16))
            .provider(provider)
            .client(&client);

        let result = match generate(opts).await {
            Ok(result) => result,
            Err(error) if should_skip_provider_error(provider, &error) => continue,
            Err(error) => return Err(error),
        };
        assert!(
            result.text.contains('4'),
            "{provider}: expected '4' in response, got: {}",
            result.text
        );
    }

    Ok(())
}

/// §8.9 parity: Streaming text generation across all providers.
#[tokio::test]
async fn parity_streaming_text() -> SdkResult<()> {
    let available = available_providers(&["openai", "anthropic", "gemini", "mistral", "deepseek"]);
    if available.is_empty() {
        return Ok(());
    }

    let client = helpers::client_from_env()?;

    for &provider in &available {
        let model = helpers::test_model(provider);
        let opts = StreamOptions::new(model)
            .prompt("Say hello.")
            .max_tokens(helpers::provider_max_tokens(provider, 50))
            .provider(provider)
            .client(&client);

        let stream_result = match stream_generate(opts).await {
            Ok(stream_result) => stream_result,
            Err(error) if should_skip_provider_error(provider, &error) => continue,
            Err(error) => return Err(error),
        };
        let collected = match stream_result.collect().await {
            Ok(collected) => collected,
            Err(error) if should_skip_provider_error(provider, &error) => continue,
            Err(error) => return Err(error),
        };

        assert!(
            !collected.response.text().is_empty(),
            "{provider}: expected non-empty text from streaming"
        );

        let has_start = collected
            .events
            .iter()
            .any(|e| e.event_type == StreamEventType::StreamStart);
        let has_finish = collected
            .events
            .iter()
            .any(|e| e.event_type == StreamEventType::Finish);
        assert!(has_start, "{provider}: expected StreamStart event");
        assert!(has_finish, "{provider}: expected Finish event");
    }

    Ok(())
}

/// §8.9 parity: Single tool call + execution across all providers.
#[tokio::test]
async fn parity_single_tool_call() -> SdkResult<()> {
    let available = available_providers(&["openai", "anthropic", "gemini", "mistral", "deepseek"]);
    if available.is_empty() {
        return Ok(());
    }

    let client = helpers::client_from_env()?;

    for &provider in &available {
        let model = helpers::test_model(provider);
        let tool = Tool::active(helpers::weather_tool_definition(), helpers::weather_handler);

        let opts = GenerateOptions::new(model)
            .prompt("What is the weather in Tokyo? Use the get_weather tool.")
            .tools(vec![tool])
            .max_tool_rounds(3)
            .max_tokens(helpers::provider_max_tokens(provider, 300))
            .provider(provider)
            .client(&client);

        let result = match generate(opts).await {
            Ok(result) => result,
            Err(error) if should_skip_provider_error(provider, &error) => continue,
            Err(error) => return Err(error),
        };

        assert!(
            result.steps.len() >= 2,
            "{provider}: expected at least 2 steps for tool call, got {}",
            result.steps.len()
        );

        assert!(
            !result.text.is_empty(),
            "{provider}: expected non-empty final text after tool call"
        );
    }

    Ok(())
}

/// §8.9 parity: Usage token counts are reported across all providers.
#[tokio::test]
async fn parity_usage_token_counts() -> SdkResult<()> {
    let available = available_providers(&["openai", "anthropic", "gemini", "mistral", "deepseek"]);
    if available.is_empty() {
        return Ok(());
    }

    let client = helpers::client_from_env()?;

    for &provider in &available {
        let model = helpers::test_model(provider);
        let opts = GenerateOptions::new(model)
            .prompt("Say hi.")
            .max_tokens(helpers::provider_max_tokens(provider, 16))
            .provider(provider)
            .client(&client);

        let result = match generate(opts).await {
            Ok(result) => result,
            Err(error) if should_skip_provider_error(provider, &error) => continue,
            Err(error) => return Err(error),
        };

        assert!(
            result.usage.input_tokens > 0,
            "{provider}: expected input_tokens > 0"
        );
        assert!(
            result.usage.output_tokens > 0,
            "{provider}: expected output_tokens > 0"
        );
        assert!(
            result.usage.total_tokens > 0,
            "{provider}: expected total_tokens > 0"
        );
    }

    Ok(())
}

/// §8.9 parity: Error handling — invalid API key (401).
///
/// Creates a provider with an intentionally wrong API key and verifies
/// an authentication error is returned. Each provider block is gated
/// behind the corresponding env-var so this test only hits APIs when we
/// are already in an integration-test environment.
#[tokio::test]
async fn parity_error_invalid_api_key() -> SdkResult<()> {
    use stencila_models3::providers::{
        AnthropicAdapter, DeepSeekAdapter, GeminiAdapter, MistralAdapter, OpenAIAdapter,
    };

    let any_available = helpers::has_provider("openai")
        || helpers::has_provider("anthropic")
        || helpers::has_provider("gemini")
        || helpers::has_provider("mistral")
        || helpers::has_provider("deepseek");
    if !any_available {
        return Ok(());
    }

    // OpenAI
    if helpers::has_provider("openai") {
        let adapter = OpenAIAdapter::new("sk-invalid-key-for-testing")?;
        let client = Client::builder().add_provider(adapter).build()?;
        let opts = GenerateOptions::new("gpt-4.1-mini")
            .prompt("test")
            .max_tokens(helpers::provider_max_tokens("openai", 16))
            .max_retries(0)
            .client(&client);

        let result = generate(opts).await;
        assert!(result.is_err(), "openai: expected error with bad key");
        let err = match result {
            Ok(_) => {
                return Err(SdkError::Configuration {
                    message: "openai: expected error with bad key".to_string(),
                });
            }
            Err(error) => error,
        };
        assert!(
            matches!(
                err,
                SdkError::Authentication { .. } | SdkError::Server { .. }
            ),
            "openai: expected auth/server error, got: {err}"
        );
    }

    // Anthropic
    if helpers::has_provider("anthropic") {
        let adapter = AnthropicAdapter::new("invalid-key-for-testing", None)?;
        let client = Client::builder().add_provider(adapter).build()?;
        let opts = GenerateOptions::new("claude-sonnet-4-5-20250929")
            .prompt("test")
            .max_tokens(helpers::provider_max_tokens("anthropic", 5))
            .max_retries(0)
            .client(&client);

        let result = generate(opts).await;
        assert!(result.is_err(), "anthropic: expected error with bad key");
        let err = match result {
            Ok(_) => {
                return Err(SdkError::Configuration {
                    message: "anthropic: expected error with bad key".to_string(),
                });
            }
            Err(error) => error,
        };
        assert!(
            matches!(
                err,
                SdkError::Authentication { .. } | SdkError::Server { .. }
            ),
            "anthropic: expected auth/server error, got: {err}"
        );
    }

    // Gemini
    if helpers::has_provider("gemini") {
        let adapter = GeminiAdapter::new("invalid-key-for-testing", None)?;
        let client = Client::builder().add_provider(adapter).build()?;
        let opts = GenerateOptions::new("gemini-2.0-flash")
            .prompt("test")
            .max_tokens(helpers::provider_max_tokens("gemini", 5))
            .max_retries(0)
            .client(&client);

        let result = generate(opts).await;
        assert!(result.is_err(), "gemini: expected error with bad key");
    }

    // Mistral
    if helpers::has_provider("mistral") {
        let adapter = MistralAdapter::new("invalid-key-for-testing", None)?;
        let client = Client::builder().add_provider(adapter).build()?;
        let opts = GenerateOptions::new("mistral-small-latest")
            .prompt("test")
            .max_tokens(helpers::provider_max_tokens("mistral", 5))
            .max_retries(0)
            .client(&client);

        let result = generate(opts).await;
        assert!(result.is_err(), "mistral: expected error with bad key");
        let err = match result {
            Ok(_) => {
                return Err(SdkError::Configuration {
                    message: "mistral: expected error with bad key".to_string(),
                });
            }
            Err(error) => error,
        };
        assert!(
            matches!(
                err,
                SdkError::Authentication { .. } | SdkError::Server { .. }
            ),
            "mistral: expected auth/server error, got: {err}"
        );
    }

    // DeepSeek
    if helpers::has_provider("deepseek") {
        let adapter = DeepSeekAdapter::new("invalid-key-for-testing", None)?;
        let client = Client::builder().add_provider(adapter).build()?;
        let opts = GenerateOptions::new("deepseek-chat")
            .prompt("test")
            .max_tokens(helpers::provider_max_tokens("deepseek", 5))
            .max_retries(0)
            .client(&client);

        let result = generate(opts).await;
        assert!(result.is_err(), "deepseek: expected error with bad key");
        let err = match result {
            Ok(_) => {
                return Err(SdkError::Configuration {
                    message: "deepseek: expected error with bad key".to_string(),
                });
            }
            Err(error) => error,
        };
        assert!(
            matches!(
                err,
                SdkError::Authentication { .. } | SdkError::Server { .. }
            ),
            "deepseek: expected auth/server error, got: {err}"
        );
    }

    Ok(())
}

/// §8.9 parity: Streaming with tool calls across providers.
#[tokio::test]
async fn parity_streaming_with_tools() -> SdkResult<()> {
    let available = available_providers(&["openai", "anthropic", "gemini", "mistral", "deepseek"]);
    if available.is_empty() {
        return Ok(());
    }

    let client = helpers::client_from_env()?;

    for &provider in &available {
        let model = helpers::test_model(provider);
        let tool = Tool::active(helpers::weather_tool_definition(), helpers::weather_handler);

        let opts = StreamOptions::new(model)
            .prompt("What is the weather in London? Use the get_weather tool.")
            .tools(vec![tool])
            .max_tool_rounds(3)
            .max_tokens(helpers::provider_max_tokens(provider, 300))
            .provider(provider)
            .client(&client);

        let stream_result = match stream_generate(opts).await {
            Ok(stream_result) => stream_result,
            Err(error) if should_skip_provider_error(provider, &error) => continue,
            Err(error) => return Err(error),
        };
        let collected = match stream_result.collect().await {
            Ok(collected) => collected,
            Err(error) if should_skip_provider_error(provider, &error) => continue,
            Err(error) => return Err(error),
        };

        assert!(
            collected.steps.len() >= 2,
            "{provider}: expected at least 2 steps in streaming tool call, got {}",
            collected.steps.len()
        );

        let has_non_empty_text = !collected.response.text().is_empty();
        let has_tool_calls = collected
            .steps
            .iter()
            .any(|step| !step.tool_calls.is_empty());
        assert!(
            has_non_empty_text || has_tool_calls,
            "{provider}: expected text output or streamed tool-call steps"
        );
    }

    Ok(())
}

/// §8.9 parity: Structured output (generate_object) for providers that support it.
#[tokio::test]
async fn parity_structured_output() -> SdkResult<()> {
    let available = available_providers(&["openai", "gemini", "mistral", "deepseek"]);
    if available.is_empty() {
        return Ok(());
    }

    let client = helpers::client_from_env()?;

    for &provider in &available {
        let model = helpers::test_model(provider);
        let opts = GenerateObjectOptions::new(model, helpers::person_schema_for(provider))
            .prompt("Extract the person: Bob is 25 years old")
            .max_tokens(helpers::provider_max_tokens(provider, 200))
            .provider(provider)
            .client(&client);

        let result = match generate_object(opts).await {
            Ok(result) => result,
            Err(error) if should_skip_provider_error(provider, &error) => continue,
            Err(error) => return Err(error),
        };

        let output = result
            .output
            .as_ref()
            .ok_or_else(|| SdkError::Configuration {
                message: format!("{provider}: expected output to be populated"),
            })?;

        assert_eq!(
            output.get("name").and_then(|v| v.as_str()),
            Some("Bob"),
            "{provider}: expected name=Bob"
        );
        assert_eq!(
            output.get("age").and_then(|v| v.as_i64()),
            Some(25),
            "{provider}: expected age=25"
        );
    }

    Ok(())
}

/// §8.9 parity: Image input (base64) for providers that support vision.
///
/// Uses a 16x16 PNG to verify image content parts are accepted.
#[tokio::test]
async fn parity_image_input_base64() -> SdkResult<()> {
    // DeepSeek does not support vision input.
    let available = available_providers(&["openai", "anthropic", "gemini", "mistral"]);
    if available.is_empty() {
        return Ok(());
    }

    let client = helpers::client_from_env()?;

    // Valid 32x32 RGB PNG fixture.
    let png_bytes = base64::engine::general_purpose::STANDARD
        .decode("iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAAAIGNIUk0AAHomAACAhAAA+gAAAIDoAAB1MAAA6mAAADqYAAAXcJy6UTwAAAAGYktHRAD/AP8A/6C9p5MAAAAHdElNRQfqAgkQFhf/rh3OAAAAKElEQVRIx+3NMQEAAAjDMMC/ZzDBvlRA01vZJvwHAAAAAAAAAAAAbx2jxAE/ehR5RwAAAABJRU5ErkJggg==")
        .map_err(|error| SdkError::Configuration {
            message: format!("failed to decode fixture png: {error}"),
        })?;

    for &provider in &available {
        let model = helpers::vision_test_model(provider);
        let image_part = ContentPart::image_data(png_bytes.clone(), "image/png");
        let text_part = ContentPart::text("What do you see in this image? Answer briefly.");

        let message = Message::new(
            stencila_models3::types::role::Role::User,
            vec![text_part, image_part],
        );

        let opts = GenerateOptions::new(model)
            .messages(vec![message])
            .max_tokens(helpers::provider_max_tokens(provider, 50))
            .provider(provider)
            .client(&client);

        let result = match generate(opts).await {
            Ok(result) => result,
            Err(error) if is_openai_invalid_image_error(&error) => continue,
            Err(error) if should_skip_provider_error(provider, &error) => continue,
            Err(error) => return Err(error),
        };
        assert!(
            !result.text.is_empty(),
            "{provider}: expected non-empty text for image input"
        );
    }

    Ok(())
}
