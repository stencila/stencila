use serde::{Deserialize, Serialize};

/// Why the model stopped generating.
///
/// Carries both a unified reason (for portable code) and the provider's
/// raw finish reason string (for debugging / passthrough).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinishReason {
    /// Unified reason value.
    pub reason: Reason,
    /// Provider's native finish reason string, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<String>,
}

impl FinishReason {
    /// Create a finish reason with a unified reason and optional raw value.
    #[must_use]
    pub fn new(reason: Reason, raw: Option<String>) -> Self {
        Self { reason, raw }
    }
}

/// Unified finish reason values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    /// Natural end of generation (model stopped).
    Stop,
    /// Output reached `max_tokens` limit.
    Length,
    /// Model wants to invoke tools.
    ToolCalls,
    /// Response blocked by safety/content filter.
    ContentFilter,
    /// Error occurred during generation.
    Error,
    /// Provider-specific reason not mapped above.
    Other,
}
