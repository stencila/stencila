//! Spec Section 7 conformance tests.
//!
//! Target areas:
//! - Provider-native request translation
//! - Provider-native response translation
//! - Streaming event translation and SSE handling
//! - OpenAI-ChatCompletions adapter constraints

use std::collections::HashMap;

use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use stencila_models3::error::{ProviderDetails, SdkError};
use stencila_models3::http::sse::SseEvent;
use stencila_models3::providers::{
    anthropic, gemini, openai,
    openai_chat_completions::{self as chat},
};
use stencila_models3::types::{
    content::{ContentPart, ToolCallData},
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

    let tools = translated
        .body
        .get("tools")
        .and_then(serde_json::Value::as_array)
        .ok_or("tools should be array")?;
    assert_eq!(tools.len(), 2);

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
fn openai_response_translation_maps_usage_fields() -> Result<(), Box<dyn std::error::Error>> {
    let raw_response = serde_json::json!({
        "id": "resp_123",
        "model": "gpt-5.2",
        "status": "completed",
        "output": [
            {
                "type": "message",
                "content": [
                    {"type": "output_text", "text": "Result: "},
                    {"type": "output_text", "text": "sunny"}
                ]
            },
            {
                "type": "function_call",
                "id": "call_1",
                "name": "get_weather",
                "arguments": "{\"city\":\"Paris\"}"
            }
        ],
        "usage": {
            "input_tokens": 20,
            "output_tokens": 30,
            "total_tokens": 50,
            "output_tokens_details": {"reasoning_tokens": 11},
            "prompt_tokens_details": {"cached_tokens": 7}
        }
    });

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

    let delta_event = SseEvent {
        event_type: "response.output_text.delta".to_string(),
        data: serde_json::json!({
            "type": "response.output_text.delta",
            "delta": "Hel",
            "output_index": 0,
            "content_index": 0
        })
        .to_string(),
        retry: None,
    };

    let events = openai::translate_stream::translate_sse_event(&delta_event, &mut state)?;
    assert_eq!(events[0].event_type, StreamEventType::StreamStart);
    assert_eq!(events[1].event_type, StreamEventType::TextStart);
    assert_eq!(events[2].event_type, StreamEventType::TextDelta);
    assert_eq!(events[2].delta.as_deref(), Some("Hel"));

    let end_event = SseEvent {
        event_type: "response.output_item.done".to_string(),
        data: serde_json::json!({
            "type": "response.output_item.done",
            "item": {
                "id": "text_0_0",
                "type": "message"
            }
        })
        .to_string(),
        retry: None,
    };

    let end_events = openai::translate_stream::translate_sse_event(&end_event, &mut state)?;
    assert_eq!(end_events[0].event_type, StreamEventType::TextEnd);

    let completed_event = SseEvent {
        event_type: "response.completed".to_string(),
        data: serde_json::json!({
            "type": "response.completed",
            "response": {
                "id": "resp_final",
                "model": "gpt-5.2",
                "status": "completed",
                "output": [{
                    "type": "message",
                    "content": [{"type": "output_text", "text": "Hello"}]
                }],
                "usage": {
                    "input_tokens": 3,
                    "output_tokens": 4,
                    "total_tokens": 7,
                    "output_tokens_details": {"reasoning_tokens": 1}
                }
            }
        })
        .to_string(),
        retry: None,
    };

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
    let raw_response = serde_json::json!({
        "id": "chatcmpl_123",
        "model": "llama-3.1-70b",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Hello"
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 6,
            "total_tokens": 16,
            "prompt_tokens_details": {"cached_tokens": 3}
        }
    });

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

    let usage_event = SseEvent {
        event_type: "message".to_string(),
        data: serde_json::json!({
            "usage": {
                "prompt_tokens": 4,
                "completion_tokens": 3,
                "total_tokens": 7
            }
        })
        .to_string(),
        retry: None,
    };

    let usage_events = chat::translate_stream::translate_sse_event(&usage_event, &mut state)?;
    assert_eq!(usage_events[0].event_type, StreamEventType::StreamStart);

    let tool_delta_1 = SseEvent {
        event_type: "message".to_string(),
        data: serde_json::json!({
            "choices": [{
                "index": 0,
                "delta": {
                    "tool_calls": [{
                        "index": 0,
                        "id": "call_1",
                        "function": {
                            "name": "get_weather",
                            "arguments": "{\"city\":\"Par"
                        }
                    }]
                },
                "finish_reason": null
            }]
        })
        .to_string(),
        retry: None,
    };

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

    let tool_delta_2 = SseEvent {
        event_type: "message".to_string(),
        data: serde_json::json!({
            "choices": [{
                "index": 0,
                "delta": {
                    "tool_calls": [{
                        "index": 0,
                        "function": {"arguments": "is\"}"}
                    }]
                },
                "finish_reason": null
            }]
        })
        .to_string(),
        retry: None,
    };

    let _tool_events_2 = chat::translate_stream::translate_sse_event(&tool_delta_2, &mut state)?;

    let finish_event = SseEvent {
        event_type: "message".to_string(),
        data: serde_json::json!({
            "choices": [{
                "index": 0,
                "delta": {},
                "finish_reason": "tool_calls"
            }]
        })
        .to_string(),
        retry: None,
    };

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
    let error_event = SseEvent {
        event_type: "message".to_string(),
        data: serde_json::json!({
            "error": {
                "message": "Internal server error",
                "type": "server_error",
                "code": "internal_error"
            }
        })
        .to_string(),
        retry: None,
    };

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

    let translated = anthropic::translate_request::translate_request(&request, false)?;

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
fn anthropic_response_translation_content_blocks() -> Result<(), Box<dyn std::error::Error>> {
    let raw_response = serde_json::json!({
        "id": "msg_123",
        "type": "message",
        "role": "assistant",
        "model": "claude-sonnet-4-5-20250929",
        "content": [
            {"type": "text", "text": "Here's the weather:"},
            {
                "type": "tool_use",
                "id": "toolu_1",
                "name": "get_weather",
                "input": {"city": "Paris"}
            }
        ],
        "stop_reason": "tool_use",
        "usage": {
            "input_tokens": 15,
            "output_tokens": 25,
            "cache_read_input_tokens": 5,
            "cache_creation_input_tokens": 3
        }
    });

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
    let raw_response = serde_json::json!({
        "id": "msg_think",
        "type": "message",
        "role": "assistant",
        "model": "claude-sonnet-4-5-20250929",
        "content": [
            {
                "type": "thinking",
                "thinking": "Let me think about this...",
                "signature": "sig_abc"
            },
            {"type": "text", "text": "The answer is 42"}
        ],
        "stop_reason": "end_turn",
        "usage": {"input_tokens": 10, "output_tokens": 20}
    });

    let response = anthropic::translate_response::translate_response(raw_response, None)?;

    assert_eq!(
        response.reasoning(),
        Some("Let me think about this...".to_string())
    );
    assert_eq!(response.text(), "The answer is 42");
    assert_eq!(response.finish_reason.reason, Reason::Stop);

    Ok(())
}

#[test]
fn anthropic_stream_translation_full_sequence() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = anthropic::translate_stream::AnthropicStreamState::default();

    // message_start
    let msg_start = SseEvent {
        event_type: "message_start".to_string(),
        data: serde_json::json!({
            "type": "message_start",
            "message": {
                "id": "msg_stream_1",
                "model": "claude-sonnet-4-5-20250929",
                "usage": {"input_tokens": 10}
            }
        })
        .to_string(),
        retry: None,
    };
    let events = anthropic::translate_stream::translate_sse_event(&msg_start, &mut state)?;
    assert_eq!(events[0].event_type, StreamEventType::StreamStart);

    // content_block_start (text)
    let block_start = SseEvent {
        event_type: "content_block_start".to_string(),
        data: serde_json::json!({
            "type": "content_block_start",
            "index": 0,
            "content_block": {"type": "text", "text": ""}
        })
        .to_string(),
        retry: None,
    };
    let events = anthropic::translate_stream::translate_sse_event(&block_start, &mut state)?;
    assert_eq!(events[0].event_type, StreamEventType::TextStart);

    // content_block_delta (text)
    let block_delta = SseEvent {
        event_type: "content_block_delta".to_string(),
        data: serde_json::json!({
            "type": "content_block_delta",
            "index": 0,
            "delta": {"type": "text_delta", "text": "Hello!"}
        })
        .to_string(),
        retry: None,
    };
    let events = anthropic::translate_stream::translate_sse_event(&block_delta, &mut state)?;
    assert_eq!(events[0].event_type, StreamEventType::TextDelta);
    assert_eq!(events[0].delta.as_deref(), Some("Hello!"));

    // content_block_stop
    let block_stop = SseEvent {
        event_type: "content_block_stop".to_string(),
        data: serde_json::json!({"type": "content_block_stop", "index": 0}).to_string(),
        retry: None,
    };
    let events = anthropic::translate_stream::translate_sse_event(&block_stop, &mut state)?;
    assert_eq!(events[0].event_type, StreamEventType::TextEnd);

    // message_delta
    let msg_delta = SseEvent {
        event_type: "message_delta".to_string(),
        data: serde_json::json!({
            "type": "message_delta",
            "delta": {"stop_reason": "end_turn"},
            "usage": {"output_tokens": 8}
        })
        .to_string(),
        retry: None,
    };
    let _events = anthropic::translate_stream::translate_sse_event(&msg_delta, &mut state)?;

    // message_stop
    let msg_stop = SseEvent {
        event_type: "message_stop".to_string(),
        data: serde_json::json!({"type": "message_stop"}).to_string(),
        retry: None,
    };
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

    // message_start
    let msg_start = SseEvent {
        event_type: "message_start".to_string(),
        data: serde_json::json!({
            "type": "message_start",
            "message": {
                "id": "msg_sig_1",
                "model": "claude-sonnet-4-5-20250929",
                "usage": {"input_tokens": 10}
            }
        })
        .to_string(),
        retry: None,
    };
    anthropic::translate_stream::translate_sse_event(&msg_start, &mut state)?;

    // content_block_start (thinking)
    let block_start = SseEvent {
        event_type: "content_block_start".to_string(),
        data: serde_json::json!({
            "type": "content_block_start",
            "index": 0,
            "content_block": {"type": "thinking", "thinking": ""}
        })
        .to_string(),
        retry: None,
    };
    anthropic::translate_stream::translate_sse_event(&block_start, &mut state)?;

    // content_block_delta (thinking text)
    let thinking_delta = SseEvent {
        event_type: "content_block_delta".to_string(),
        data: serde_json::json!({
            "type": "content_block_delta",
            "index": 0,
            "delta": {"type": "thinking_delta", "thinking": "Let me reason..."}
        })
        .to_string(),
        retry: None,
    };
    anthropic::translate_stream::translate_sse_event(&thinking_delta, &mut state)?;

    // content_block_delta (signature)
    let sig_delta = SseEvent {
        event_type: "content_block_delta".to_string(),
        data: serde_json::json!({
            "type": "content_block_delta",
            "index": 0,
            "delta": {"type": "signature_delta", "signature": "sig_abc123"}
        })
        .to_string(),
        retry: None,
    };
    anthropic::translate_stream::translate_sse_event(&sig_delta, &mut state)?;

    // content_block_stop
    let block_stop = SseEvent {
        event_type: "content_block_stop".to_string(),
        data: serde_json::json!({"type": "content_block_stop", "index": 0}).to_string(),
        retry: None,
    };
    anthropic::translate_stream::translate_sse_event(&block_stop, &mut state)?;

    // content_block_start (text)
    let text_start = SseEvent {
        event_type: "content_block_start".to_string(),
        data: serde_json::json!({
            "type": "content_block_start",
            "index": 1,
            "content_block": {"type": "text", "text": ""}
        })
        .to_string(),
        retry: None,
    };
    anthropic::translate_stream::translate_sse_event(&text_start, &mut state)?;

    // content_block_delta (text)
    let text_delta = SseEvent {
        event_type: "content_block_delta".to_string(),
        data: serde_json::json!({
            "type": "content_block_delta",
            "index": 1,
            "delta": {"type": "text_delta", "text": "The answer."}
        })
        .to_string(),
        retry: None,
    };
    anthropic::translate_stream::translate_sse_event(&text_delta, &mut state)?;

    // content_block_stop (text)
    let text_stop = SseEvent {
        event_type: "content_block_stop".to_string(),
        data: serde_json::json!({"type": "content_block_stop", "index": 1}).to_string(),
        retry: None,
    };
    anthropic::translate_stream::translate_sse_event(&text_stop, &mut state)?;

    // message_delta + message_stop to get the final response
    let msg_delta = SseEvent {
        event_type: "message_delta".to_string(),
        data: serde_json::json!({
            "type": "message_delta",
            "delta": {"stop_reason": "end_turn"},
            "usage": {"output_tokens": 15}
        })
        .to_string(),
        retry: None,
    };
    anthropic::translate_stream::translate_sse_event(&msg_delta, &mut state)?;

    let msg_stop = SseEvent {
        event_type: "message_stop".to_string(),
        data: serde_json::json!({"type": "message_stop"}).to_string(),
        retry: None,
    };
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

    let translated = anthropic::translate_request::translate_request(&request, false)?;

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

    let translated = anthropic::translate_request::translate_request(&request, false)?;

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

    let translated = anthropic::translate_request::translate_request(&request, false)?;

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
fn gemini_response_translation_candidates_and_usage() -> Result<(), Box<dyn std::error::Error>> {
    let raw_response = serde_json::json!({
        "modelVersion": "gemini-2.5-pro",
        "candidates": [{
            "content": {
                "parts": [
                    {"text": "Rust is a systems language."}
                ],
                "role": "model"
            },
            "finishReason": "STOP"
        }],
        "usageMetadata": {
            "promptTokenCount": 12,
            "candidatesTokenCount": 8,
            "totalTokenCount": 20,
            "cachedContentTokenCount": 4,
            "thoughtsTokenCount": 2
        }
    });

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
    let raw_response = serde_json::json!({
        "modelVersion": "gemini-2.5-pro",
        "candidates": [{
            "content": {
                "parts": [{
                    "functionCall": {
                        "name": "search",
                        "args": {"query": "Rust programming"}
                    }
                }],
                "role": "model"
            },
            "finishReason": "STOP"
        }],
        "usageMetadata": {
            "promptTokenCount": 10,
            "candidatesTokenCount": 5,
            "totalTokenCount": 15
        }
    });

    let response = gemini::translate_response::translate_response(raw_response, None)?;

    assert_eq!(response.tool_calls().len(), 1);
    let tc = &response.tool_calls()[0];
    assert_eq!(tc.name, "search");
    assert_eq!(response.finish_reason.reason, Reason::ToolCalls);

    Ok(())
}

#[test]
fn gemini_response_translation_thinking() -> Result<(), Box<dyn std::error::Error>> {
    let raw_response = serde_json::json!({
        "modelVersion": "gemini-2.5-pro",
        "candidates": [{
            "content": {
                "parts": [
                    {"text": "Thinking deeply...", "thought": true},
                    {"text": "The answer is 42"}
                ],
                "role": "model"
            },
            "finishReason": "STOP"
        }],
        "usageMetadata": {
            "promptTokenCount": 5,
            "candidatesTokenCount": 10,
            "totalTokenCount": 15,
            "thoughtsTokenCount": 3
        }
    });

    let response = gemini::translate_response::translate_response(raw_response, None)?;

    assert_eq!(response.reasoning(), Some("Thinking deeply...".to_string()));
    assert_eq!(response.text(), "The answer is 42");

    Ok(())
}

#[test]
fn gemini_stream_translation_text_and_finish() -> Result<(), Box<dyn std::error::Error>> {
    let mut state = gemini::translate_stream::GeminiStreamState::default();

    // First chunk with text
    let chunk1 = SseEvent {
        event_type: "message".to_string(),
        data: serde_json::json!({
            "modelVersion": "gemini-2.5-pro",
            "candidates": [{
                "content": {
                    "parts": [{"text": "Hello "}],
                    "role": "model"
                }
            }]
        })
        .to_string(),
        retry: None,
    };

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
    let chunk2 = SseEvent {
        event_type: "message".to_string(),
        data: serde_json::json!({
            "candidates": [{
                "content": {
                    "parts": [{"text": "world!"}],
                    "role": "model"
                },
                "finishReason": "STOP"
            }],
            "usageMetadata": {
                "promptTokenCount": 5,
                "candidatesTokenCount": 3,
                "totalTokenCount": 8
            }
        })
        .to_string(),
        retry: None,
    };

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

    let chunk = SseEvent {
        event_type: "message".to_string(),
        data: serde_json::json!({
            "modelVersion": "gemini-2.5-pro",
            "candidates": [{
                "content": {
                    "parts": [{
                        "functionCall": {
                            "name": "search",
                            "args": {"query": "Rust"}
                        }
                    }],
                    "role": "model"
                },
                "finishReason": "STOP"
            }],
            "usageMetadata": {
                "promptTokenCount": 5,
                "candidatesTokenCount": 3,
                "totalTokenCount": 8
            }
        })
        .to_string(),
        retry: None,
    };

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
    let error_event = SseEvent {
        event_type: "message".to_string(),
        data: serde_json::json!({
            "error": {
                "code": 500,
                "message": "Internal error",
                "status": "INTERNAL"
            }
        })
        .to_string(),
        retry: None,
    };

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
