use serde::{Deserialize, Serialize};

/// Timeout configuration for requests.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Timeout {
    /// Total request timeout in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<f64>,
    /// Timeout for initial connection in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connect: Option<f64>,
    /// Timeout between stream events in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_idle: Option<f64>,
}
