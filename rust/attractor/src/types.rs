use std::fmt;
use std::str::FromStr;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::error::{AttractorError, AttractorResult};

/// The outcome status of a pipeline stage execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StageStatus {
    /// The stage completed successfully.
    Success,
    /// The stage failed.
    Fail,
    /// The stage partially succeeded.
    PartialSuccess,
    /// The stage should be retried.
    Retry,
    /// The stage was skipped.
    Skipped,
}

impl StageStatus {
    /// Whether this status represents a successful outcome.
    #[must_use]
    pub fn is_success(self) -> bool {
        matches!(self, Self::Success | Self::PartialSuccess)
    }

    /// Return the `snake_case` string representation matching serde serialization.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Fail => "fail",
            Self::PartialSuccess => "partial_success",
            Self::Retry => "retry",
            Self::Skipped => "skipped",
        }
    }
}

/// The result of executing a pipeline stage.
///
/// Field names use serde renames to match the Appendix C file interop contract:
/// the Rust field `status` serializes as `"outcome"`, and `preferred_label`
/// serializes as `"preferred_next_label"`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Outcome {
    /// The outcome status of the stage.
    #[serde(rename = "outcome")]
    pub status: StageStatus,

    /// A preferred label for the next transition edge.
    #[serde(
        rename = "preferred_next_label",
        alias = "preferred_label",
        default,
        skip_serializing_if = "String::is_empty"
    )]
    pub preferred_label: String,

    /// Suggested node IDs to transition to next.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suggested_next_ids: Vec<String>,

    /// Key-value updates to apply to the pipeline context.
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub context_updates: IndexMap<String, serde_json::Value>,

    /// Free-form notes about this outcome.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub notes: String,

    /// The reason for failure (populated when status is `Fail`).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub failure_reason: String,
}

impl Outcome {
    /// Create a successful outcome with default fields.
    #[must_use]
    pub fn success() -> Self {
        Self {
            status: StageStatus::Success,
            preferred_label: String::new(),
            suggested_next_ids: Vec::new(),
            context_updates: IndexMap::new(),
            notes: String::new(),
            failure_reason: String::new(),
        }
    }

    /// Create a failed outcome with the given reason.
    #[must_use]
    pub fn fail(reason: impl Into<String>) -> Self {
        Self {
            status: StageStatus::Fail,
            failure_reason: reason.into(),
            preferred_label: String::new(),
            suggested_next_ids: Vec::new(),
            context_updates: IndexMap::new(),
            notes: String::new(),
        }
    }

    /// Create a retry outcome with the given reason.
    #[must_use]
    pub fn retry(reason: impl Into<String>) -> Self {
        Self {
            status: StageStatus::Retry,
            failure_reason: reason.into(),
            preferred_label: String::new(),
            suggested_next_ids: Vec::new(),
            context_updates: IndexMap::new(),
            notes: String::new(),
        }
    }

    /// Create a skipped outcome.
    #[must_use]
    pub fn skipped() -> Self {
        Self {
            status: StageStatus::Skipped,
            preferred_label: String::new(),
            suggested_next_ids: Vec::new(),
            context_updates: IndexMap::new(),
            notes: String::new(),
            failure_reason: String::new(),
        }
    }
}

/// Controls how context values are compressed or summarized for LLM prompts.
///
/// The colon-separated variants (`summary:low`, `summary:medium`, `summary:high`)
/// use custom serde to handle the non-standard naming.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FidelityMode {
    /// Include the full value without modification.
    Full,
    /// Truncate long values to a length limit.
    Truncate,
    /// Use a compact representation.
    Compact,
    /// Low-detail summary.
    SummaryLow,
    /// Medium-detail summary.
    SummaryMedium,
    /// High-detail summary.
    SummaryHigh,
}

impl Default for FidelityMode {
    fn default() -> Self {
        Self::Compact
    }
}

impl fmt::Display for FidelityMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Full => "full",
            Self::Truncate => "truncate",
            Self::Compact => "compact",
            Self::SummaryLow => "summary:low",
            Self::SummaryMedium => "summary:medium",
            Self::SummaryHigh => "summary:high",
        };
        f.write_str(s)
    }
}

impl FromStr for FidelityMode {
    type Err = AttractorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "full" => Ok(Self::Full),
            "truncate" => Ok(Self::Truncate),
            "compact" => Ok(Self::Compact),
            "summary:low" => Ok(Self::SummaryLow),
            "summary:medium" => Ok(Self::SummaryMedium),
            "summary:high" => Ok(Self::SummaryHigh),
            other => Err(AttractorError::InvalidPipeline {
                reason: format!("unknown fidelity mode: {other}"),
            }),
        }
    }
}

impl Serialize for FidelityMode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for FidelityMode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// Controls the reasoning effort level for LLM calls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    /// Minimal reasoning.
    Low,
    /// Balanced reasoning.
    Medium,
    /// Maximum reasoning depth.
    High,
}

impl Default for ReasoningEffort {
    fn default() -> Self {
        Self::High
    }
}

impl ReasoningEffort {
    /// Return the string representation.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

/// A duration parsed from specification format strings like `"250ms"`, `"15m"`, `"2h"`.
///
/// Wraps [`std::time::Duration`] with spec-compatible parsing and display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Duration(std::time::Duration);

impl Duration {
    /// Parse a duration from a specification string.
    ///
    /// Supported suffixes: `ms` (milliseconds), `s` (seconds), `m` (minutes),
    /// `h` (hours), `d` (days).
    ///
    /// # Errors
    ///
    /// Returns [`AttractorError::InvalidPipeline`] if the string cannot be parsed.
    pub fn from_spec_str(s: &str) -> AttractorResult<Self> {
        let (value, unit) = if let Some(rest) = s.strip_suffix("ms") {
            (rest, "ms")
        } else if let Some(rest) = s.strip_suffix('s') {
            (rest, "s")
        } else if let Some(rest) = s.strip_suffix('m') {
            (rest, "m")
        } else if let Some(rest) = s.strip_suffix('h') {
            (rest, "h")
        } else if let Some(rest) = s.strip_suffix('d') {
            (rest, "d")
        } else {
            return Err(AttractorError::InvalidPipeline {
                reason: format!("invalid duration string: {s}"),
            });
        };

        let n: u64 = value.parse().map_err(|_| AttractorError::InvalidPipeline {
            reason: format!("invalid duration number: {s}"),
        })?;

        let millis = match unit {
            "ms" => Some(n),
            "s" => n.checked_mul(1_000),
            "m" => n.checked_mul(60_000),
            "h" => n.checked_mul(3_600_000),
            "d" => n.checked_mul(86_400_000),
            _ => unreachable!(),
        };

        let millis = millis.ok_or_else(|| AttractorError::InvalidPipeline {
            reason: format!("duration overflow: {s}"),
        })?;

        Ok(Self(std::time::Duration::from_millis(millis)))
    }

    /// Return the inner [`std::time::Duration`].
    #[must_use]
    pub fn inner(self) -> std::time::Duration {
        self.0
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ms = self.0.as_millis();
        if ms == 0 {
            return f.write_str("0ms");
        }

        let ms_u64 = u64::try_from(ms).unwrap_or(u64::MAX);

        if ms_u64 % 86_400_000 == 0 {
            write!(f, "{}d", ms_u64 / 86_400_000)
        } else if ms_u64 % 3_600_000 == 0 {
            write!(f, "{}h", ms_u64 / 3_600_000)
        } else if ms_u64 % 60_000 == 0 {
            write!(f, "{}m", ms_u64 / 60_000)
        } else if ms_u64 % 1_000 == 0 {
            write!(f, "{}s", ms_u64 / 1_000)
        } else {
            write!(f, "{ms_u64}ms")
        }
    }
}

impl Serialize for Duration {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_spec_str(&s).map_err(serde::de::Error::custom)
    }
}
