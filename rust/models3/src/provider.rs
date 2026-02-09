use std::pin::Pin;

use futures::Stream;

use crate::catalog::ModelInfo;
use crate::error::SdkResult;
use crate::types::request::Request;
use crate::types::response::Response;
use crate::types::stream_event::StreamEvent;
use crate::types::tool::ToolChoice;

/// A boxed future that is Send.
pub type BoxFuture<'a, T> = Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

/// A boxed stream that is Send.
pub type BoxStream<'a, T> = Pin<Box<dyn Stream<Item = T> + Send + 'a>>;

/// The core trait that each LLM provider adapter must implement.
///
/// Object-safe: uses boxed futures/streams instead of async fn.
pub trait ProviderAdapter: Send + Sync {
    /// The provider's identifier (e.g. "anthropic", "openai", "gemini").
    fn name(&self) -> &str;

    /// Send a request and return a complete response.
    fn complete(&self, request: Request) -> BoxFuture<'_, SdkResult<Response>>;

    /// Send a request and return a stream of events.
    ///
    /// The return type is `Future<Result<Stream<Result<StreamEvent>>>>`:
    /// - The outer `Future` resolves once the HTTP connection is established.
    ///   A failure here (e.g. DNS, TLS, auth rejection) is returned as an `Err`
    ///   before any events are produced.
    /// - The inner `Stream` yields individual server-sent events. Per-chunk
    ///   errors (e.g. mid-stream disconnect) appear as `Err` items in the stream.
    ///
    /// This two-phase design lets callers distinguish connection-time failures
    /// from streaming failures — the spec's `stream()` returns a single stream,
    /// but Rust's ownership and async model benefits from the explicit split.
    fn stream(
        &self,
        request: Request,
    ) -> BoxFuture<'_, SdkResult<BoxStream<'_, SdkResult<StreamEvent>>>>;

    /// Perform any cleanup (close connections, flush buffers).
    /// Default: no-op.
    fn close(&self) -> BoxFuture<'_, SdkResult<()>> {
        Box::pin(async { Ok(()) })
    }

    /// One-time initialization (validate credentials, warm caches).
    /// Default: no-op.
    fn initialize(&self) -> BoxFuture<'_, SdkResult<()>> {
        Box::pin(async { Ok(()) })
    }

    /// Whether the provider supports a given tool choice mode.
    /// Default: true for all choices.
    fn supports_tool_choice(&self, _choice: &ToolChoice) -> bool {
        true
    }

    /// List models available from this provider.
    ///
    /// Calls the provider's native model-listing endpoint and returns
    /// basic `ModelInfo` for each discovered model. Fields not available
    /// from the listing API are filled with defaults.
    ///
    /// Default: returns an empty list (provider doesn't support listing).
    fn list_models(&self) -> BoxFuture<'_, SdkResult<Vec<ModelInfo>>> {
        Box::pin(async { Ok(vec![]) })
    }
}
