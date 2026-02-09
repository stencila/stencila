//! Spec Section 3 conformance tests.
//!
//! Target areas:
//! - Role/content/message/request/response type behavior
//! - Serde round-trip conformance
//! - Edge cases: empty content, missing optional fields, unsupported variants

use stencila_models3::error::{ProviderDetails, SdkError};
use stencila_models3::types::{
    content::{AudioData, ContentPart, DocumentData, ImageData, ThinkingData, ToolCallData},
    finish_reason::{FinishReason, Reason},
    message::Message,
    rate_limit::RateLimitInfo,
    request::Request,
    response::Response,
    response_format::{ResponseFormat, ResponseFormatType},
    role::Role,
    stream_event::{StreamEvent, StreamEventType},
    timeout::Timeout,
    tool::{ToolCall, ToolChoice, ToolDefinition, ToolResult},
    usage::Usage,
    warning::Warning,
};

// ── Role ──────────────────────────────────────────────────────────────

#[test]
fn role_serde_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    for (role, expected_json) in [
        (Role::System, "\"system\""),
        (Role::User, "\"user\""),
        (Role::Assistant, "\"assistant\""),
        (Role::Tool, "\"tool\""),
        (Role::Developer, "\"developer\""),
    ] {
        let json = serde_json::to_string(&role)?;
        assert_eq!(json, expected_json, "serialization of {role:?}");
        let back: Role = serde_json::from_str(&json)?;
        assert_eq!(back, role, "round-trip of {role:?}");
    }
    Ok(())
}

// ── Message constructors ──────────────────────────────────────────────

#[test]
fn message_system_constructor() {
    let msg = Message::system("Be helpful.");
    assert_eq!(msg.role, Role::System);
    assert_eq!(msg.text(), "Be helpful.");
    assert!(msg.name.is_none());
    assert!(msg.tool_call_id.is_none());
}

#[test]
fn message_user_constructor() {
    let msg = Message::user("Hello");
    assert_eq!(msg.role, Role::User);
    assert_eq!(msg.text(), "Hello");
}

#[test]
fn message_assistant_constructor() {
    let msg = Message::assistant("Hi there");
    assert_eq!(msg.role, Role::Assistant);
    assert_eq!(msg.text(), "Hi there");
}

#[test]
fn message_developer_constructor() {
    let msg = Message::developer("Internal instruction");
    assert_eq!(msg.role, Role::Developer);
    assert_eq!(msg.text(), "Internal instruction");
}

#[test]
fn message_tool_result_text_constructor() {
    let msg = Message::tool_result_text("call_123", "result text", false);
    assert_eq!(msg.role, Role::Tool);
    assert_eq!(msg.tool_call_id.as_deref(), Some("call_123"));
    assert_eq!(msg.content.len(), 1);
}

#[test]
fn message_tool_result_structured_constructor() {
    let value = serde_json::json!({"temperature": 72, "unit": "F"});
    let msg = Message::tool_result("call_456", value.clone(), false);
    assert_eq!(msg.role, Role::Tool);
    assert_eq!(msg.tool_call_id.as_deref(), Some("call_456"));
    if let ContentPart::ToolResult { tool_result } = &msg.content[0] {
        assert_eq!(tool_result.content, value);
    }
}

// ── Message.text() ────────────────────────────────────────────────────

#[test]
fn message_text_concatenates_multiple_text_parts() {
    let msg = Message::new(
        Role::Assistant,
        vec![ContentPart::text("Hello, "), ContentPart::text("world!")],
    );
    assert_eq!(msg.text(), "Hello, world!");
}

#[test]
fn message_text_skips_non_text_parts() {
    let msg = Message::new(
        Role::Assistant,
        vec![
            ContentPart::text("Hello"),
            ContentPart::image_url("https://example.com/img.png"),
            ContentPart::text(" there"),
        ],
    );
    assert_eq!(msg.text(), "Hello there");
}

#[test]
fn message_text_empty_when_no_text_parts() {
    let msg = Message::new(
        Role::Assistant,
        vec![ContentPart::image_url("https://example.com/img.png")],
    );
    assert_eq!(msg.text(), "");
}

#[test]
fn message_text_empty_content() {
    let msg = Message::new(Role::User, vec![]);
    assert_eq!(msg.text(), "");
}

// ── ContentPart serde ─────────────────────────────────────────────────

#[test]
fn content_part_text_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let part = ContentPart::text("Hello");
    let json = serde_json::to_string(&part)?;
    assert!(json.contains("\"kind\":\"text\""));
    let back: ContentPart = serde_json::from_str(&json)?;
    assert_eq!(back, part);
    Ok(())
}

#[test]
fn content_part_tool_call_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let part = ContentPart::tool_call("id1", "get_weather", serde_json::json!({"city": "NYC"}));
    let json = serde_json::to_string(&part)?;
    let back: ContentPart = serde_json::from_str(&json)?;
    assert_eq!(back, part);
    Ok(())
}

#[test]
fn content_part_tool_result_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let part = ContentPart::tool_result("id1", serde_json::Value::String("72°F".into()), false);
    let json = serde_json::to_string(&part)?;
    let back: ContentPart = serde_json::from_str(&json)?;
    assert_eq!(back, part);
    Ok(())
}

#[test]
fn content_part_thinking_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let part = ContentPart::Thinking {
        thinking: ThinkingData {
            text: "Let me reason...".into(),
            signature: Some("sig123".into()),
            redacted: false,
        },
    };
    let json = serde_json::to_string(&part)?;
    let back: ContentPart = serde_json::from_str(&json)?;
    assert_eq!(back, part);
    Ok(())
}

#[test]
fn content_part_image_url_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let part = ContentPart::image_url("https://example.com/img.png");
    let json = serde_json::to_string(&part)?;
    let back: ContentPart = serde_json::from_str(&json)?;
    assert_eq!(back, part);
    Ok(())
}

#[test]
fn content_part_image_data_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let bytes = vec![0x89, 0x50, 0x4E, 0x47]; // PNG magic bytes
    let part = ContentPart::image_data(bytes.clone(), "image/png");
    let json = serde_json::to_string(&part)?;
    let back: ContentPart = serde_json::from_str(&json)?;
    assert_eq!(back, part);
    Ok(())
}

// ── FinishReason ──────────────────────────────────────────────────────

#[test]
fn finish_reason_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let fr = FinishReason::new(Reason::Stop, Some("end_turn".into()));
    let json = serde_json::to_string(&fr)?;
    let back: FinishReason = serde_json::from_str(&json)?;
    assert_eq!(back, fr);
    Ok(())
}

#[test]
fn finish_reason_without_raw() -> Result<(), Box<dyn std::error::Error>> {
    let fr = FinishReason::new(Reason::ToolCalls, None);
    let json = serde_json::to_string(&fr)?;
    assert!(!json.contains("raw"));
    let back: FinishReason = serde_json::from_str(&json)?;
    assert_eq!(back, fr);
    Ok(())
}

// ── Usage ─────────────────────────────────────────────────────────────

#[test]
fn usage_addition_both_some() {
    let a = Usage {
        input_tokens: 100,
        output_tokens: 50,
        total_tokens: 150,
        reasoning_tokens: Some(10),
        cache_read_tokens: Some(20),
        cache_write_tokens: None,
        raw: None,
    };
    let b = Usage {
        input_tokens: 200,
        output_tokens: 100,
        total_tokens: 300,
        reasoning_tokens: Some(30),
        cache_read_tokens: Some(40),
        cache_write_tokens: Some(5),
        raw: None,
    };
    let sum = a + b;
    assert_eq!(sum.input_tokens, 300);
    assert_eq!(sum.output_tokens, 150);
    assert_eq!(sum.total_tokens, 450);
    assert_eq!(sum.reasoning_tokens, Some(40));
    assert_eq!(sum.cache_read_tokens, Some(60));
    assert_eq!(sum.cache_write_tokens, Some(5));
}

#[test]
fn usage_addition_none_none() {
    let a = Usage::default();
    let b = Usage::default();
    let sum = a + b;
    assert_eq!(sum.reasoning_tokens, None);
    assert_eq!(sum.cache_read_tokens, None);
    assert_eq!(sum.cache_write_tokens, None);
}

#[test]
fn usage_addition_some_none() {
    let a = Usage {
        reasoning_tokens: Some(10),
        ..Usage::default()
    };
    let b = Usage::default();
    let sum = a + b;
    assert_eq!(sum.reasoning_tokens, Some(10));
}

#[test]
fn usage_serde_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let usage = Usage {
        input_tokens: 100,
        output_tokens: 50,
        total_tokens: 150,
        reasoning_tokens: Some(10),
        cache_read_tokens: None,
        cache_write_tokens: None,
        raw: None,
    };
    let json = serde_json::to_string(&usage)?;
    let back: Usage = serde_json::from_str(&json)?;
    assert_eq!(back, usage);
    Ok(())
}

// ── Request ───────────────────────────────────────────────────────────

#[test]
fn request_minimal() -> Result<(), Box<dyn std::error::Error>> {
    let req = Request::new("test-model", vec![Message::user("Hi")]);
    assert_eq!(req.model, "test-model");
    assert_eq!(req.messages.len(), 1);
    assert!(req.provider.is_none());
    assert!(req.tools.is_none());
    let json = serde_json::to_string(&req)?;
    let back: Request = serde_json::from_str(&json)?;
    assert_eq!(back.model, req.model);
    Ok(())
}

#[test]
fn request_provider_options_extraction() {
    let mut opts = std::collections::HashMap::new();
    opts.insert(
        "anthropic".to_string(),
        serde_json::json!({"thinking": true}),
    );
    opts.insert(
        "openai".to_string(),
        serde_json::json!({"builtin_tools": []}),
    );
    let req = Request {
        provider_options: Some(opts),
        ..Request::new("test-model", vec![Message::user("Hi")])
    };
    let anthropic_opts = req.provider_options_for("anthropic");
    assert!(anthropic_opts.is_some());
    assert!(req.provider_options_for("gemini").is_none());
}

// ── Response ──────────────────────────────────────────────────────────

#[test]
fn response_text_accessor() {
    let resp = make_response(vec![
        ContentPart::text("Hello, "),
        ContentPart::text("world!"),
    ]);
    assert_eq!(resp.text(), "Hello, world!");
}

#[test]
fn response_tool_calls_accessor() {
    let resp = make_response(vec![
        ContentPart::text("I'll call a tool"),
        ContentPart::tool_call("id1", "get_weather", serde_json::json!({})),
        ContentPart::tool_call("id2", "get_time", serde_json::json!({})),
    ]);
    let calls = resp.tool_calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].name, "get_weather");
    assert_eq!(calls[1].name, "get_time");
}

#[test]
fn response_reasoning_accessor() {
    let resp = make_response(vec![
        ContentPart::Thinking {
            thinking: ThinkingData {
                text: "Step 1. ".into(),
                signature: None,
                redacted: false,
            },
        },
        ContentPart::text("Answer"),
        ContentPart::Thinking {
            thinking: ThinkingData {
                text: "Step 2.".into(),
                signature: None,
                redacted: false,
            },
        },
    ]);
    assert_eq!(resp.reasoning(), Some("Step 1. Step 2.".into()));
}

// ── Tool types ────────────────────────────────────────────────────────

#[test]
fn tool_definition_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let tool = ToolDefinition {
        name: "get_weather".into(),
        description: "Get the weather for a city".into(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "city": {"type": "string"}
            },
            "required": ["city"]
        }),
        strict: false,
    };
    tool.validate()?;
    let json = serde_json::to_string(&tool)?;
    let back: ToolDefinition = serde_json::from_str(&json)?;
    assert_eq!(back, tool);
    Ok(())
}

#[test]
fn tool_definition_validate_rejects_bad_name() {
    let tool = ToolDefinition {
        name: "123bad".into(),
        description: "desc".into(),
        parameters: serde_json::json!({"type": "object"}),
        strict: false,
    };
    assert!(
        tool.validate().is_err(),
        "name starting with digit should fail"
    );

    let tool2 = ToolDefinition {
        name: "has space".into(),
        description: "desc".into(),
        parameters: serde_json::json!({"type": "object"}),
        strict: false,
    };
    assert!(tool2.validate().is_err(), "name with space should fail");

    let long_name: String = "a".repeat(65);
    let tool3 = ToolDefinition {
        name: long_name,
        description: "desc".into(),
        parameters: serde_json::json!({"type": "object"}),
        strict: false,
    };
    assert!(tool3.validate().is_err(), "name > 64 chars should fail");
}

#[test]
fn tool_definition_validate_rejects_empty_description() {
    let tool = ToolDefinition {
        name: "my_tool".into(),
        description: String::new(),
        parameters: serde_json::json!({"type": "object"}),
        strict: false,
    };
    assert!(tool.validate().is_err(), "empty description should fail");
}

#[test]
fn tool_definition_validate_rejects_non_object_parameters() {
    let tool = ToolDefinition {
        name: "my_tool".into(),
        description: "desc".into(),
        parameters: serde_json::json!({"type": "array"}),
        strict: false,
    };
    assert!(
        tool.validate().is_err(),
        "non-object parameters should fail"
    );
}

#[test]
fn tool_choice_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    for choice in [
        ToolChoice::Auto,
        ToolChoice::None,
        ToolChoice::Required,
        ToolChoice::Tool("get_weather".into()),
    ] {
        let json = serde_json::to_string(&choice)?;
        let back: ToolChoice = serde_json::from_str(&json)?;
        assert_eq!(back, choice);
    }
    Ok(())
}

/// `ToolChoice` uses a Rust enum shape, NOT the spec's `{ mode, tool_name }` record.
/// This is intentional — provider adapters translate to/from wire format.
/// This test documents the actual serialized shapes for clarity.
#[test]
fn tool_choice_serialized_shape() -> Result<(), Box<dyn std::error::Error>> {
    // Simple variants serialize as lowercase strings
    assert_eq!(serde_json::to_string(&ToolChoice::Auto)?, r#""auto""#);
    assert_eq!(serde_json::to_string(&ToolChoice::None)?, r#""none""#);
    assert_eq!(
        serde_json::to_string(&ToolChoice::Required)?,
        r#""required""#
    );
    // Named variant serializes as {"tool":"name"} — not spec's {mode:"named",tool_name:"name"}
    // Adapters are responsible for translating to provider wire format
    assert_eq!(
        serde_json::to_string(&ToolChoice::Tool("get_weather".into()))?,
        r#"{"tool":"get_weather"}"#
    );
    Ok(())
}

// ── ResponseFormat ────────────────────────────────────────────────────

#[test]
fn response_format_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let fmt = ResponseFormat {
        format_type: ResponseFormatType::JsonSchema,
        json_schema: Some(serde_json::json!({"type": "object"})),
        strict: true,
    };
    let json = serde_json::to_string(&fmt)?;
    let back: ResponseFormat = serde_json::from_str(&json)?;
    assert_eq!(back, fmt);
    Ok(())
}

// ── Warning ───────────────────────────────────────────────────────────

#[test]
fn warning_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let w = Warning {
        message: "Deprecated model".into(),
        code: Some("model_deprecated".into()),
    };
    let json = serde_json::to_string(&w)?;
    let back: Warning = serde_json::from_str(&json)?;
    assert_eq!(back, w);
    Ok(())
}

// ── RateLimitInfo ─────────────────────────────────────────────────────

#[test]
fn rate_limit_info_default_is_all_none() {
    let rl = RateLimitInfo::default();
    assert!(rl.requests_remaining.is_none());
    assert!(rl.tokens_remaining.is_none());
}

// ── StreamEvent ───────────────────────────────────────────────────────

#[test]
fn stream_event_text_delta() -> Result<(), Box<dyn std::error::Error>> {
    let evt = StreamEvent::text_delta("Hello");
    assert_eq!(evt.event_type, StreamEventType::TextDelta);
    assert_eq!(evt.delta.as_deref(), Some("Hello"));
    let json = serde_json::to_string(&evt)?;
    let back: StreamEvent = serde_json::from_str(&json)?;
    assert_eq!(back, evt);
    Ok(())
}

#[test]
fn stream_event_finish() -> Result<(), Box<dyn std::error::Error>> {
    let evt = StreamEvent::finish(
        FinishReason::new(Reason::Stop, None),
        Usage {
            input_tokens: 10,
            output_tokens: 20,
            total_tokens: 30,
            ..Usage::default()
        },
    );
    assert_eq!(evt.event_type, StreamEventType::Finish);
    assert!(evt.finish_reason.is_some());
    assert!(evt.usage.is_some());
    let json = serde_json::to_string(&evt)?;
    let back: StreamEvent = serde_json::from_str(&json)?;
    assert_eq!(back, evt);
    Ok(())
}

// ── Message serde round-trip ──────────────────────────────────────────

#[test]
fn message_serde_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let msg = Message::user("Hello, world!");
    let json = serde_json::to_string(&msg)?;
    let back: Message = serde_json::from_str(&json)?;
    assert_eq!(back, msg);
    Ok(())
}

#[test]
fn message_with_tool_result_serde() -> Result<(), Box<dyn std::error::Error>> {
    let msg = Message::tool_result_text("call_1", "the result", false);
    let json = serde_json::to_string(&msg)?;
    let back: Message = serde_json::from_str(&json)?;
    assert_eq!(back, msg);
    Ok(())
}

// ── Response::reasoning() returns None vs Some ───────────────────────

#[test]
fn response_reasoning_none_when_absent() {
    let resp = make_response(vec![ContentPart::text("just text")]);
    assert_eq!(resp.reasoning(), None);
}

#[test]
fn response_reasoning_some_when_present() {
    let resp = make_response(vec![ContentPart::Thinking {
        thinking: ThinkingData {
            text: "thinking".into(),
            signature: None,
            redacted: false,
        },
    }]);
    assert_eq!(resp.reasoning(), Some("thinking".into()));
}

// ── ToolCall unified type ─────────────────────────────────────────────

#[test]
fn tool_call_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let tc = ToolCall {
        id: "call_1".into(),
        name: "get_weather".into(),
        arguments: serde_json::json!({"city": "NYC"}),
        raw_arguments: Some("{\"city\": \"NYC\"}".into()),
        parse_error: None,
    };
    let json = serde_json::to_string(&tc)?;
    let back: ToolCall = serde_json::from_str(&json)?;
    assert_eq!(back, tc);
    Ok(())
}

#[test]
fn tool_result_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let tr = ToolResult {
        tool_call_id: "call_1".into(),
        content: serde_json::json!({"temperature": 72}),
        is_error: false,
    };
    let json = serde_json::to_string(&tr)?;
    let back: ToolResult = serde_json::from_str(&json)?;
    assert_eq!(back, tr);
    Ok(())
}

#[test]
fn tool_result_error_flag() -> Result<(), Box<dyn std::error::Error>> {
    let tr = ToolResult {
        tool_call_id: "call_2".into(),
        content: serde_json::Value::String("connection refused".into()),
        is_error: true,
    };
    let json = serde_json::to_string(&tr)?;
    let back: ToolResult = serde_json::from_str(&json)?;
    assert!(back.is_error);
    Ok(())
}

#[test]
fn response_tool_calls_returns_unified_tool_call() {
    let resp = make_response(vec![ContentPart::tool_call(
        "id1",
        "get_weather",
        serde_json::json!({"city": "NYC"}),
    )]);
    let calls = resp.tool_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "get_weather");
    assert_eq!(calls[0].arguments, serde_json::json!({"city": "NYC"}));
    assert!(calls[0].raw_arguments.is_none());
    assert!(calls[0].parse_error.is_none());
}

#[test]
fn response_tool_calls_parses_string_arguments() {
    let resp = make_response(vec![ContentPart::ToolCall {
        tool_call: ToolCallData {
            id: "id1".into(),
            name: "get_weather".into(),
            arguments: serde_json::Value::String("{\"city\":\"NYC\"}".into()),
            call_type: "function".into(),
        },
    }]);
    let calls = resp.tool_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].arguments, serde_json::json!({"city": "NYC"}));
    assert_eq!(
        calls[0].raw_arguments.as_deref(),
        Some("{\"city\":\"NYC\"}")
    );
    assert!(calls[0].parse_error.is_none());
}

#[test]
fn response_tool_calls_surfaces_parse_error_for_malformed_json() {
    let resp = make_response(vec![ContentPart::ToolCall {
        tool_call: ToolCallData {
            id: "id1".into(),
            name: "get_weather".into(),
            arguments: serde_json::Value::String("not valid json{".into()),
            call_type: "function".into(),
        },
    }]);
    let calls = resp.tool_calls();
    assert_eq!(calls.len(), 1);
    // arguments should be the raw string, not {}
    assert_eq!(
        calls[0].arguments,
        serde_json::Value::String("not valid json{".into())
    );
    assert!(
        calls[0].parse_error.is_some(),
        "parse_error should be set for malformed JSON"
    );
    assert_eq!(calls[0].raw_arguments.as_deref(), Some("not valid json{"));
}

// ── ToolCallData.call_type ────────────────────────────────────────────

#[test]
fn tool_call_data_default_type_is_function() {
    let tc = ToolCallData {
        id: "id1".into(),
        name: "test".into(),
        arguments: serde_json::json!({}),
        call_type: "function".into(),
    };
    assert_eq!(tc.call_type, "function");
}

#[test]
fn tool_call_data_type_omitted_in_json_when_default() -> Result<(), Box<dyn std::error::Error>> {
    let tc = ToolCallData {
        id: "id1".into(),
        name: "test".into(),
        arguments: serde_json::json!({}),
        call_type: "function".into(),
    };
    let json = serde_json::to_string(&tc)?;
    assert!(!json.contains("\"type\""), "default type should be omitted");
    Ok(())
}

#[test]
fn tool_call_data_custom_type_round_trips() -> Result<(), Box<dyn std::error::Error>> {
    let tc = ToolCallData {
        id: "id1".into(),
        name: "test".into(),
        arguments: serde_json::json!({}),
        call_type: "custom".into(),
    };
    let json = serde_json::to_string(&tc)?;
    assert!(json.contains("\"type\":\"custom\""));
    let back: ToolCallData = serde_json::from_str(&json)?;
    assert_eq!(back.call_type, "custom");
    Ok(())
}

#[test]
fn tool_call_data_deserializes_without_type_field() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{"id":"id1","name":"test","arguments":{}}"#;
    let tc: ToolCallData = serde_json::from_str(json)?;
    assert_eq!(tc.call_type, "function");
    Ok(())
}

// ── StreamEvent error payload ─────────────────────────────────────────

#[test]
fn stream_event_error_carries_sdk_error() -> Result<(), Box<dyn std::error::Error>> {
    let err = SdkError::Server {
        message: "internal error".into(),
        details: ProviderDetails::default(),
    };
    let evt = StreamEvent::error(err.clone());
    assert_eq!(evt.event_type, StreamEventType::Error);
    assert_eq!(evt.error, Some(err));
    let json = serde_json::to_string(&evt)?;
    let back: StreamEvent = serde_json::from_str(&json)?;
    assert_eq!(back, evt);
    Ok(())
}

// ── Extension content kind (forward compatibility) ────────────────────

#[test]
fn extension_content_kind_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    // Simulate a future content kind not yet in the enum
    let json = r#"{"kind":"hologram","data":"xyz"}"#;
    let part: ContentPart = serde_json::from_str(json)?;
    match &part {
        ContentPart::Extension(v) => {
            assert_eq!(v.get("kind").and_then(|v| v.as_str()), Some("hologram"));
        }
        other => return Err(format!("expected Extension, got {other:?}").into()),
    }
    // Round-trip preserves the data
    let back_json = serde_json::to_string(&part)?;
    let back: ContentPart = serde_json::from_str(&back_json)?;
    assert_eq!(back, part);
    Ok(())
}

// ── ContentPart::validate catches known-kind-as-Extension ─────────────

#[test]
fn content_part_validate_rejects_known_kind_as_extension() -> Result<(), Box<dyn std::error::Error>>
{
    // {"kind":"text"} without the required "text" field falls through to Extension
    let json = r#"{"kind":"text"}"#;
    let part: ContentPart = serde_json::from_str(json)?;
    assert!(
        matches!(part, ContentPart::Extension(_)),
        "malformed 'text' should deserialize as Extension"
    );
    assert!(
        part.validate().is_err(),
        "validate() should reject known kind in Extension"
    );
    Ok(())
}

#[test]
fn content_part_validate_allows_unknown_extension() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#"{"kind":"hologram","data":"xyz"}"#;
    let part: ContentPart = serde_json::from_str(json)?;
    assert!(
        matches!(part, ContentPart::Extension(_)),
        "unknown kind should deserialize as Extension"
    );
    part.validate()?; // should pass — truly unknown kind
    Ok(())
}

#[test]
fn content_part_validate_ok_for_well_formed_known_kinds() -> Result<(), Box<dyn std::error::Error>>
{
    let part = ContentPart::text("hello");
    part.validate()?;
    Ok(())
}

// ── StreamEventType forward compatibility ─────────────────────────────

#[test]
fn stream_event_type_unknown_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#""custom_provider_event""#;
    let evt_type: StreamEventType = serde_json::from_str(json)?;
    assert_eq!(
        evt_type,
        StreamEventType::Unknown("custom_provider_event".into())
    );
    let back = serde_json::to_string(&evt_type)?;
    assert_eq!(back, json);
    Ok(())
}

#[test]
fn stream_event_type_known_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let json = r#""text_delta""#;
    let evt_type: StreamEventType = serde_json::from_str(json)?;
    assert_eq!(evt_type, StreamEventType::TextDelta);
    let back = serde_json::to_string(&evt_type)?;
    assert_eq!(back, json);
    Ok(())
}

// ── ImageData validation ──────────────────────────────────────────────

#[test]
fn image_data_validate_url_only() {
    let img = ImageData {
        url: Some("https://example.com/img.png".into()),
        data: None,
        media_type: None,
        detail: None,
    };
    assert!(img.validate().is_ok());
}

#[test]
fn image_data_validate_data_only() {
    let img = ImageData {
        url: None,
        data: Some(vec![0x89, 0x50]),
        media_type: Some("image/png".into()),
        detail: None,
    };
    assert!(img.validate().is_ok());
}

#[test]
fn image_data_validate_both_set_is_error() {
    let img = ImageData {
        url: Some("https://example.com/img.png".into()),
        data: Some(vec![0x89]),
        media_type: None,
        detail: None,
    };
    assert!(img.validate().is_err());
}

#[test]
fn image_data_validate_neither_set_is_error() {
    let img = ImageData {
        url: None,
        data: None,
        media_type: None,
        detail: None,
    };
    assert!(img.validate().is_err());
}

#[test]
fn image_data_effective_media_type_defaults_to_png() {
    let img = ImageData {
        url: None,
        data: Some(vec![0x89]),
        media_type: None,
        detail: None,
    };
    assert_eq!(img.effective_media_type(), Some("image/png"));
}

#[test]
fn image_data_effective_media_type_uses_explicit() {
    let img = ImageData {
        url: None,
        data: Some(vec![0xFF]),
        media_type: Some("image/jpeg".into()),
        detail: None,
    };
    assert_eq!(img.effective_media_type(), Some("image/jpeg"));
}

#[test]
fn image_data_effective_media_type_none_for_url() {
    let img = ImageData {
        url: Some("https://example.com/img.png".into()),
        data: None,
        media_type: None,
        detail: None,
    };
    assert_eq!(img.effective_media_type(), None);
}

// ── AudioData serde ───────────────────────────────────────────────────

#[test]
fn audio_data_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let audio = AudioData {
        url: Some("https://example.com/audio.wav".into()),
        data: None,
        media_type: Some("audio/wav".into()),
    };
    let json = serde_json::to_string(&audio)?;
    let back: AudioData = serde_json::from_str(&json)?;
    assert_eq!(back, audio);
    Ok(())
}

// ── DocumentData serde ────────────────────────────────────────────────

#[test]
fn document_data_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let doc = DocumentData {
        url: None,
        data: Some(vec![0x25, 0x50, 0x44, 0x46]), // %PDF
        media_type: Some("application/pdf".into()),
        file_name: Some("report.pdf".into()),
    };
    let json = serde_json::to_string(&doc)?;
    let back: DocumentData = serde_json::from_str(&json)?;
    assert_eq!(back, doc);
    Ok(())
}

// ── Timeout serde ─────────────────────────────────────────────────────

#[test]
fn timeout_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let timeout = Timeout {
        request: Some(30.0),
        connect: Some(5.0),
        stream_idle: None,
    };
    let json = serde_json::to_string(&timeout)?;
    let back: Timeout = serde_json::from_str(&json)?;
    assert_eq!(back, timeout);
    Ok(())
}

// ── ProviderDetails.retryable field ───────────────────────────────────

#[test]
fn provider_details_retryable_serialized() -> Result<(), Box<dyn std::error::Error>> {
    let err =
        SdkError::from_status_code(429, "rate limited", Some("openai".into()), None, None, None);
    let json = serde_json::to_string(&err)?;
    assert!(
        json.contains("\"retryable\":true"),
        "retryable should be in JSON"
    );
    Ok(())
}

#[test]
fn provider_details_retryable_false_for_client_errors() -> Result<(), Box<dyn std::error::Error>> {
    let err = SdkError::from_status_code(401, "unauthorized", None, None, None, None);
    let json = serde_json::to_string(&err)?;
    assert!(
        json.contains("\"retryable\":false"),
        "retryable should be false"
    );
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────

fn make_response(content: Vec<ContentPart>) -> Response {
    Response {
        id: "resp_1".into(),
        model: "test-model".into(),
        provider: "test".into(),
        message: Message::new(Role::Assistant, content),
        finish_reason: FinishReason::new(Reason::Stop, None),
        usage: Usage::default(),
        raw: None,
        warnings: None,
        rate_limit: None,
    }
}
