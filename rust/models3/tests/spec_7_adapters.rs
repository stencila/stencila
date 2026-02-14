//! Spec Section 7 conformance tests.
//!
//! Target areas:
//! - Provider-native request translation
//! - Provider-native response translation
//! - Streaming event translation and SSE handling
//! - OpenAI-ChatCompletions adapter constraints

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use stencila_models3::error::{ProviderDetails, SdkError};
use stencila_models3::http::sse::SseEvent;
use stencila_models3::providers::{
    anthropic, deepseek, gemini, mistral, ollama, openai,
    openai_chat_completions::{self as chat},
};
use stencila_models3::types::{
    content::{ContentPart, ThinkingData, ToolCallData, ToolResultData},
    finish_reason::Reason,
    message::Message,
    request::Request,
    response_format::{ResponseFormat, ResponseFormatType},
    role::Role,
    stream_event::StreamEventType,
    tool::{ToolChoice, ToolDefinition},
};

fn make_headers(pairs: &[(&str, &str)]) -> Result<HeaderMap, Box<dyn std::error::Error>> {
    let mut map = HeaderMap::new();
    for &(name, value) in pairs {
        map.insert(
            HeaderName::from_bytes(name.as_bytes())?,
            HeaderValue::from_str(value)?,
        );
    }
    Ok(map)
}

fn fixture_json(path: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let full_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(path);
    let contents = fs::read_to_string(full_path)?;
    Ok(serde_json::from_str(&contents)?)
}

fn fixture_sse_event(event_type: &str, path: &str) -> Result<SseEvent, Box<dyn std::error::Error>> {
    Ok(SseEvent {
        event_type: event_type.to_string(),
        data: fixture_json(path)?.to_string(),
        retry: None,
    })
}

fn write_temp_image_file(
    extension: &str,
    bytes: &[u8],
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let unique = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let path = std::env::temp_dir().join(format!("models3-spec7-{unique}.{extension}"));
    fs::write(&path, bytes)?;
    Ok(path)
}

#[test]
fn openai_request_translation_includes_provider_options() -> Result<(), Box<dyn std::error::Error>>
{
    let mut request = Request::new(
        "gpt-5.2",
        vec![
            Message::system("system instruction"),
            Message::developer("developer instruction"),
            Message::new(
                Role::User,
                vec![
                    ContentPart::text("hello"),
                    ContentPart::image_url("https://example.com/cat.png"),
                ],
            ),
        ],
    );

    request.reasoning_effort = Some("high".to_string());
    request.max_tokens = Some(512);
    request.stop_sequences = Some(vec!["END".to_string()]);
    request.response_format = Some(ResponseFormat {
        format_type: ResponseFormatType::JsonSchema,
        json_schema: Some(serde_json::json!({
            "type": "object",
            "properties": {"ok": {"type": "boolean"}},
            "required": ["ok"]
        })),
        strict: true,
    });

    request.tools = Some(vec![ToolDefinition {
        name: "get_weather".to_string(),
        description: "Get weather for city".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {"city": {"type": "string"}},
            "required": ["city"]
        }),
        strict: true,
    }]);
    request.tool_choice = Some(ToolChoice::Tool("get_weather".to_string()));

    let mut provider_options = HashMap::new();
    provider_options.insert(
        "openai".to_string(),
        serde_json::json!({
            "built_in_tools": [{"type": "web_search_preview"}],
            "custom_headers": {"x-openai-feature": "enabled"},
            "service_tier": "flex"
        }),
    );
    request.provider_options = Some(provider_options);

    let translated = openai::translate_request::translate_request(&request, false)?;

    assert_eq!(
        translated.body["instructions"],
        serde_json::Value::String("system instruction\n\ndeveloper instruction".to_string())
    );
    assert!(translated.body.get("input").is_some());
    assert_eq!(translated.body["reasoning"]["effort"], "high");
    assert_eq!(translated.body["max_output_tokens"], 512);
    assert_eq!(translated.body["stop"], serde_json::json!(["END"]));
    assert_eq!(translated.body["service_tier"], "flex");
    assert_eq!(translated.body["text"]["format"]["type"], "json_schema");
    assert_eq!(translated.body["text"]["format"]["name"], "response");
    assert_eq!(
        translated.body["text"]["format"]["schema"]["type"],
        "object"
    );
    assert_eq!(translated.body["tool_choice"]["type"], "function");
    assert_eq!(translated.body["tool_choice"]["name"], "get_weather");

    let tools = translated
        .body
        .get("tools")
        .and_then(serde_json::Value::as_array)
        .ok_or("tools should be array")?;
    assert_eq!(tools.len(), 2);
    assert_eq!(tools[0]["type"], "function");
    assert_eq!(tools[0]["name"], "get_weather");
    assert_eq!(tools[0]["parameters"]["type"], "object");

    let header = translated
        .headers
        .get("x-openai-feature")
        .ok_or("missing x-openai-feature header")?
        .to_str()?;
    assert_eq!(header, "enabled");

    Ok(())
}

#[test]
fn openai_request_translation_rejects_non_object_provider_options() {
    let mut request = Request::new("gpt-5.2", vec![Message::user("hello")]);

    let mut provider_options = HashMap::new();
    provider_options.insert("openai".to_string(), serde_json::json!([1, 2, 3]));
    request.provider_options = Some(provider_options);

    let result = openai::translate_request::translate_request(&request, false);
    assert!(matches!(result, Err(SdkError::InvalidRequest { .. })));
}

#[test]
fn openai_request_translation_local_image_path_becomes_data_uri()
-> Result<(), Box<dyn std::error::Error>> {
    let path = write_temp_image_file("png", &[137, 80, 78, 71, 13, 10, 26, 10])?;

    let request = Request::new(
        "gpt-5.2",
        vec![Message::new(
            Role::User,
            vec![ContentPart::image_url(path.to_string_lossy())],
        )],
    );

    let translated = openai::translate_request::translate_request(&request, false)?;
    let _ = fs::remove_file(&path);

    let input = translated
        .body
        .get("input")
        .and_then(serde_json::Value::as_array)
        .ok_or("missing input array")?;
    let user_message = input
        .iter()
        .find(|item| item.get("type").and_then(serde_json::Value::as_str) == Some("message"))
        .ok_or("missing message input item")?;
    let content = user_message
        .get("content")
        .and_then(serde_json::Value::as_array)
        .ok_or("missing message content")?;
    let image_part = content
        .iter()
        .find(|part| part.get("type").and_then(serde_json::Value::as_str) == Some("input_image"))
        .ok_or("missing input_image part")?;
    let image_url = image_part
        .get("image_url")
        .and_then(serde_json::Value::as_str)
        .ok_or("input_image.image_url should be string")?;

    assert!(image_url.starts_with("data:image/png;base64,"));
    Ok(())
}

#[test]
fn openai_request_translation_missing_local_image_path_errors() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let missing_path = std::env::temp_dir().join(format!("models3-missing-{unique}.png"));
    let _ = fs::remove_file(&missing_path);

    let request = Request::new(
        "gpt-5.2",
        vec![Message::new(
            Role::User,
            vec![ContentPart::image_url(missing_path.to_string_lossy())],
        )],
    );

    let result = openai::translate_request::translate_request(&request, false);
    assert!(matches!(result, Err(SdkError::InvalidRequest { .. })));
}

#[test]
fn openai_request_translation_portably_maps_thinking_blocks()
-> Result<(), Box<dyn std::error::Error>> {
    let request = Request::new(
        "gpt-5.2",
        vec![Message::new(
            Role::Assistant,
            vec![
                ContentPart::Thinking {
                    thinking: ThinkingData {
                        text: "Reasoning context".to_string(),
                        signature: Some("sig_should_not_be_sent".to_string()),
                        redacted: false,
                    },
                },
                ContentPart::RedactedThinking {
                    thinking: ThinkingData {
                        text: "opaque".to_string(),
                        signature: None,
                        redacted: true,
                    },
                },
                ContentPart::text("Visible answer"),
            ],
        )],
    );

    let translated = openai::translate_request::translate_request(&request, false)?;
    let input = translated
        .body
        .get("input")
        .and_then(serde_json::Value::as_array)
        .ok_or("missing input array")?;
    let assistant_message = input
        .iter()
        .find(|item| {
            item.get("type").and_then(serde_json::Value::as_str) == Some("message")
                && item.get("role").and_then(serde_json::Value::as_str) == Some("assistant")
        })
        .ok_or("missing assistant message")?;
    let content = assistant_message
        .get("content")
        .and_then(serde_json::Value::as_array)
        .ok_or("assistant content should be array")?;

    assert_eq!(content.len(), 2, "redacted thinking should be omitted");
    assert_eq!(content[0]["type"], "output_text");
    assert_eq!(content[0]["text"], "Reasoning context");
    assert_eq!(content[1]["type"], "output_text");
    assert_eq!(content[1]["text"], "Visible answer");

    Ok(())
}

#[test]
fn openai_request_translation_uses_call_id_for_tool_history()
-> Result<(), Box<dyn std::error::Error>> {
    let request = Request::new(
        "gpt-5.2",
        vec![
            Message::user("What is the weather in Paris?"),
            Message::new(
                Role::Assistant,
                vec![ContentPart::tool_call(
                    "call_1",
                    "get_weather",
                    serde_json::json!({"city": "Paris"}),
                )],
            ),
            Message::tool_result("call_1", serde_json::json!({"temperature": "72F"}), false),
        ],
    );

    let translated = openai::translate_request::translate_request(&request, false)?;
    let input = translated
        .body
        .get("input")
        .and_then(serde_json::Value::as_array)
        .ok_or("input should be array")?;

    let tool_call = input
        .iter()
        .find(|item| item.get("type").and_then(serde_json::Value::as_str) == Some("function_call"))
        .ok_or("missing function_call item")?;

    assert_eq!(tool_call["call_id"], "call_1");
    assert!(tool_call.get("id").is_none());

    Ok(())
}

#[test]
fn openai_request_translation_serializes_tool_result_output()
-> Result<(), Box<dyn std::error::Error>> {
    let request = Request::new(
        "gpt-5.2",
        vec![Message::tool_result(
            "call_1",
            serde_json::json!({"temperature": "72F"}),
            false,
        )],
    );

    let translated = openai::translate_request::translate_request(&request, false)?;
    let input = translated
        .body
        .get("input")
        .and_then(serde_json::Value::as_array)
        .ok_or("input should be array")?;

    let tool_output = input
        .iter()
        .find(|item| {
            item.get("type").and_then(serde_json::Value::as_str) == Some("function_call_output")
        })
        .ok_or("missing function_call_output item")?;

    let output = tool_output
        .get("output")
        .and_then(serde_json::Value::as_str)
        .ok_or("output should be string")?;
    assert_eq!(output, r#"{"temperature":"72F"}"#);

    Ok(())
}

/// Assistant messages with both text and tool-call parts must preserve
/// content ordering: the text `message` item appears before the `function_call`.
#[test]
fn openai_request_translation_preserves_text_tool_call_order()
-> Result<(), Box<dyn std::error::Error>> {
    let request = Request::new(
        "gpt-5.2",
        vec![Message::new(
            Role::Assistant,
            vec![
                ContentPart::text("I'll check the weather for you."),
                ContentPart::tool_call(
                    "call_1",
                    "get_weather",
                    serde_json::json!({"city": "Paris"}),
                ),
            ],
        )],
    );

    let translated = openai::translate_request::translate_request(&request, false)?;
    let input = translated
        .body
        .get("input")
        .and_then(serde_json::Value::as_array)
        .ok_or("input should be array")?;

    assert_eq!(
        input.len(),
        2,
        "expected message + function_call, got {input:?}"
    );

    // First item: the text message
    assert_eq!(
        input[0].get("type").and_then(serde_json::Value::as_str),
        Some("message"),
        "first item should be a message"
    );

    // Second item: the function_call
    assert_eq!(
        input[1].get("type").and_then(serde_json::Value::as_str),
        Some("function_call"),
        "second item should be a function_call"
    );

    Ok(())
}

/// Text → tool_call → more text must produce three items in order:
/// message, function_call, message.
#[test]
fn openai_request_translation_preserves_text_tool_call_text_order()
-> Result<(), Box<dyn std::error::Error>> {
    let request = Request::new(
        "gpt-5.2",
        vec![Message::new(
            Role::Assistant,
            vec![
                ContentPart::text("Let me look that up."),
                ContentPart::tool_call("call_1", "search", serde_json::json!({"q": "rust"})),
                ContentPart::text("Here are the results."),
            ],
        )],
    );

    let translated = openai::translate_request::translate_request(&request, false)?;
    let input = translated
        .body
        .get("input")
        .and_then(serde_json::Value::as_array)
        .ok_or("input should be array")?;

    assert_eq!(
        input.len(),
        3,
        "expected message + function_call + message, got {input:?}"
    );

    let types: Vec<_> = input
        .iter()
        .filter_map(|item| item.get("type").and_then(serde_json::Value::as_str))
        .collect();
    assert_eq!(types, vec!["message", "function_call", "message"]);

    Ok(())
}

/// Text → tool_result must produce message before function_call_output.
#[test]
fn openai_request_translation_preserves_text_tool_result_order()
-> Result<(), Box<dyn std::error::Error>> {
    let request = Request::new(
        "gpt-5.2",
        vec![Message::new(
            Role::User,
            vec![
                ContentPart::text("Here is the tool output."),
                ContentPart::ToolResult {
                    tool_result: ToolResultData {
                        tool_call_id: "call_1".into(),
                        content: serde_json::json!({"temp": "72F"}),
                        is_error: false,
                        image_data: None,
                        image_media_type: None,
                    },
                },
            ],
        )],
    );

    let translated = openai::translate_request::translate_request(&request, false)?;
    let input = translated
        .body
        .get("input")
        .and_then(serde_json::Value::as_array)
        .ok_or("input should be array")?;

    assert_eq!(
        input.len(),
        2,
        "expected message + function_call_output, got {input:?}"
    );

    let types: Vec<_> = input
        .iter()
        .filter_map(|item| item.get("type").and_then(serde_json::Value::as_str))
        .collect();
    assert_eq!(types, vec!["message", "function_call_output"]);

    Ok(())
}

#[test]
fn openai_response_translation_maps_usage_fields() -> Result<(), Box<dyn std::error::Error>> {
    let raw_response = fixture_json("openai/response_translation_maps_usage_fields.json")?;

    let headers = make_headers(&[
        ("x-ratelimit-remaining-requests", "9"),
        ("x-ratelimit-limit-requests", "10"),
    ])?;

    let response = openai::translate_response::translate_response(raw_response, Some(&headers))?;

    assert_eq!(response.id, "resp_123");
    assert_eq!(response.model, "gpt-5.2");
    assert_eq!(response.provider, "openai");
    assert_eq!(response.text(), "Result: sunny");
    assert_eq!(response.tool_calls().len(), 1);
    assert_eq!(response.finish_reason.reason, Reason::ToolCalls);
    assert_eq!(response.usage.input_tokens, 20);
    assert_eq!(response.usage.output_tokens, 30);
    assert_eq!(response.usage.total_tokens, 50);
    assert_eq!(response.usage.reasoning_tokens, Some(11));
    assert_eq!(response.usage.cache_read_tokens, Some(7));
    assert_eq!(
        response
            .rate_limit
            .as_ref()
            .and_then(|rl| rl.requests_remaining),
        Some(9)
    );

    Ok(())
}

#[test]
fn openai_error_translation_refines_quota_and_provider() {
    let err = SdkError::RateLimit {
        message: "You exceeded your current quota".to_string(),
        details: ProviderDetails {
            provider: None,
            status_code: Some(429),
            error_code: Some("insufficient_quota".to_string()),
            retryable: true,
            retry_after: Some(2.0),
            raw: Some(serde_json::json!({"error": {"code": "insufficient_quota"}})),
        },
    };

    let translated = openai::translate_error::translate_error(err);
    assert!(matches!(translated, SdkError::QuotaExceeded { .. }));

    if let SdkError::QuotaExceeded { details, .. } = translated {
        assert_eq!(details.provider.as_deref(), Some("openai"));
        assert_eq!(details.error_code.as_deref(), Some("insufficient_quota"));
    }
}

#[test]
fn openai_stream_translation_maps_text_and_finish() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = openai::translate_stream::OpenAIStreamState::default();

    let delta_event = fixture_sse_event(
        "response.output_text.delta",
        "openai/stream_translation_text_and_finish_delta.json",
    )?;

    let events = openai::translate_stream::translate_sse_event(&delta_event, &mut state)?;
    assert_eq!(events[0].event_type, StreamEventType::StreamStart);
    assert_eq!(events[1].event_type, StreamEventType::TextStart);
    assert_eq!(events[2].event_type, StreamEventType::TextDelta);
    assert_eq!(events[2].delta.as_deref(), Some("Hel"));

    let end_event = fixture_sse_event(
        "response.output_item.done",
        "openai/stream_translation_text_and_finish_item_done.json",
    )?;

    let end_events = openai::translate_stream::translate_sse_event(&end_event, &mut state)?;
    assert_eq!(end_events[0].event_type, StreamEventType::TextEnd);

    let completed_event = fixture_sse_event(
        "response.completed",
        "openai/stream_translation_text_and_finish_completed.json",
    )?;

    let done_events = openai::translate_stream::translate_sse_event(&completed_event, &mut state)?;
    let finish = done_events
        .iter()
        .find(|ev| ev.event_type == StreamEventType::Finish)
        .ok_or("missing finish event")?;

    assert_eq!(
        finish.finish_reason.as_ref().map(|f| f.reason),
        Some(Reason::Stop)
    );
    assert_eq!(
        finish.usage.as_ref().and_then(|u| u.reasoning_tokens),
        Some(1)
    );

    Ok(())
}

#[test]
fn chat_request_translation_uses_messages_shape() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new(
        "llama-3.1-70b",
        vec![
            Message::system("sys"),
            Message::user("hi"),
            Message::new(
                Role::Assistant,
                vec![ContentPart::ToolCall {
                    tool_call: ToolCallData {
                        id: "call_1".to_string(),
                        name: "get_weather".to_string(),
                        arguments: serde_json::json!({"city": "Paris"}),
                        call_type: "function".to_string(),
                    },
                }],
            ),
            Message::tool_result("call_1", serde_json::json!({"temp": 22}), false),
        ],
    );
    request.tool_choice = Some(ToolChoice::Required);

    let translated = chat::translate_request::translate_request(&request, true)?;

    assert!(translated.body.get("messages").is_some());
    assert!(translated.body.get("input").is_none());
    assert!(translated.body.get("instructions").is_none());
    assert_eq!(translated.body["stream"], true);
    assert_eq!(translated.body["tool_choice"], "required");

    Ok(())
}

#[test]
fn chat_request_translation_portably_maps_thinking_blocks() -> Result<(), Box<dyn std::error::Error>>
{
    let request = Request::new(
        "llama-3.1-70b",
        vec![Message::new(
            Role::Assistant,
            vec![
                ContentPart::Thinking {
                    thinking: ThinkingData {
                        text: "Prior reasoning ".to_string(),
                        signature: Some("sig_abc".to_string()),
                        redacted: false,
                    },
                },
                ContentPart::RedactedThinking {
                    thinking: ThinkingData {
                        text: "opaque".to_string(),
                        signature: None,
                        redacted: true,
                    },
                },
                ContentPart::text("final answer"),
            ],
        )],
    );

    let translated = chat::translate_request::translate_request(&request, false)?;
    let messages = translated
        .body
        .get("messages")
        .and_then(serde_json::Value::as_array)
        .ok_or("missing messages")?;
    let assistant = messages
        .iter()
        .find(|m| m.get("role").and_then(serde_json::Value::as_str) == Some("assistant"))
        .ok_or("missing assistant message")?;
    let content = assistant
        .get("content")
        .and_then(serde_json::Value::as_str)
        .ok_or("assistant content should be text")?;

    assert_eq!(content, "Prior reasoning final answer");
    Ok(())
}

#[test]
fn chat_request_translation_rejects_responses_api_builtin_tools() {
    let mut request = Request::new("llama", vec![Message::user("hello")]);

    let mut provider_options = HashMap::new();
    provider_options.insert(
        "openai".to_string(),
        serde_json::json!({
            "built_in_tools": [{"type": "web_search_preview"}]
        }),
    );
    request.provider_options = Some(provider_options);

    let result = chat::translate_request::translate_request(&request, false);
    assert!(matches!(result, Err(SdkError::InvalidRequest { .. })));
}

#[test]
fn chat_request_translation_rejects_builtin_tools_via_adapter_options() {
    // Built-in tools passed through the adapter-specific namespace should also
    // be rejected, not just provider_options.openai.
    let mut request = Request::new("llama", vec![Message::user("hello")]);

    let mut provider_options = HashMap::new();
    provider_options.insert(
        "openai_chat_completions".to_string(),
        serde_json::json!({
            "built_in_tools": [{"type": "web_search_preview"}]
        }),
    );
    request.provider_options = Some(provider_options);

    let result = chat::translate_request::translate_request(&request, false);
    assert!(matches!(result, Err(SdkError::InvalidRequest { .. })));
}

#[test]
fn chat_response_translation_omits_reasoning_tokens() -> Result<(), Box<dyn std::error::Error>> {
    let raw_response =
        fixture_json("openai_chat/response_translation_omits_reasoning_tokens.json")?;

    let response = chat::translate_response::translate_response(raw_response, None)?;

    assert_eq!(response.provider, "openai_chat_completions");
    assert_eq!(response.text(), "Hello");
    assert_eq!(response.finish_reason.reason, Reason::Stop);
    assert_eq!(response.usage.input_tokens, 10);
    assert_eq!(response.usage.output_tokens, 6);
    assert_eq!(response.usage.total_tokens, 16);
    assert_eq!(response.usage.cache_read_tokens, Some(3));
    assert_eq!(response.usage.reasoning_tokens, None);

    Ok(())
}

#[test]
fn chat_stream_translation_maps_tool_calls_and_finish() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = chat::translate_stream::OpenAIChatCompletionsStreamState::default();

    let usage_event = fixture_sse_event(
        "message",
        "openai_chat/stream_translation_maps_tool_calls_and_finish_usage.json",
    )?;

    let usage_events = chat::translate_stream::translate_sse_event(&usage_event, &mut state)?;
    assert_eq!(usage_events[0].event_type, StreamEventType::StreamStart);

    let tool_delta_1 = fixture_sse_event(
        "message",
        "openai_chat/stream_translation_maps_tool_calls_and_finish_tool_delta_1.json",
    )?;

    let tool_events_1 = chat::translate_stream::translate_sse_event(&tool_delta_1, &mut state)?;
    assert!(
        tool_events_1
            .iter()
            .any(|event| event.event_type == StreamEventType::ToolCallStart)
    );
    assert!(
        tool_events_1
            .iter()
            .any(|event| event.event_type == StreamEventType::ToolCallDelta)
    );

    let tool_delta_2 = fixture_sse_event(
        "message",
        "openai_chat/stream_translation_maps_tool_calls_and_finish_tool_delta_2.json",
    )?;

    let _tool_events_2 = chat::translate_stream::translate_sse_event(&tool_delta_2, &mut state)?;

    let finish_event = fixture_sse_event(
        "message",
        "openai_chat/stream_translation_maps_tool_calls_and_finish_finish.json",
    )?;

    let finish_events = chat::translate_stream::translate_sse_event(&finish_event, &mut state)?;

    assert!(
        finish_events
            .iter()
            .any(|event| event.event_type == StreamEventType::ToolCallEnd)
    );
    let finish = finish_events
        .iter()
        .find(|event| event.event_type == StreamEventType::Finish)
        .ok_or("missing finish event")?;
    assert_eq!(
        finish.finish_reason.as_ref().map(|reason| reason.reason),
        Some(Reason::ToolCalls)
    );
    assert_eq!(
        finish.usage.as_ref().map(|usage| usage.total_tokens),
        Some(7)
    );

    Ok(())
}

#[test]
fn chat_stream_translation_emits_error_for_error_payload() -> Result<(), Box<dyn std::error::Error>>
{
    let mut state = chat::translate_stream::OpenAIChatCompletionsStreamState::default();

    // An error chunk with no choices should produce an Error event, not a ProviderEvent
    let error_event = fixture_sse_event(
        "message",
        "openai_chat/stream_translation_emits_error_for_error_payload.json",
    )?;

    let events = chat::translate_stream::translate_sse_event(&error_event, &mut state)?;
    let error = events
        .iter()
        .find(|e| e.event_type == StreamEventType::Error)
        .ok_or("expected Error event for error payload")?;
    assert!(error.error.is_some());

    Ok(())
}

#[test]
fn chat_error_translation_refines_not_found_from_message() {
    let err = SdkError::Server {
        message: "Model not found".to_string(),
        details: ProviderDetails {
            provider: None,
            status_code: Some(500),
            error_code: None,
            retryable: true,
            retry_after: None,
            raw: None,
        },
    };

    let translated = chat::translate_error::translate_error(err);
    assert!(matches!(translated, SdkError::NotFound { .. }));

    if let SdkError::NotFound { details, .. } = translated {
        assert_eq!(details.provider.as_deref(), Some("openai_chat_completions"));
    }
}

// ──────────────── Anthropic adapter tests ────────────────

#[test]
fn anthropic_request_translation_system_and_tools() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new(
        "claude-sonnet-4-5-20250929",
        vec![
            Message::system("You are helpful."),
            Message::user("What's the weather?"),
        ],
    );

    request.max_tokens = Some(1024);
    request.temperature = Some(0.7);
    request.tools = Some(vec![ToolDefinition {
        name: "get_weather".to_string(),
        description: "Get weather for city".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {"city": {"type": "string"}},
            "required": ["city"]
        }),
        strict: false,
    }]);
    request.tool_choice = Some(ToolChoice::Auto);

    let translated = anthropic::translate_request::translate_request(&request, false, None)?;

    assert_eq!(translated.body["model"], "claude-sonnet-4-5-20250929");
    assert_eq!(translated.body["max_tokens"], 1024);
    assert_eq!(translated.body["temperature"], 0.7);

    // System messages become top-level system blocks
    let system = translated
        .body
        .get("system")
        .and_then(serde_json::Value::as_array)
        .ok_or("system should be array")?;
    assert_eq!(system.len(), 1);
    assert_eq!(system[0]["text"], "You are helpful.");

    // Tools present
    let tools = translated
        .body
        .get("tools")
        .and_then(serde_json::Value::as_array)
        .ok_or("tools should be array")?;
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "get_weather");

    assert_eq!(
        translated.body["tool_choice"],
        serde_json::json!({"type": "auto"})
    );

    Ok(())
}

#[test]
fn anthropic_request_translation_local_image_path_uses_base64_source()
-> Result<(), Box<dyn std::error::Error>> {
    let path = write_temp_image_file("jpg", &[255, 216, 255, 224, 0, 16, 74, 70, 73, 70])?;

    let request = Request::new(
        "claude-sonnet-4-5-20250929",
        vec![Message::new(
            Role::User,
            vec![ContentPart::image_url(path.to_string_lossy())],
        )],
    );

    let translated = anthropic::translate_request::translate_request(&request, false, None)?;
    let _ = fs::remove_file(&path);

    let messages = translated
        .body
        .get("messages")
        .and_then(serde_json::Value::as_array)
        .ok_or("messages should be array")?;
    let first_message = messages.first().ok_or("missing first message")?;
    let content = first_message
        .get("content")
        .and_then(serde_json::Value::as_array)
        .ok_or("missing content array")?;
    let image = content.first().ok_or("missing image block")?;

    assert_eq!(image["type"], "image");
    assert_eq!(image["source"]["type"], "base64");
    assert_eq!(image["source"]["media_type"], "image/jpeg");
    let encoded = image["source"]["data"]
        .as_str()
        .ok_or("missing base64 image data")?;
    let decoded = base64::engine::general_purpose::STANDARD.decode(encoded)?;
    assert_eq!(decoded, vec![255, 216, 255, 224, 0, 16, 74, 70, 73, 70]);

    Ok(())
}

#[test]
fn anthropic_response_translation_content_blocks() -> Result<(), Box<dyn std::error::Error>> {
    let raw_response = fixture_json("anthropic/response_translation_content_blocks.json")?;

    let response = anthropic::translate_response::translate_response(raw_response, None)?;

    assert_eq!(response.id, "msg_123");
    assert_eq!(response.model, "claude-sonnet-4-5-20250929");
    assert_eq!(response.provider, "anthropic");
    assert_eq!(response.text(), "Here's the weather:");
    assert_eq!(response.tool_calls().len(), 1);
    assert_eq!(response.finish_reason.reason, Reason::ToolCalls);
    assert_eq!(response.usage.input_tokens, 15);
    assert_eq!(response.usage.output_tokens, 25);
    assert_eq!(response.usage.cache_read_tokens, Some(5));
    assert_eq!(response.usage.cache_write_tokens, Some(3));

    Ok(())
}

#[test]
fn anthropic_response_translation_thinking() -> Result<(), Box<dyn std::error::Error>> {
    let raw_response = fixture_json("anthropic/response_translation_thinking.json")?;

    let response = anthropic::translate_response::translate_response(raw_response, None)?;

    assert_eq!(
        response.reasoning(),
        Some("Let me think about this...".to_string())
    );
    assert_eq!(response.text(), "The answer is 42");
    assert_eq!(response.finish_reason.reason, Reason::Stop);
    assert_eq!(response.usage.reasoning_tokens, Some(5));

    Ok(())
}

#[test]
fn anthropic_stream_translation_full_sequence() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = anthropic::translate_stream::AnthropicStreamState::default();

    let msg_start = fixture_sse_event(
        "message_start",
        "anthropic/stream_translation_full_sequence_message_start.json",
    )?;
    let events = anthropic::translate_stream::translate_sse_event(&msg_start, &mut state)?;
    assert_eq!(events[0].event_type, StreamEventType::StreamStart);

    let block_start = fixture_sse_event(
        "content_block_start",
        "anthropic/stream_translation_full_sequence_content_block_start.json",
    )?;
    let events = anthropic::translate_stream::translate_sse_event(&block_start, &mut state)?;
    assert_eq!(events[0].event_type, StreamEventType::TextStart);

    let block_delta = fixture_sse_event(
        "content_block_delta",
        "anthropic/stream_translation_full_sequence_content_block_delta.json",
    )?;
    let events = anthropic::translate_stream::translate_sse_event(&block_delta, &mut state)?;
    assert_eq!(events[0].event_type, StreamEventType::TextDelta);
    assert_eq!(events[0].delta.as_deref(), Some("Hello!"));

    let block_stop = fixture_sse_event(
        "content_block_stop",
        "anthropic/stream_translation_full_sequence_content_block_stop.json",
    )?;
    let events = anthropic::translate_stream::translate_sse_event(&block_stop, &mut state)?;
    assert_eq!(events[0].event_type, StreamEventType::TextEnd);

    let msg_delta = fixture_sse_event(
        "message_delta",
        "anthropic/stream_translation_full_sequence_message_delta.json",
    )?;
    let _events = anthropic::translate_stream::translate_sse_event(&msg_delta, &mut state)?;

    let msg_stop = fixture_sse_event(
        "message_stop",
        "anthropic/stream_translation_full_sequence_message_stop.json",
    )?;
    let events = anthropic::translate_stream::translate_sse_event(&msg_stop, &mut state)?;
    let finish = events
        .iter()
        .find(|ev| ev.event_type == StreamEventType::Finish)
        .ok_or("missing finish event")?;

    assert_eq!(
        finish.finish_reason.as_ref().map(|f| f.reason),
        Some(Reason::Stop)
    );
    assert!(finish.response.is_some());
    let resp = finish.response.as_ref().ok_or("missing response")?;
    assert_eq!(resp.id, "msg_stream_1");
    assert_eq!(resp.text(), "Hello!");

    Ok(())
}

#[test]
fn anthropic_stream_translation_preserves_thinking_signature()
-> Result<(), Box<dyn std::error::Error>> {
    let mut state = anthropic::translate_stream::AnthropicStreamState::default();

    let msg_start = fixture_sse_event(
        "message_start",
        "anthropic/stream_translation_preserves_thinking_signature_message_start.json",
    )?;
    anthropic::translate_stream::translate_sse_event(&msg_start, &mut state)?;

    let block_start = fixture_sse_event(
        "content_block_start",
        "anthropic/stream_translation_preserves_thinking_signature_content_block_start_thinking.json",
    )?;
    anthropic::translate_stream::translate_sse_event(&block_start, &mut state)?;

    let thinking_delta = fixture_sse_event(
        "content_block_delta",
        "anthropic/stream_translation_preserves_thinking_signature_content_block_delta_thinking.json",
    )?;
    anthropic::translate_stream::translate_sse_event(&thinking_delta, &mut state)?;

    let sig_delta = fixture_sse_event(
        "content_block_delta",
        "anthropic/stream_translation_preserves_thinking_signature_content_block_delta_signature.json",
    )?;
    anthropic::translate_stream::translate_sse_event(&sig_delta, &mut state)?;

    let block_stop = fixture_sse_event(
        "content_block_stop",
        "anthropic/stream_translation_preserves_thinking_signature_content_block_stop_thinking.json",
    )?;
    anthropic::translate_stream::translate_sse_event(&block_stop, &mut state)?;

    let text_start = fixture_sse_event(
        "content_block_start",
        "anthropic/stream_translation_preserves_thinking_signature_content_block_start_text.json",
    )?;
    anthropic::translate_stream::translate_sse_event(&text_start, &mut state)?;

    let text_delta = fixture_sse_event(
        "content_block_delta",
        "anthropic/stream_translation_preserves_thinking_signature_content_block_delta_text.json",
    )?;
    anthropic::translate_stream::translate_sse_event(&text_delta, &mut state)?;

    let text_stop = fixture_sse_event(
        "content_block_stop",
        "anthropic/stream_translation_preserves_thinking_signature_content_block_stop_text.json",
    )?;
    anthropic::translate_stream::translate_sse_event(&text_stop, &mut state)?;

    let msg_delta = fixture_sse_event(
        "message_delta",
        "anthropic/stream_translation_preserves_thinking_signature_message_delta.json",
    )?;
    anthropic::translate_stream::translate_sse_event(&msg_delta, &mut state)?;

    let msg_stop = fixture_sse_event(
        "message_stop",
        "anthropic/stream_translation_preserves_thinking_signature_message_stop.json",
    )?;
    let events = anthropic::translate_stream::translate_sse_event(&msg_stop, &mut state)?;

    let finish = events
        .iter()
        .find(|ev| ev.event_type == StreamEventType::Finish)
        .ok_or("missing finish event")?;
    let resp = finish.response.as_ref().ok_or("missing response")?;

    // Verify thinking block has the signature preserved
    assert_eq!(resp.reasoning(), Some("Let me reason...".to_string()));
    let thinking_part = resp
        .message
        .content
        .iter()
        .find(|p| matches!(p, ContentPart::Thinking { .. }))
        .ok_or("missing thinking content part")?;
    if let ContentPart::Thinking { thinking } = thinking_part {
        assert_eq!(thinking.signature.as_deref(), Some("sig_abc123"));
    }

    assert_eq!(resp.text(), "The answer.");
    assert_eq!(
        finish.usage.as_ref().and_then(|u| u.reasoning_tokens),
        Some(3)
    );

    Ok(())
}

#[test]
fn anthropic_error_translation_refines_billing_to_quota() {
    let err = SdkError::RateLimit {
        message: "Your credit balance is too low".to_string(),
        details: ProviderDetails {
            provider: None,
            status_code: Some(429),
            error_code: Some("billing_error".to_string()),
            retryable: true,
            retry_after: None,
            raw: None,
        },
    };

    let translated = anthropic::translate_error::translate_error(err);
    assert!(matches!(translated, SdkError::QuotaExceeded { .. }));

    if let SdkError::QuotaExceeded { details, .. } = translated {
        assert_eq!(details.provider.as_deref(), Some("anthropic"));
    }
}

#[test]
fn anthropic_auto_cache_injects_on_system_tools_and_conversation_prefix()
-> Result<(), Box<dyn std::error::Error>> {
    // Multi-turn conversation: system + 3 turns (user/assistant/user).
    // auto_cache (default true) should inject cache_control on:
    // 1. Last system block
    // 2. Last tool definition
    // 3. Second-to-last message (the conversation prefix boundary)
    let request = Request::new(
        "claude-sonnet-4-5-20250929",
        vec![
            Message::system("You are helpful."),
            Message::user("What is Rust?"),
            Message::assistant("Rust is a systems language."),
            Message::user("Tell me more."),
        ],
    );

    let translated = anthropic::translate_request::translate_request(&request, false, None)?;

    // System block should have cache_control
    let system = translated
        .body
        .get("system")
        .and_then(serde_json::Value::as_array)
        .ok_or("system should be array")?;
    let last_system = system.last().ok_or("system should not be empty")?;
    assert!(
        last_system.get("cache_control").is_some(),
        "last system block should have cache_control"
    );

    // Second-to-last message (assistant turn) should have cache_control
    // on its last content block
    let messages = translated
        .body
        .get("messages")
        .and_then(serde_json::Value::as_array)
        .ok_or("messages should be array")?;
    assert!(messages.len() >= 2, "need at least 2 messages");
    let prefix_msg = &messages[messages.len() - 2];
    let prefix_content = prefix_msg
        .get("content")
        .and_then(serde_json::Value::as_array)
        .ok_or("prefix message should have content array")?;
    let last_block = prefix_content
        .last()
        .ok_or("prefix content should not be empty")?;
    assert!(
        last_block.get("cache_control").is_some(),
        "last content block in conversation prefix message should have cache_control"
    );

    // The final message should NOT have cache_control (it's the new turn)
    let final_msg = messages.last().ok_or("messages should not be empty")?;
    let final_content = final_msg
        .get("content")
        .and_then(serde_json::Value::as_array)
        .ok_or("final message should have content array")?;
    let final_block = final_content
        .last()
        .ok_or("final content should not be empty")?;
    assert!(
        final_block.get("cache_control").is_none(),
        "final message should not have cache_control"
    );

    Ok(())
}

#[test]
fn anthropic_request_translation_provider_options() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new(
        "claude-sonnet-4-5-20250929",
        vec![Message::system("You are helpful."), Message::user("Hello")],
    );

    let mut provider_options = HashMap::new();
    provider_options.insert(
        "anthropic".to_string(),
        serde_json::json!({
            "beta_headers": ["interleaved-thinking-2025-05-14", "token-efficient-tools-2025-02-19"],
            "auto_cache": false,
            "top_k": 40
        }),
    );
    request.provider_options = Some(provider_options);

    let translated = anthropic::translate_request::translate_request(&request, false, None)?;

    // beta_headers joined into anthropic-beta header
    let beta = translated
        .headers
        .get("anthropic-beta")
        .ok_or("missing anthropic-beta header")?
        .to_str()?;
    assert!(beta.contains("interleaved-thinking-2025-05-14"));
    assert!(beta.contains("token-efficient-tools-2025-02-19"));

    // auto_cache=false should suppress cache_control injection and the
    // prompt-caching beta header
    assert!(!beta.contains("prompt-caching"));

    // System blocks should NOT have cache_control when auto_cache is false
    let system = translated
        .body
        .get("system")
        .and_then(serde_json::Value::as_array)
        .ok_or("system should be array")?;
    let last_system = system.last().ok_or("system should not be empty")?;
    assert!(
        last_system.get("cache_control").is_none(),
        "cache_control should not be injected when auto_cache is false"
    );

    // Remaining options passed through to body
    assert_eq!(translated.body["top_k"], 40);

    Ok(())
}

#[test]
fn anthropic_request_translation_beta_features_alias() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new("claude-sonnet-4-5-20250929", vec![Message::user("Hello")]);

    let mut provider_options = HashMap::new();
    provider_options.insert(
        "anthropic".to_string(),
        serde_json::json!({
            "beta_features": "max-tokens-3-5-sonnet-2025-04-14,interleaved-thinking-2025-05-14"
        }),
    );
    request.provider_options = Some(provider_options);

    let translated = anthropic::translate_request::translate_request(&request, false, None)?;

    let beta = translated
        .headers
        .get("anthropic-beta")
        .ok_or("missing anthropic-beta header")?
        .to_str()?;
    assert!(beta.contains("max-tokens-3-5-sonnet-2025-04-14"));
    assert!(beta.contains("interleaved-thinking-2025-05-14"));

    Ok(())
}

// ──────────────── Gemini adapter tests ────────────────

#[test]
fn gemini_request_translation_system_and_tools() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new(
        "gemini-2.5-pro",
        vec![
            Message::system("You are a helpful assistant."),
            Message::user("Tell me about Rust"),
        ],
    );

    request.temperature = Some(0.5);
    request.max_tokens = Some(2048);
    request.tools = Some(vec![ToolDefinition {
        name: "search".to_string(),
        description: "Search the web".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {"query": {"type": "string"}},
            "required": ["query"]
        }),
        strict: false,
    }]);
    request.tool_choice = Some(ToolChoice::Auto);

    let translated = gemini::translate_request::translate_request(&request)?;

    // System instruction is separate
    let sys = translated
        .get("systemInstruction")
        .ok_or("missing systemInstruction")?;
    let sys_text = sys
        .pointer("/parts/0/text")
        .and_then(serde_json::Value::as_str)
        .ok_or("missing system text")?;
    assert_eq!(sys_text, "You are a helpful assistant.");

    // Contents has the user message
    let contents = translated
        .get("contents")
        .and_then(serde_json::Value::as_array)
        .ok_or("contents should be array")?;
    assert!(!contents.is_empty());

    // Generation config
    let gen_config = translated
        .get("generationConfig")
        .ok_or("missing generationConfig")?;
    assert_eq!(gen_config["temperature"], 0.5);
    assert_eq!(gen_config["maxOutputTokens"], 2048);

    // Tools
    let tools = translated
        .get("tools")
        .and_then(serde_json::Value::as_array)
        .ok_or("tools should be array")?;
    assert!(!tools.is_empty());

    Ok(())
}

#[test]
fn gemini_request_translation_local_image_path_uses_inline_data()
-> Result<(), Box<dyn std::error::Error>> {
    let path = write_temp_image_file("webp", &[82, 73, 70, 70, 0, 0, 0, 0, 87, 69, 66, 80])?;

    let request = Request::new(
        "gemini-2.5-pro",
        vec![Message::new(
            Role::User,
            vec![ContentPart::image_url(path.to_string_lossy())],
        )],
    );

    let translated = gemini::translate_request::translate_request(&request)?;
    let _ = fs::remove_file(&path);

    let contents = translated
        .get("contents")
        .and_then(serde_json::Value::as_array)
        .ok_or("contents should be array")?;
    let first = contents.first().ok_or("missing first content entry")?;
    let parts = first
        .get("parts")
        .and_then(serde_json::Value::as_array)
        .ok_or("parts should be array")?;
    let image = parts.first().ok_or("missing image part")?;

    assert!(image.get("inlineData").is_some(), "should use inlineData");
    assert!(
        image.get("fileData").is_none(),
        "local image paths should not use fileData"
    );
    assert_eq!(image["inlineData"]["mimeType"], "image/webp");
    let encoded = image["inlineData"]["data"]
        .as_str()
        .ok_or("missing inline base64 data")?;
    let decoded = base64::engine::general_purpose::STANDARD.decode(encoded)?;
    assert_eq!(decoded, vec![82, 73, 70, 70, 0, 0, 0, 0, 87, 69, 66, 80]);

    Ok(())
}

#[test]
fn gemini_response_translation_candidates_and_usage() -> Result<(), Box<dyn std::error::Error>> {
    let raw_response = fixture_json("gemini/response_translation_candidates_and_usage.json")?;

    let response = gemini::translate_response::translate_response(raw_response, None)?;

    assert_eq!(response.model, "gemini-2.5-pro");
    assert_eq!(response.provider, "gemini");
    assert_eq!(response.text(), "Rust is a systems language.");
    assert_eq!(response.finish_reason.reason, Reason::Stop);
    assert_eq!(response.usage.input_tokens, 12);
    assert_eq!(response.usage.output_tokens, 8);
    assert_eq!(response.usage.total_tokens, 20);
    assert_eq!(response.usage.cache_read_tokens, Some(4));
    assert_eq!(response.usage.reasoning_tokens, Some(2));

    Ok(())
}

#[test]
fn gemini_response_translation_function_call() -> Result<(), Box<dyn std::error::Error>> {
    let raw_response = fixture_json("gemini/response_translation_function_call.json")?;

    let response = gemini::translate_response::translate_response(raw_response, None)?;

    assert_eq!(response.tool_calls().len(), 1);
    let tc = &response.tool_calls()[0];
    assert_eq!(tc.name, "search");
    assert_eq!(response.finish_reason.reason, Reason::ToolCalls);

    Ok(())
}

#[test]
fn gemini_response_translation_thinking() -> Result<(), Box<dyn std::error::Error>> {
    let raw_response = fixture_json("gemini/response_translation_thinking.json")?;

    let response = gemini::translate_response::translate_response(raw_response, None)?;

    assert_eq!(response.reasoning(), Some("Thinking deeply...".to_string()));
    assert_eq!(response.text(), "The answer is 42");

    Ok(())
}

#[test]
fn gemini_stream_translation_text_and_finish() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = gemini::translate_stream::GeminiStreamState::default();

    // First chunk with text
    let chunk1 = fixture_sse_event(
        "message",
        "gemini/stream_translation_text_and_finish_chunk_1.json",
    )?;

    let events = gemini::translate_stream::translate_sse_event(&chunk1, &mut state)?;
    assert!(
        events
            .iter()
            .any(|e| e.event_type == StreamEventType::StreamStart)
    );
    assert!(
        events
            .iter()
            .any(|e| e.event_type == StreamEventType::TextStart)
    );
    assert!(
        events
            .iter()
            .any(|e| e.event_type == StreamEventType::TextDelta)
    );

    // Second chunk with finish reason and usage
    let chunk2 = fixture_sse_event(
        "message",
        "gemini/stream_translation_text_and_finish_chunk_2.json",
    )?;

    let events = gemini::translate_stream::translate_sse_event(&chunk2, &mut state)?;
    assert!(
        events
            .iter()
            .any(|e| e.event_type == StreamEventType::TextDelta)
    );

    // TextEnd should be emitted immediately when finishReason arrives
    assert!(
        events
            .iter()
            .any(|e| e.event_type == StreamEventType::TextEnd),
        "TextEnd should be emitted when finishReason arrives"
    );

    Ok(())
}

#[test]
fn gemini_stream_translation_function_call() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = gemini::translate_stream::GeminiStreamState::default();

    let chunk = fixture_sse_event("message", "gemini/stream_translation_function_call.json")?;

    let events = gemini::translate_stream::translate_sse_event(&chunk, &mut state)?;
    assert!(
        events
            .iter()
            .any(|e| e.event_type == StreamEventType::StreamStart)
    );
    assert!(
        events
            .iter()
            .any(|e| e.event_type == StreamEventType::ToolCallStart)
    );
    assert!(
        events
            .iter()
            .any(|e| e.event_type == StreamEventType::ToolCallEnd)
    );

    let tool_end = events
        .iter()
        .find(|e| e.event_type == StreamEventType::ToolCallEnd)
        .ok_or("missing ToolCallEnd")?;
    let tc = tool_end.tool_call.as_ref().ok_or("missing tool_call")?;
    assert_eq!(tc.name, "search");

    Ok(())
}

#[test]
fn gemini_stream_translation_emits_error_for_error_payload()
-> Result<(), Box<dyn std::error::Error>> {
    let mut state = gemini::translate_stream::GeminiStreamState::default();

    // An error chunk in the stream should produce an Error event
    let error_event = fixture_sse_event(
        "message",
        "gemini/stream_translation_emits_error_for_error_payload.json",
    )?;

    let events = gemini::translate_stream::translate_sse_event(&error_event, &mut state)?;
    assert!(
        events
            .iter()
            .any(|e| e.event_type == StreamEventType::Error),
        "should emit Error event for error payload"
    );

    // The error should not be emitted as a ProviderEvent
    assert!(
        !events
            .iter()
            .any(|e| e.event_type == StreamEventType::ProviderEvent),
        "error should not be a raw ProviderEvent"
    );
    // After error, state.finished is true so on_stream_end() won't emit
    // a spurious Finish event (verified by the finished guard at line 287).

    Ok(())
}

#[test]
fn gemini_error_translation_refines_resource_exhausted() {
    let err = SdkError::RateLimit {
        message: "Resource exhausted".to_string(),
        details: ProviderDetails {
            provider: None,
            status_code: Some(429),
            error_code: Some("RESOURCE_EXHAUSTED".to_string()),
            retryable: true,
            retry_after: None,
            raw: None,
        },
    };

    let translated = gemini::translate_error::translate_error(err);
    assert!(matches!(translated, SdkError::QuotaExceeded { .. }));

    if let SdkError::QuotaExceeded { details, .. } = translated {
        assert_eq!(details.provider.as_deref(), Some("gemini"));
    }
}

#[test]
fn gemini_request_translation_provider_options() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new("gemini-2.5-pro", vec![Message::user("Tell me about Rust")]);

    let mut provider_options = HashMap::new();
    provider_options.insert(
        "gemini".to_string(),
        serde_json::json!({
            "safetySettings": [{
                "category": "HARM_CATEGORY_DANGEROUS_CONTENT",
                "threshold": "BLOCK_ONLY_HIGH"
            }],
            "cachedContent": "cachedContents/abc123"
        }),
    );
    request.provider_options = Some(provider_options);

    let translated = gemini::translate_request::translate_request(&request)?;

    // Safety settings passed through to body
    let safety = translated
        .get("safetySettings")
        .and_then(serde_json::Value::as_array)
        .ok_or("safetySettings should be array")?;
    assert_eq!(safety.len(), 1);
    assert_eq!(safety[0]["category"], "HARM_CATEGORY_DANGEROUS_CONTENT");
    assert_eq!(safety[0]["threshold"], "BLOCK_ONLY_HIGH");

    // Cached content ref passed through
    assert_eq!(translated["cachedContent"], "cachedContents/abc123");

    Ok(())
}

#[test]
fn gemini_request_translation_rejects_non_object_provider_options() {
    let mut request = Request::new("gemini-2.5-pro", vec![Message::user("hello")]);

    let mut provider_options = HashMap::new();
    provider_options.insert("gemini".to_string(), serde_json::json!("not an object"));
    request.provider_options = Some(provider_options);

    let result = gemini::translate_request::translate_request(&request);
    assert!(matches!(result, Err(SdkError::InvalidRequest { .. })));
}

#[test]
fn gemini_request_translation_rejects_unknown_tool_call_id() {
    // A tool result referencing a tool_call_id that doesn't appear in any
    // prior assistant message should fail, not silently send "unknown_function".
    let request = Request::new(
        "gemini-2.5-pro",
        vec![
            Message::user("Call search"),
            Message::tool_result(
                "nonexistent_call_id",
                serde_json::json!({"result": "data"}),
                false,
            ),
        ],
    );

    let result = gemini::translate_request::translate_request(&request);
    assert!(matches!(result, Err(SdkError::InvalidRequest { .. })));
}

// ──────────────── Mistral adapter tests ────────────────

#[test]
fn mistral_request_translation_basic() -> Result<(), Box<dyn std::error::Error>> {
    let request = Request::new(
        "mistral-small-latest",
        vec![Message::system("sys"), Message::user("hi")],
    );

    let translated = mistral::translate_request::translate_request(&request, false)?;

    assert_eq!(translated.body["model"], "mistral-small-latest");
    assert!(translated.body.get("messages").is_some());
    assert_eq!(translated.body.get("stream"), None);

    Ok(())
}

#[test]
fn mistral_request_translation_omits_null_content() -> Result<(), Box<dyn std::error::Error>> {
    // Mistral rejects null values — assistant messages with only tool calls
    // must omit the content key entirely, not send "content": null.
    let request = Request::new(
        "mistral-small-latest",
        vec![
            Message::user("hi"),
            Message::new(
                Role::Assistant,
                vec![ContentPart::ToolCall {
                    tool_call: ToolCallData {
                        id: "call_1".to_string(),
                        name: "get_weather".to_string(),
                        arguments: serde_json::json!({"city": "Paris"}),
                        call_type: "function".to_string(),
                    },
                }],
            ),
            Message::tool_result("call_1", serde_json::json!({"temp": 22}), false),
        ],
    );

    let translated = mistral::translate_request::translate_request(&request, false)?;
    let messages = translated
        .body
        .get("messages")
        .and_then(serde_json::Value::as_array)
        .ok_or("missing messages")?;
    let assistant = messages
        .iter()
        .find(|m| m.get("role").and_then(serde_json::Value::as_str) == Some("assistant"))
        .ok_or("missing assistant message")?;

    // The "content" key should NOT be present (not null, not empty string).
    assert!(
        assistant.get("content").is_none(),
        "Mistral: content key should be omitted, not set to null"
    );

    // Tool calls should still be present
    assert!(
        assistant.get("tool_calls").is_some(),
        "Mistral: tool_calls should be present"
    );

    // Tool result content (a JSON object) should be stringified for Mistral.
    let tool_msg = messages
        .iter()
        .find(|m| m.get("role").and_then(serde_json::Value::as_str) == Some("tool"))
        .ok_or("missing tool message")?;
    let tool_content = tool_msg
        .get("content")
        .and_then(serde_json::Value::as_str)
        .ok_or("tool content should be a string")?;
    assert!(
        tool_content.contains("\"temp\""),
        "stringified tool content should contain the original JSON: {tool_content}"
    );

    Ok(())
}

#[test]
fn mistral_request_translation_omits_null_image_detail() -> Result<(), Box<dyn std::error::Error>> {
    // Mistral rejects null values — image_url objects must not include "detail": null.
    let request = Request::new(
        "mistral-small-latest",
        vec![Message::new(
            Role::User,
            vec![
                ContentPart::text("describe this"),
                ContentPart::image_url("https://example.com/img.png"),
            ],
        )],
    );

    let translated = mistral::translate_request::translate_request(&request, false)?;
    let messages = translated
        .body
        .get("messages")
        .and_then(serde_json::Value::as_array)
        .ok_or("missing messages")?;
    let user = messages
        .iter()
        .find(|m| m.get("role").and_then(serde_json::Value::as_str) == Some("user"))
        .ok_or("missing user message")?;

    // Find the image_url part
    let content = user
        .get("content")
        .and_then(serde_json::Value::as_array)
        .ok_or("missing content array")?;
    let image_part = content
        .iter()
        .find(|p| p.get("type").and_then(serde_json::Value::as_str) == Some("image_url"))
        .ok_or("missing image_url part")?;
    let image_url = image_part
        .get("image_url")
        .ok_or("missing image_url object")?;

    // detail key should NOT be present when the source ImageData has detail: None.
    assert!(
        image_url.get("detail").is_none(),
        "Mistral: detail should be omitted when None, not set to null"
    );
    assert_eq!(
        image_url.get("url").and_then(serde_json::Value::as_str),
        Some("https://example.com/img.png")
    );

    Ok(())
}

#[test]
fn mistral_request_translation_provider_options_namespace() -> Result<(), Box<dyn std::error::Error>>
{
    let mut request = Request::new("mistral-small-latest", vec![Message::user("hi")]);

    let mut provider_options = HashMap::new();
    provider_options.insert(
        "mistral".to_string(),
        serde_json::json!({"safe_prompt": true}),
    );
    request.provider_options = Some(provider_options);

    let translated = mistral::translate_request::translate_request(&request, false)?;
    assert_eq!(translated.body["safe_prompt"], true);

    Ok(())
}

#[test]
fn mistral_response_translation_basic() -> Result<(), Box<dyn std::error::Error>> {
    let raw_response = fixture_json("mistral/response_basic.json")?;

    let response = mistral::translate_response::translate_response(raw_response, None)?;

    assert_eq!(response.provider, "mistral");
    assert_eq!(response.model, "mistral-small-latest");
    assert_eq!(response.text(), "Hello from Mistral!");
    assert_eq!(response.finish_reason.reason, Reason::Stop);
    assert_eq!(response.usage.input_tokens, 8);
    assert_eq!(response.usage.output_tokens, 4);
    assert_eq!(response.usage.total_tokens, 12);

    Ok(())
}

#[test]
fn mistral_stream_translation_text_and_finish() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = mistral::translate_stream::MistralStreamState::default();

    let delta_event = fixture_sse_event("message", "mistral/stream_text_delta.json")?;

    let events = mistral::translate_stream::translate_sse_event(&delta_event, &mut state)?;
    assert_eq!(events[0].event_type, StreamEventType::StreamStart);
    assert_eq!(events[1].event_type, StreamEventType::TextStart);
    assert_eq!(events[2].event_type, StreamEventType::TextDelta);
    assert_eq!(events[2].delta.as_deref(), Some("Hel"));

    let finish_event = fixture_sse_event("message", "mistral/stream_finish.json")?;

    let finish_events = mistral::translate_stream::translate_sse_event(&finish_event, &mut state)?;

    let text_end = finish_events
        .iter()
        .find(|e| e.event_type == StreamEventType::TextEnd);
    assert!(text_end.is_some(), "expected TextEnd event");

    let finish = finish_events
        .iter()
        .find(|e| e.event_type == StreamEventType::Finish)
        .ok_or("missing finish event")?;
    assert_eq!(
        finish.finish_reason.as_ref().map(|f| f.reason),
        Some(Reason::Stop)
    );
    assert_eq!(finish.usage.as_ref().map(|u| u.total_tokens), Some(8));

    // Verify provider attribution in the accumulated response
    let response = finish
        .response
        .as_ref()
        .ok_or("missing response in finish event")?;
    assert_eq!(response.provider, "mistral");

    Ok(())
}

#[test]
fn mistral_error_translation_refines_not_found() {
    let err = SdkError::Server {
        message: "Model not found".to_string(),
        details: ProviderDetails {
            provider: None,
            status_code: Some(500),
            error_code: None,
            retryable: true,
            retry_after: None,
            raw: None,
        },
    };

    let translated = mistral::translate_error::translate_error(err);
    assert!(matches!(translated, SdkError::NotFound { .. }));

    if let SdkError::NotFound { details, .. } = translated {
        assert_eq!(details.provider.as_deref(), Some("mistral"));
    }
}

// ──────────────── DeepSeek adapter tests ────────────────

#[test]
fn deepseek_request_translation_basic() -> Result<(), Box<dyn std::error::Error>> {
    let request = Request::new(
        "deepseek-chat",
        vec![Message::system("sys"), Message::user("hi")],
    );

    let translated = deepseek::translate_request::translate_request(&request, false)?;

    assert_eq!(translated.body["model"], "deepseek-chat");
    assert!(translated.body.get("messages").is_some());
    assert_eq!(translated.body.get("stream"), None);

    Ok(())
}

#[test]
fn deepseek_request_translation_provider_options_namespace()
-> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new("deepseek-chat", vec![Message::user("hi")]);

    let mut provider_options = HashMap::new();
    provider_options.insert(
        "deepseek".to_string(),
        serde_json::json!({"temperature": 0.5}),
    );
    request.provider_options = Some(provider_options);

    let translated = deepseek::translate_request::translate_request(&request, false)?;
    assert_eq!(translated.body["temperature"], 0.5);

    Ok(())
}

#[test]
fn deepseek_request_translation_openai_compatible_namespace()
-> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new("deepseek-chat", vec![Message::user("hi")]);

    let mut provider_options = HashMap::new();
    provider_options.insert(
        "openai_compatible".to_string(),
        serde_json::json!({"frequency_penalty": 0.2}),
    );
    request.provider_options = Some(provider_options);

    let translated = deepseek::translate_request::translate_request(&request, false)?;
    assert_eq!(translated.body["frequency_penalty"], 0.2);

    Ok(())
}

#[test]
fn deepseek_request_translation_rejects_builtin_tools() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new("deepseek-chat", vec![Message::user("search for cats")]);

    let mut provider_options = HashMap::new();
    provider_options.insert(
        "deepseek".to_string(),
        serde_json::json!({"built_in_tools": [{"type": "web_search"}]}),
    );
    request.provider_options = Some(provider_options);

    let result = deepseek::translate_request::translate_request(&request, false);
    match result {
        Err(SdkError::InvalidRequest { message, .. }) => {
            assert!(
                message.contains("built-in tools"),
                "expected built-in tools rejection: {message}"
            );
        }
        Err(other) => {
            return Err(format!("expected InvalidRequest, got: {other}").into());
        }
        Ok(_) => {
            return Err("expected error for built-in tools, got Ok".into());
        }
    }

    Ok(())
}

#[test]
fn deepseek_response_translation_basic() -> Result<(), Box<dyn std::error::Error>> {
    let raw_response = fixture_json("deepseek/response_basic.json")?;

    let response = deepseek::translate_response::translate_response(raw_response, None)?;

    assert_eq!(response.provider, "deepseek");
    assert_eq!(response.model, "deepseek-chat");
    assert_eq!(response.text(), "Hello from DeepSeek!");
    assert_eq!(response.finish_reason.reason, Reason::Stop);
    assert_eq!(response.usage.input_tokens, 8);
    assert_eq!(response.usage.output_tokens, 4);
    assert_eq!(response.usage.total_tokens, 12);

    Ok(())
}

#[test]
fn deepseek_stream_translation_text_and_finish() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = deepseek::translate_stream::DeepSeekStreamState::default();

    let delta_event = fixture_sse_event("message", "deepseek/stream_text_delta.json")?;

    let events = deepseek::translate_stream::translate_sse_event(&delta_event, &mut state)?;
    assert_eq!(events[0].event_type, StreamEventType::StreamStart);
    assert_eq!(events[1].event_type, StreamEventType::TextStart);
    assert_eq!(events[2].event_type, StreamEventType::TextDelta);
    assert_eq!(events[2].delta.as_deref(), Some("Hel"));

    let finish_event = fixture_sse_event("message", "deepseek/stream_finish.json")?;

    let finish_events = deepseek::translate_stream::translate_sse_event(&finish_event, &mut state)?;

    let text_end = finish_events
        .iter()
        .find(|e| e.event_type == StreamEventType::TextEnd);
    assert!(text_end.is_some(), "expected TextEnd event");

    let finish = finish_events
        .iter()
        .find(|e| e.event_type == StreamEventType::Finish)
        .ok_or("missing finish event")?;
    assert_eq!(
        finish.finish_reason.as_ref().map(|f| f.reason),
        Some(Reason::Stop)
    );
    assert_eq!(finish.usage.as_ref().map(|u| u.total_tokens), Some(8));

    // Verify provider attribution in the accumulated response
    let response = finish
        .response
        .as_ref()
        .ok_or("missing response in finish event")?;
    assert_eq!(response.provider, "deepseek");

    Ok(())
}

#[test]
fn deepseek_error_translation_refines_not_found() {
    let err = SdkError::Server {
        message: "Model not found".to_string(),
        details: ProviderDetails {
            provider: None,
            status_code: Some(500),
            error_code: None,
            retryable: true,
            retry_after: None,
            raw: None,
        },
    };

    let translated = deepseek::translate_error::translate_error(err);
    assert!(matches!(translated, SdkError::NotFound { .. }));

    if let SdkError::NotFound { details, .. } = translated {
        assert_eq!(details.provider.as_deref(), Some("deepseek"));
    }
}

// ──────────────── Ollama adapter tests ────────────────

#[test]
fn ollama_request_translation_basic() -> Result<(), Box<dyn std::error::Error>> {
    let request = Request::new(
        "llama3.2:1b",
        vec![Message::system("sys"), Message::user("hi")],
    );

    let translated = ollama::translate_request::translate_request(&request, false)?;

    assert_eq!(translated.body["model"], "llama3.2:1b");
    assert!(translated.body.get("messages").is_some());
    assert_eq!(translated.body.get("stream"), None);

    Ok(())
}

#[test]
fn ollama_request_translation_provider_options_namespace() -> Result<(), Box<dyn std::error::Error>>
{
    let mut request = Request::new("llama3.2:1b", vec![Message::user("hi")]);

    let mut provider_options = HashMap::new();
    provider_options.insert(
        "ollama".to_string(),
        serde_json::json!({"temperature": 0.5}),
    );
    request.provider_options = Some(provider_options);

    let translated = ollama::translate_request::translate_request(&request, false)?;
    assert_eq!(translated.body["temperature"], 0.5);

    Ok(())
}

#[test]
fn ollama_request_translation_openai_compatible_namespace() -> Result<(), Box<dyn std::error::Error>>
{
    let mut request = Request::new("llama3.2:1b", vec![Message::user("hi")]);

    let mut provider_options = HashMap::new();
    provider_options.insert(
        "openai_compatible".to_string(),
        serde_json::json!({"frequency_penalty": 0.2}),
    );
    request.provider_options = Some(provider_options);

    let translated = ollama::translate_request::translate_request(&request, false)?;
    assert_eq!(translated.body["frequency_penalty"], 0.2);

    Ok(())
}

#[test]
fn ollama_request_translation_rejects_builtin_tools() -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::new("llama3.2:1b", vec![Message::user("search for cats")]);

    let mut provider_options = HashMap::new();
    provider_options.insert(
        "ollama".to_string(),
        serde_json::json!({"built_in_tools": [{"type": "web_search"}]}),
    );
    request.provider_options = Some(provider_options);

    let result = ollama::translate_request::translate_request(&request, false);
    match result {
        Err(SdkError::InvalidRequest { message, .. }) => {
            assert!(
                message.contains("built-in tools"),
                "expected built-in tools rejection: {message}"
            );
        }
        Err(other) => {
            return Err(format!("expected InvalidRequest, got: {other}").into());
        }
        Ok(_) => {
            return Err("expected error for built-in tools, got Ok".into());
        }
    }

    Ok(())
}

#[test]
fn ollama_response_translation_basic() -> Result<(), Box<dyn std::error::Error>> {
    let raw_response = fixture_json("ollama/response_basic.json")?;

    let response = ollama::translate_response::translate_response(raw_response, None)?;

    assert_eq!(response.provider, "ollama");
    assert_eq!(response.model, "llama3.2:1b");
    assert_eq!(response.text(), "Hello from Ollama!");
    assert_eq!(response.finish_reason.reason, Reason::Stop);
    assert_eq!(response.usage.input_tokens, 8);
    assert_eq!(response.usage.output_tokens, 4);
    assert_eq!(response.usage.total_tokens, 12);

    Ok(())
}

#[test]
fn ollama_stream_translation_text_and_finish() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = ollama::translate_stream::OllamaStreamState::default();

    let delta_event = fixture_sse_event("message", "ollama/stream_text_delta.json")?;

    let events = ollama::translate_stream::translate_sse_event(&delta_event, &mut state)?;
    assert_eq!(events[0].event_type, StreamEventType::StreamStart);
    assert_eq!(events[1].event_type, StreamEventType::TextStart);
    assert_eq!(events[2].event_type, StreamEventType::TextDelta);
    assert_eq!(events[2].delta.as_deref(), Some("Hel"));

    let finish_event = fixture_sse_event("message", "ollama/stream_finish.json")?;

    let finish_events = ollama::translate_stream::translate_sse_event(&finish_event, &mut state)?;

    let text_end = finish_events
        .iter()
        .find(|e| e.event_type == StreamEventType::TextEnd);
    assert!(text_end.is_some(), "expected TextEnd event");

    let finish = finish_events
        .iter()
        .find(|e| e.event_type == StreamEventType::Finish)
        .ok_or("missing finish event")?;
    assert_eq!(
        finish.finish_reason.as_ref().map(|f| f.reason),
        Some(Reason::Stop)
    );
    assert_eq!(finish.usage.as_ref().map(|u| u.total_tokens), Some(8));

    // Verify provider attribution in the accumulated response
    let response = finish
        .response
        .as_ref()
        .ok_or("missing response in finish event")?;
    assert_eq!(response.provider, "ollama");

    Ok(())
}

#[test]
fn ollama_error_translation_refines_not_found() {
    let err = SdkError::Server {
        message: "Model not found".to_string(),
        details: ProviderDetails {
            provider: None,
            status_code: Some(500),
            error_code: None,
            retryable: true,
            retry_after: None,
            raw: None,
        },
    };

    let translated = ollama::translate_error::translate_error(err);
    assert!(matches!(translated, SdkError::NotFound { .. }));

    if let SdkError::NotFound { details, .. } = translated {
        assert_eq!(details.provider.as_deref(), Some("ollama"));
    }
}
