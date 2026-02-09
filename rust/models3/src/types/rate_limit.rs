use serde::{Deserialize, Serialize};

/// Rate limit information extracted from provider response headers.
///
/// All fields are optional — populated when the provider includes
/// rate limit headers in its response.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RateLimitInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requests_remaining: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requests_limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_remaining: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_limit: Option<u64>,
    /// Timestamp (seconds since epoch) when rate limits reset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reset_at: Option<f64>,
}
