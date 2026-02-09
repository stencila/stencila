//! Shared integration-test helpers live here.
//!
//! Keep helpers deterministic and free of real network calls.
#![allow(dead_code)]

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use stencila_models3::error::{SdkError, SdkResult};
use stencila_models3::provider::{BoxFuture, BoxStream, ProviderAdapter};
use stencila_models3::types::content::ContentPart;
use stencila_models3::types::finish_reason::{FinishReason, Reason};
use stencila_models3::types::message::Message;
use stencila_models3::types::request::Request;
use stencila_models3::types::response::Response;
use stencila_models3::types::stream_event::StreamEvent;
use stencila_models3::types::usage::Usage;

/// A mock provider adapter that returns a fixed response.
pub struct MockAdapter {
    provider_name: &'static str,
    response: Response,
}

impl MockAdapter {
    /// Create a mock adapter that returns the given response for every request.
    pub fn new(provider_name: &'static str, response: Response) -> Self {
        Self {
            provider_name,
            response,
        }
    }

    /// Create a mock adapter with a simple text response.
    pub fn with_text(provider_name: &'static str, text: &str) -> Self {
        Self::new(provider_name, make_response(provider_name, text))
    }
}

impl ProviderAdapter for MockAdapter {
    #[allow(clippy::unnecessary_literal_bound)]
    fn name(&self) -> &str {
        self.provider_name
    }

    fn complete(&self, _request: Request) -> BoxFuture<'_, SdkResult<Response>> {
        let resp = self.response.clone();
        Box::pin(async move { Ok(resp) })
    }

    fn stream(
        &self,
        _request: Request,
    ) -> BoxFuture<'_, SdkResult<BoxStream<'_, SdkResult<StreamEvent>>>> {
        let events = response_to_stream_events(&self.response);
        Box::pin(async {
            Ok(Box::pin(futures::stream::iter(events)) as BoxStream<'_, SdkResult<StreamEvent>>)
        })
    }
}

/// A mock adapter that always returns an error.
pub struct ErrorAdapter {
    provider_name: &'static str,
    error: SdkError,
}

impl ErrorAdapter {
    pub fn new(provider_name: &'static str, error: SdkError) -> Self {
        Self {
            provider_name,
            error,
        }
    }
}

impl ProviderAdapter for ErrorAdapter {
    #[allow(clippy::unnecessary_literal_bound)]
    fn name(&self) -> &str {
        self.provider_name
    }

    fn complete(&self, _request: Request) -> BoxFuture<'_, SdkResult<Response>> {
        let err = self.error.clone();
        Box::pin(async move { Err(err) })
    }

    fn stream(
        &self,
        _request: Request,
    ) -> BoxFuture<'_, SdkResult<BoxStream<'_, SdkResult<StreamEvent>>>> {
        let err = self.error.clone();
        Box::pin(async move { Err(err) })
    }

    fn list_models(&self) -> BoxFuture<'_, SdkResult<Vec<stencila_models3::catalog::ModelInfo>>> {
        let err = self.error.clone();
        Box::pin(async move { Err(err) })
    }
}

/// Create a simple test response.
pub fn make_response(provider: &str, text: &str) -> Response {
    Response {
        id: "test-id".into(),
        model: "test-model".into(),
        provider: provider.into(),
        message: Message::assistant(text),
        finish_reason: FinishReason::new(Reason::Stop, None),
        usage: Usage::default(),
        raw: None,
        warnings: None,
        rate_limit: None,
    }
}

/// Create a response with tool calls.
pub fn make_tool_call_response(
    provider: &str,
    calls: Vec<(&str, &str, serde_json::Value)>,
) -> Response {
    let content: Vec<ContentPart> = calls
        .into_iter()
        .map(|(id, name, args)| ContentPart::tool_call(id, name, args))
        .collect();

    Response {
        id: "test-id".into(),
        model: "test-model".into(),
        provider: provider.into(),
        message: Message::new(stencila_models3::types::role::Role::Assistant, content),
        finish_reason: FinishReason::new(Reason::ToolCalls, None),
        usage: Usage {
            input_tokens: 10,
            output_tokens: 5,
            total_tokens: 15,
            ..Usage::default()
        },
        raw: None,
        warnings: None,
        rate_limit: None,
    }
}

/// A mock adapter that returns tool calls on the first request,
/// then a text response on subsequent requests.
pub struct ToolCallAdapter {
    provider_name: &'static str,
    tool_call_response: Response,
    final_response: Response,
    call_count: Arc<AtomicU32>,
}

impl ToolCallAdapter {
    pub fn new(
        provider_name: &'static str,
        tool_call_response: Response,
        final_response: Response,
    ) -> Self {
        Self {
            provider_name,
            tool_call_response,
            final_response,
            call_count: Arc::new(AtomicU32::new(0)),
        }
    }

    /// How many times complete() was called.
    pub fn call_count(&self) -> u32 {
        self.call_count.load(Ordering::SeqCst)
    }

    /// Get a clone of the call counter for assertions.
    pub fn call_counter(&self) -> Arc<AtomicU32> {
        self.call_count.clone()
    }
}

impl ProviderAdapter for ToolCallAdapter {
    #[allow(clippy::unnecessary_literal_bound)]
    fn name(&self) -> &str {
        self.provider_name
    }

    fn complete(&self, _request: Request) -> BoxFuture<'_, SdkResult<Response>> {
        let n = self.call_count.fetch_add(1, Ordering::SeqCst);
        let resp = if n == 0 {
            self.tool_call_response.clone()
        } else {
            self.final_response.clone()
        };
        Box::pin(async move { Ok(resp) })
    }

    fn stream(
        &self,
        _request: Request,
    ) -> BoxFuture<'_, SdkResult<BoxStream<'_, SdkResult<StreamEvent>>>> {
        let n = self.call_count.fetch_add(1, Ordering::SeqCst);
        let resp = if n == 0 {
            &self.tool_call_response
        } else {
            &self.final_response
        };
        let events = response_to_stream_events(resp);
        Box::pin(async {
            Ok(Box::pin(futures::stream::iter(events)) as BoxStream<'_, SdkResult<StreamEvent>>)
        })
    }
}

/// Convert a Response into stream events for mock streaming.
pub fn response_to_stream_events(resp: &Response) -> Vec<SdkResult<StreamEvent>> {
    use stencila_models3::types::stream_event::StreamEventType;
    use stencila_models3::types::tool::ToolCall;

    let mut events = vec![Ok(StreamEvent::stream_start())];

    for part in &resp.message.content {
        match part {
            ContentPart::Text { text } => {
                events.push(Ok(StreamEvent::text_delta(text)));
            }
            ContentPart::ToolCall { tool_call } => {
                let tc = ToolCall {
                    id: tool_call.id.clone(),
                    name: tool_call.name.clone(),
                    arguments: tool_call.arguments.clone(),
                    raw_arguments: None,
                    parse_error: None,
                };
                events.push(Ok(StreamEvent::tool_call_event(
                    StreamEventType::ToolCallStart,
                    tc.clone(),
                    serde_json::Value::Null,
                )));
                events.push(Ok(StreamEvent::tool_call_event(
                    StreamEventType::ToolCallEnd,
                    tc,
                    serde_json::Value::Null,
                )));
            }
            _ => {}
        }
    }

    events.push(Ok(StreamEvent::finish(
        resp.finish_reason.clone(),
        resp.usage.clone(),
    )));

    events
}

/// A mock provider adapter that returns a fixed response and a list of models.
pub struct ModelListingAdapter {
    provider_name: &'static str,
    response: Response,
    models: Vec<stencila_models3::catalog::ModelInfo>,
}

impl ModelListingAdapter {
    pub fn new(
        provider_name: &'static str,
        models: Vec<stencila_models3::catalog::ModelInfo>,
    ) -> Self {
        Self {
            provider_name,
            response: make_response(provider_name, "ok"),
            models,
        }
    }
}

impl ProviderAdapter for ModelListingAdapter {
    #[allow(clippy::unnecessary_literal_bound)]
    fn name(&self) -> &str {
        self.provider_name
    }

    fn complete(&self, _request: Request) -> BoxFuture<'_, SdkResult<Response>> {
        let resp = self.response.clone();
        Box::pin(async move { Ok(resp) })
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

    fn list_models(&self) -> BoxFuture<'_, SdkResult<Vec<stencila_models3::catalog::ModelInfo>>> {
        let models = self.models.clone();
        Box::pin(async move { Ok(models) })
    }
}

/// A mock adapter whose stream yields some events then an error.
///
/// Used to test that mid-stream errors become error events (spec §6.6).
pub struct MidStreamErrorAdapter {
    provider_name: &'static str,
    error: SdkError,
}

impl MidStreamErrorAdapter {
    pub fn new(provider_name: &'static str, error: SdkError) -> Self {
        Self {
            provider_name,
            error,
        }
    }
}

impl ProviderAdapter for MidStreamErrorAdapter {
    #[allow(clippy::unnecessary_literal_bound)]
    fn name(&self) -> &str {
        self.provider_name
    }

    fn complete(&self, _request: Request) -> BoxFuture<'_, SdkResult<Response>> {
        Box::pin(async {
            Err(SdkError::Configuration {
                message: "not implemented".into(),
            })
        })
    }

    fn stream(
        &self,
        _request: Request,
    ) -> BoxFuture<'_, SdkResult<BoxStream<'_, SdkResult<StreamEvent>>>> {
        let err = self.error.clone();
        let events: Vec<SdkResult<StreamEvent>> = vec![
            Ok(StreamEvent::stream_start()),
            Ok(StreamEvent::text_delta("partial ")),
            Err(err),
        ];
        Box::pin(async {
            Ok(Box::pin(futures::stream::iter(events)) as BoxStream<'_, SdkResult<StreamEvent>>)
        })
    }
}

/// A mock adapter that delays before returning a response.
///
/// Used to test timeout enforcement.
pub struct SlowAdapter {
    provider_name: &'static str,
    response: Response,
    delay: std::time::Duration,
}

impl SlowAdapter {
    pub fn new(
        provider_name: &'static str,
        response: Response,
        delay: std::time::Duration,
    ) -> Self {
        Self {
            provider_name,
            response,
            delay,
        }
    }
}

impl ProviderAdapter for SlowAdapter {
    #[allow(clippy::unnecessary_literal_bound)]
    fn name(&self) -> &str {
        self.provider_name
    }

    fn complete(&self, _request: Request) -> BoxFuture<'_, SdkResult<Response>> {
        let resp = self.response.clone();
        let delay = self.delay;
        Box::pin(async move {
            tokio::time::sleep(delay).await;
            Ok(resp)
        })
    }

    fn stream(
        &self,
        _request: Request,
    ) -> BoxFuture<'_, SdkResult<BoxStream<'_, SdkResult<StreamEvent>>>> {
        let resp = self.response.clone();
        let delay = self.delay;
        Box::pin(async move {
            tokio::time::sleep(delay).await;
            let events = response_to_stream_events(&resp);
            Ok(Box::pin(futures::stream::iter(events)) as BoxStream<'_, SdkResult<StreamEvent>>)
        })
    }
}

/// Create a simple test request.
pub fn make_request(model: &str) -> Request {
    Request::new(model, vec![Message::user("hello")])
}

/// Create a request targeting a specific provider.
pub fn make_request_for(model: &str, provider: &str) -> Request {
    let mut req = make_request(model);
    req.provider = Some(provider.into());
    req
}
