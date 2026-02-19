use std::fmt;
use std::str::FromStr;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::error::{AttractorError, AttractorResult};

/// Byte-wise string equality usable in `const fn` contexts.
const fn const_str_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// The built-in handler types recognized by the attractor engine.
///
/// Each variant corresponds to a handler registered in the default
/// [`HandlerRegistry`](crate::handler::HandlerRegistry). The `Display`
/// implementation produces the canonical dot-separated string (e.g.
/// `"parallel.fan_in"`), and `PartialEq<str>` allows ergonomic
/// comparison with the string returned by [`Node::handler_type()`](crate::graph::Node::handler_type):
///
/// ```ignore
/// if node.handler_type() == HandlerType::ParallelFanIn {
///     // ...
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HandlerType {
    Start,
    Exit,
    Fail,
    Conditional,
    Codergen,
    Shell,
    WaitHuman,
    Parallel,
    ParallelFanIn,
    StackManagerLoop,
}

impl HandlerType {
    /// All known handler types.
    pub const ALL: &[HandlerType] = &[
        Self::Start,
        Self::Exit,
        Self::Fail,
        Self::Conditional,
        Self::Codergen,
        Self::Shell,
        Self::WaitHuman,
        Self::Parallel,
        Self::ParallelFanIn,
        Self::StackManagerLoop,
    ];

    /// Return the canonical string representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Exit => "exit",
            Self::Fail => "fail",
            Self::Conditional => "conditional",
            Self::Codergen => "codergen",
            Self::Shell => "shell",
            Self::WaitHuman => "wait.human",
            Self::Parallel => "parallel",
            Self::ParallelFanIn => "parallel.fan_in",
            Self::StackManagerLoop => "stack.manager_loop",
        }
    }

    /// Map a DOT shape name to the corresponding handler type per §2.8.
    ///
    /// Unknown shapes (including `"box"`) default to [`Codergen`](Self::Codergen).
    #[must_use]
    pub const fn from_shape(shape: &str) -> Self {
        if const_str_eq(shape, "Mdiamond") {
            Self::Start
        } else if const_str_eq(shape, "Msquare") {
            Self::Exit
        } else if const_str_eq(shape, "invtriangle") {
            Self::Fail
        } else if const_str_eq(shape, "hexagon") || const_str_eq(shape, "human") {
            Self::WaitHuman
        } else if const_str_eq(shape, "diamond") {
            Self::Conditional
        } else if const_str_eq(shape, "component") {
            Self::Parallel
        } else if const_str_eq(shape, "tripleoctagon") {
            Self::ParallelFanIn
        } else if const_str_eq(shape, "parallelogram") {
            Self::Shell
        } else if const_str_eq(shape, "house") {
            Self::StackManagerLoop
        } else {
            // "box" and all unknown shapes default to codergen
            Self::Codergen
        }
    }
}

impl fmt::Display for HandlerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for HandlerType {
    type Err = AttractorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "start" => Ok(Self::Start),
            "exit" => Ok(Self::Exit),
            "fail" => Ok(Self::Fail),
            "conditional" => Ok(Self::Conditional),
            "codergen" => Ok(Self::Codergen),
            "shell" => Ok(Self::Shell),
            "wait.human" => Ok(Self::WaitHuman),
            "parallel" => Ok(Self::Parallel),
            "parallel.fan_in" => Ok(Self::ParallelFanIn),
            "stack.manager_loop" => Ok(Self::StackManagerLoop),
            other => Err(AttractorError::InvalidPipeline {
                reason: format!("unknown handler type: {other}"),
            }),
        }
    }
}

impl PartialEq<str> for HandlerType {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for HandlerType {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<HandlerType> for str {
    fn eq(&self, other: &HandlerType) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<HandlerType> for &str {
    fn eq(&self, other: &HandlerType) -> bool {
        *self == other.as_str()
    }
}

impl From<HandlerType> for String {
    fn from(ht: HandlerType) -> Self {
        ht.as_str().to_string()
    }
}

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

    /// Optional node ID that the engine should jump to directly,
    /// bypassing normal edge selection.
    ///
    /// Set by the parallel handler to route execution to the structural
    /// fan-in node after all branches complete. Using a dedicated field
    /// (rather than piggybacking on `context_updates`) keeps the jump
    /// target out of the pipeline context, preventing stale values from
    /// leaking into downstream stages or interfering with later parallel
    /// blocks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jump_target: Option<String>,
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
            jump_target: None,
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
            jump_target: None,
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
            jump_target: None,
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
            jump_target: None,
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
