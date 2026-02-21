use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::error::SdkError;

/// Shared state between controller and signal.
#[derive(Debug)]
struct Inner {
    aborted: AtomicBool,
    notify: tokio::sync::Notify,
}

/// Controls cancellation of in-flight requests.
///
/// Create an `AbortController`, extract its [`AbortSignal`], and pass the
/// signal to `generate()` / `stream()`. Call [`abort()`](Self::abort) to
/// cancel the operation.
#[derive(Debug, Clone)]
pub struct AbortController {
    inner: Arc<Inner>,
}

impl Default for AbortController {
    fn default() -> Self {
        Self::new()
    }
}

impl AbortController {
    /// Create a new abort controller.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                aborted: AtomicBool::new(false),
                notify: tokio::sync::Notify::new(),
            }),
        }
    }

    /// Get the signal to pass to generation functions.
    #[must_use]
    pub fn signal(&self) -> AbortSignal {
        AbortSignal {
            inner: self.inner.clone(),
        }
    }

    /// Abort the operation associated with this controller's signal.
    pub fn abort(&self) {
        self.inner.aborted.store(true, Ordering::Release);
        self.inner.notify.notify_waiters();
    }
}

/// A signal that indicates an operation should be cancelled.
///
/// Obtained from [`AbortController::signal()`]. Check cancellation
/// via [`is_aborted()`](Self::is_aborted).
#[derive(Debug, Clone)]
pub struct AbortSignal {
    inner: Arc<Inner>,
}

impl AbortSignal {
    /// Whether the signal has been triggered.
    #[must_use]
    pub fn is_aborted(&self) -> bool {
        self.inner.aborted.load(Ordering::Acquire)
    }

    /// Check if aborted; if so, return `SdkError::Abort`.
    pub(crate) fn check(&self) -> Result<(), SdkError> {
        if self.is_aborted() {
            Err(SdkError::Abort {
                message: "operation was aborted".into(),
            })
        } else {
            Ok(())
        }
    }

    /// Wait until the signal is triggered.
    ///
    /// Useful with `tokio::select!` to race an abort against other futures.
    /// Returns immediately if already aborted.
    pub(crate) async fn notified(&self) {
        if self.is_aborted() {
            return;
        }
        self.inner.notify.notified().await;
    }
}
