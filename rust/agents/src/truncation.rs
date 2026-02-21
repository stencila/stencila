//! Tool output truncation (spec 5.1-5.3).
//!
//! Provides character-based and line-based truncation for tool outputs before
//! they are sent to the LLM. The full untruncated output is always available
//! via the event stream (`TOOL_CALL_END` event).
//!
//! All "character" limits count Unicode scalar values (`.chars()`), not bytes.

use std::collections::HashMap;
use std::sync::LazyLock;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// TruncationMode (spec 5.1)
// ---------------------------------------------------------------------------

/// How to truncate tool output that exceeds the character limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TruncationMode {
    /// Keep the beginning and end, remove the middle.
    HeadTail,
    /// Keep only the end, remove the beginning.
    Tail,
}

// ---------------------------------------------------------------------------
// Per-tool truncation policy (spec 5.2, 5.3)
// ---------------------------------------------------------------------------

/// Truncation policy for a single tool, combining character limit, mode,
/// and optional line limit into a single record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolTruncationPolicy {
    /// Maximum characters (Unicode scalar values) before char truncation.
    pub max_chars: usize,
    /// Which truncation mode to use.
    pub mode: TruncationMode,
    /// Maximum lines after char truncation (None = no line limit).
    pub max_lines: Option<usize>,
}

/// Default truncation policies per tool (spec 5.2/5.3 tables combined).
///
/// One entry per tool — adding or changing a tool only requires touching
/// one place.
pub static DEFAULT_POLICIES: LazyLock<HashMap<&'static str, ToolTruncationPolicy>> =
    LazyLock::new(|| {
        use TruncationMode::{HeadTail, Tail};
        HashMap::from([
            (
                "read_file",
                ToolTruncationPolicy {
                    max_chars: 50_000,
                    mode: HeadTail,
                    max_lines: None,
                },
            ),
            (
                "shell",
                ToolTruncationPolicy {
                    max_chars: 30_000,
                    mode: HeadTail,
                    max_lines: Some(256),
                },
            ),
            (
                "grep",
                ToolTruncationPolicy {
                    max_chars: 20_000,
                    mode: Tail,
                    max_lines: Some(200),
                },
            ),
            (
                "glob",
                ToolTruncationPolicy {
                    max_chars: 20_000,
                    mode: Tail,
                    max_lines: Some(500),
                },
            ),
            (
                "edit_file",
                ToolTruncationPolicy {
                    max_chars: 10_000,
                    mode: Tail,
                    max_lines: None,
                },
            ),
            (
                "apply_patch",
                ToolTruncationPolicy {
                    max_chars: 10_000,
                    mode: Tail,
                    max_lines: None,
                },
            ),
            (
                "write_file",
                ToolTruncationPolicy {
                    max_chars: 1_000,
                    mode: Tail,
                    max_lines: None,
                },
            ),
            (
                "spawn_agent",
                ToolTruncationPolicy {
                    max_chars: 20_000,
                    mode: HeadTail,
                    max_lines: None,
                },
            ),
        ])
    });

/// Generous fallback policy for tools not in the default table.
const FALLBACK_POLICY: ToolTruncationPolicy = ToolTruncationPolicy {
    max_chars: 30_000,
    mode: TruncationMode::HeadTail,
    max_lines: None,
};

// ---------------------------------------------------------------------------
// TruncationConfig
// ---------------------------------------------------------------------------

/// Per-session overrides for tool output limits.
///
/// Empty maps mean "use the spec defaults from [`DEFAULT_POLICIES`]."
#[derive(Debug, Clone, Default)]
pub struct TruncationConfig {
    /// Per-tool character limit overrides.
    pub tool_output_limits: HashMap<String, usize>,
    /// Per-tool line limit overrides.
    pub tool_line_limits: HashMap<String, usize>,
}

// ---------------------------------------------------------------------------
// truncate_output — character-based (spec 5.1)
// ---------------------------------------------------------------------------

/// Truncate `output` to at most `max_chars` Unicode scalar values using the
/// given mode.
///
/// If the output is within the limit it is returned unchanged. Otherwise a
/// truncation warning marker is inserted so the model knows data was removed.
///
/// A `max_chars` of 0 returns only the truncation marker (no kept content).
#[must_use]
pub fn truncate_output(output: &str, max_chars: usize, mode: TruncationMode) -> String {
    let char_count = output.chars().count();
    if char_count <= max_chars {
        return output.to_string();
    }

    let removed = char_count - max_chars;

    match mode {
        TruncationMode::HeadTail => {
            let tail_half = max_chars / 2;
            let head_half = max_chars - tail_half;
            let head: String = output.chars().take(head_half).collect();
            let tail: String = output.chars().skip(char_count - tail_half).collect();
            format!(
                "{head}\n\n\
                 [WARNING: Tool output was truncated. \
                 {removed} characters were removed from the middle. \
                 The full output is available in the event stream. \
                 If you need to see specific parts, re-run the tool \
                 with more targeted parameters.]\n\n\
                 {tail}"
            )
        }
        TruncationMode::Tail => {
            let tail: String = output.chars().skip(removed).collect();
            format!(
                "[WARNING: Tool output was truncated. First \
                 {removed} characters were removed. \
                 The full output is available in the event stream.]\n\n\
                 {tail}"
            )
        }
    }
}

// ---------------------------------------------------------------------------
// truncate_lines — line-based (spec 5.3)
// ---------------------------------------------------------------------------

/// Truncate `output` to at most `max_lines` lines using a head/tail split.
///
/// If the line count is within the limit the output is returned unchanged.
/// A `max_lines` of 0 returns only the omission marker.
#[must_use]
pub fn truncate_lines(output: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = output.split('\n').collect();
    if lines.len() <= max_lines {
        return output.to_string();
    }

    let head_count = max_lines / 2;
    let tail_count = max_lines - head_count;
    let omitted = lines.len() - head_count - tail_count;

    let head = lines[..head_count].join("\n");
    let tail = lines[lines.len() - tail_count..].join("\n");

    format!("{head}\n[... {omitted} lines omitted ...]\n{tail}")
}

// ---------------------------------------------------------------------------
// truncate_tool_output — full pipeline (spec 5.3)
// ---------------------------------------------------------------------------

/// Apply the full truncation pipeline for a tool's output.
///
/// 1. Character-based truncation (always runs first — spec 5.3).
/// 2. Line-based truncation (secondary readability pass).
///
/// Uses `config` overrides when present, otherwise falls back to the spec
/// defaults in [`DEFAULT_POLICIES`].
#[must_use]
pub fn truncate_tool_output(output: &str, tool_name: &str, config: &TruncationConfig) -> String {
    let policy = DEFAULT_POLICIES.get(tool_name).unwrap_or(&FALLBACK_POLICY);

    // Step 1: resolve char limit (config override or policy default)
    let max_chars = config
        .tool_output_limits
        .get(tool_name)
        .copied()
        .unwrap_or(policy.max_chars);

    let result = truncate_output(output, max_chars, policy.mode);

    // Step 2: resolve line limit (config override, then policy default)
    let max_lines = config
        .tool_line_limits
        .get(tool_name)
        .copied()
        .or(policy.max_lines);

    match max_lines {
        Some(limit) => truncate_lines(&result, limit),
        None => result,
    }
}
