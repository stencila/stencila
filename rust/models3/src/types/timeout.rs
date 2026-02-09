use serde::{Deserialize, Serialize};

/// Timeout configuration for requests.
///
/// Covers both adapter-level timeouts (`request`, `connect`, `stream_idle`)
/// and high-level API timeouts (`total`, `per_step`):
///
/// - **`total`**: Maximum time (seconds) for the entire multi-step operation
///   including tool execution loops and retries. Used by `generate()` and
///   `stream_generate()`. Spec §4.7.
/// - **`per_step`**: Maximum time (seconds) per individual LLM call within
///   the multi-step operation. Spec §4.7.
/// - **`request`**: Total request/response cycle timeout at the adapter level.
/// - **`connect`**: Time to establish the HTTP connection.
/// - **`stream_idle`**: Maximum time between consecutive stream events.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Timeout {
    /// Max time for the entire multi-step operation (seconds). Spec §4.7.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<f64>,
    /// Max time per individual LLM call (seconds). Spec §4.7.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_step: Option<f64>,
    /// Total request timeout in seconds (adapter-level).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<f64>,
    /// Timeout for initial connection in seconds (adapter-level).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connect: Option<f64>,
    /// Timeout between stream events in seconds (adapter-level).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_idle: Option<f64>,
}
