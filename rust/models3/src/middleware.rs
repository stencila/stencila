use crate::error::SdkResult;
use crate::provider::{BoxFuture, BoxStream};
use crate::types::request::Request;
use crate::types::response::Response;
use crate::types::stream_event::StreamEvent;

/// Callback for the next step in the `complete()` middleware chain.
pub type NextComplete<'a> =
    Box<dyn FnOnce(Request) -> BoxFuture<'a, SdkResult<Response>> + Send + 'a>;

/// Callback for the next step in the `stream()` middleware chain.
pub type NextStream<'a> = Box<
    dyn FnOnce(Request) -> BoxFuture<'a, SdkResult<BoxStream<'a, SdkResult<StreamEvent>>>>
        + Send
        + 'a,
>;

/// Middleware for cross-cutting concerns on Client requests.
///
/// Wraps both `complete()` and `stream()` calls. Each method has a
/// default pass-through implementation, so implementors only override
/// the methods they need.
///
/// ## Execution order
///
/// Middleware runs in registration order for the request phase
/// (first registered = first to execute) and in reverse order for the
/// response phase. This is the standard onion / chain-of-responsibility
/// pattern.
pub trait Middleware: Send + Sync {
    /// Wrap a `complete()` call.
    ///
    /// Default: pass through to the next handler.
    fn handle_complete<'a>(
        &'a self,
        request: Request,
        next: NextComplete<'a>,
    ) -> BoxFuture<'a, SdkResult<Response>> {
        next(request)
    }

    /// Wrap a `stream()` call.
    ///
    /// Default: pass through to the next handler.
    fn handle_stream<'a>(
        &'a self,
        request: Request,
        next: NextStream<'a>,
    ) -> BoxFuture<'a, SdkResult<BoxStream<'a, SdkResult<StreamEvent>>>> {
        next(request)
    }
}
