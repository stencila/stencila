use serde::{Deserialize, Serialize};

/// A non-fatal warning from the SDK or provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Warning {
    /// Human-readable description of the warning.
    pub message: String,
    /// Machine-readable warning code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}
