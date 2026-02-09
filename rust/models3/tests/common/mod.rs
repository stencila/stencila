//! Shared integration-test helpers live here.
//!
//! Keep helpers deterministic and free of real network calls.

use stencila_models3::error::{SdkError, SdkResult};
use stencila_models3::provider::{BoxFuture, BoxStream, ProviderAdapter};
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
        Box::pin(async {
            let events: Vec<SdkResult<StreamEvent>> = vec![
                Ok(StreamEvent::stream_start()),
                Ok(StreamEvent::text_delta("mock")),
                Ok(StreamEvent::finish(
                    FinishReason::new(Reason::Stop, None),
                    Usage::default(),
                )),
            ];
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
