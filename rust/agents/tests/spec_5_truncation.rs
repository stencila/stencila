//! Spec 5.1-5.3: Tool output truncation.
//!
//! Tests the character-based truncation algorithm, line-based truncation,
//! the full pipeline (`truncate_tool_output`), and default limits.

use stencila_agents::truncation::{
    DEFAULT_POLICIES, TruncationConfig, TruncationMode, truncate_lines, truncate_output,
    truncate_tool_output,
};

// =========================================================================
// 5.1 — truncate_output (character-based)
// =========================================================================

#[test]
fn truncate_output_below_limit_passthrough() {
    let input = "hello world";
    let result = truncate_output(input, 100, TruncationMode::HeadTail);
    assert_eq!(result, input);
}

#[test]
fn truncate_output_exactly_at_limit_passthrough() {
    let input = "abcde";
    let result = truncate_output(input, 5, TruncationMode::HeadTail);
    assert_eq!(result, input);
}

#[test]
fn truncate_output_empty_string_passthrough() {
    let result = truncate_output("", 100, TruncationMode::HeadTail);
    assert_eq!(result, "");
}

#[test]
fn truncate_output_head_tail_splits_evenly() {
    // 10 chars, limit 6 → half=3, removed=4
    let input = "0123456789";
    let result = truncate_output(input, 6, TruncationMode::HeadTail);
    assert!(result.starts_with("012"), "should keep first 3 chars");
    assert!(result.ends_with("789"), "should keep last 3 chars");
}

#[test]
fn truncate_output_head_tail_marker_contains_removed_count() {
    let input = "0123456789"; // 10 chars
    let result = truncate_output(input, 6, TruncationMode::HeadTail);
    // Removed = 10 - 6 = 4
    assert!(
        result.contains("4 characters were removed"),
        "marker should report removed count, got: {result}"
    );
}

#[test]
fn truncate_output_head_tail_marker_mentions_event_stream() {
    let input = "a".repeat(200);
    let result = truncate_output(&input, 100, TruncationMode::HeadTail);
    assert!(
        result.contains("event stream"),
        "marker should mention event stream"
    );
}

#[test]
fn truncate_output_tail_mode_keeps_end() {
    // 10 chars, limit 6 → removed=4, keep last 6
    let input = "0123456789";
    let result = truncate_output(input, 6, TruncationMode::Tail);
    assert!(result.ends_with("456789"), "should keep last 6 chars");
}

#[test]
fn truncate_output_tail_mode_marker_contains_removed_count() {
    let input = "0123456789"; // 10 chars
    let result = truncate_output(input, 6, TruncationMode::Tail);
    // Removed = 10 - 6 = 4
    assert!(
        result.contains("4 characters were removed"),
        "marker should report removed count, got: {result}"
    );
}

#[test]
fn truncate_output_tail_mode_no_head_content() {
    let input = "0123456789";
    let result = truncate_output(input, 6, TruncationMode::Tail);
    // Should NOT contain the head characters "012"
    assert!(
        !result.contains("012"),
        "tail mode should not include head content, got: {result}"
    );
}

#[test]
fn truncate_output_head_tail_odd_limit() {
    // Odd limit: 7 chars, half = 3 (integer division)
    // head = first 3, tail = last 3, removed = 10 - 7 = 3
    let input = "0123456789"; // 10 chars
    let result = truncate_output(input, 7, TruncationMode::HeadTail);
    assert!(result.starts_with("012"), "head should be first 3 chars");
    assert!(result.ends_with("789"), "tail should be last 3 chars");
    assert!(
        result.contains("3 characters were removed"),
        "got: {result}"
    );
}

#[test]
fn truncate_output_large_input() {
    let input = "x".repeat(100_000);
    let result = truncate_output(&input, 50_000, TruncationMode::HeadTail);
    // kept portions + marker text should be well bounded
    assert!(result.len() < 60_000, "result should be bounded");
    assert!(
        result.contains("50000 characters were removed"),
        "got: {result}"
    );
}

// =========================================================================
// 5.1 — UTF-8 / multibyte safety
// =========================================================================

#[test]
fn truncate_output_multibyte_head_tail_no_panic() {
    // Each CJK character is 3 bytes; 10 chars = 30 bytes.
    // Byte-indexed slicing at position 9 (half of 6 chars × 3 bytes/char = 9 bytes)
    // could land mid-codepoint on non-char-aware code.
    let input = "你好世界测试一二三四"; // 10 CJK chars
    assert_eq!(input.chars().count(), 10);
    let result = truncate_output(input, 6, TruncationMode::HeadTail);
    // half = 3 chars → head = "你好世", tail = "二三四"
    assert!(result.starts_with("你好世"), "head: {result}");
    assert!(result.ends_with("二三四"), "tail: {result}");
    assert!(
        result.contains("4 characters were removed"),
        "got: {result}"
    );
}

#[test]
fn truncate_output_multibyte_tail_no_panic() {
    let input = "你好世界测试一二三四"; // 10 CJK chars
    let result = truncate_output(input, 6, TruncationMode::Tail);
    // removed = 4, keep last 6
    assert!(result.ends_with("测试一二三四"), "tail: {result}");
    assert!(
        result.contains("4 characters were removed"),
        "got: {result}"
    );
}

#[test]
fn truncate_output_emoji_no_panic() {
    // Emoji are 4 bytes each in UTF-8
    let input = "🔴🟢🔵🟡🟣🟠🔶🔷🔸🔹"; // 10 emoji chars
    assert_eq!(input.chars().count(), 10);
    let result = truncate_output(input, 6, TruncationMode::HeadTail);
    assert!(result.starts_with("🔴🟢🔵"), "head: {result}");
    assert!(result.ends_with("🔸🔹"), "tail: {result}");
}

#[test]
fn truncate_output_mixed_ascii_multibyte() {
    let input = "ab你cde好fg"; // a(1) b(1) 你(1) c(1) d(1) e(1) 好(1) f(1) g(1) = 9 chars
    assert_eq!(input.chars().count(), 9);
    let result = truncate_output(input, 5, TruncationMode::HeadTail);
    // half = 2, head = "ab", tail = "fg", removed = 4
    assert!(result.starts_with("ab"), "head: {result}");
    assert!(result.ends_with("fg"), "tail: {result}");
    assert!(
        result.contains("4 characters were removed"),
        "got: {result}"
    );
}

// =========================================================================
// 5.1 — Zero-limit boundary conditions
// =========================================================================

#[test]
fn truncate_output_zero_limit_head_tail() {
    let input = "hello";
    let result = truncate_output(input, 0, TruncationMode::HeadTail);
    // half = 0, so no head or tail content, just the marker
    assert!(
        result.contains("5 characters were removed"),
        "got: {result}"
    );
    assert!(!result.contains("hello"), "no original content kept");
}

#[test]
fn truncate_output_zero_limit_tail() {
    let input = "hello";
    let result = truncate_output(input, 0, TruncationMode::Tail);
    assert!(
        result.contains("5 characters were removed"),
        "got: {result}"
    );
}

#[test]
fn truncate_output_limit_one_head_tail() {
    let input = "abc";
    let result = truncate_output(input, 1, TruncationMode::HeadTail);
    // half = 0, so no head/tail kept, removed = 2
    assert!(
        result.contains("2 characters were removed"),
        "got: {result}"
    );
}

// =========================================================================
// 5.3 — truncate_lines
// =========================================================================

#[test]
fn truncate_lines_below_limit_passthrough() {
    let input = "line1\nline2\nline3";
    let result = truncate_lines(input, 10);
    assert_eq!(result, input);
}

#[test]
fn truncate_lines_exactly_at_limit_passthrough() {
    let input = "line1\nline2\nline3";
    let result = truncate_lines(input, 3);
    assert_eq!(result, input);
}

#[test]
fn truncate_lines_empty_string_passthrough() {
    let result = truncate_lines("", 10);
    assert_eq!(result, "");
}

#[test]
fn truncate_lines_head_tail_split() {
    // 10 lines, limit 6 → head_count=3, tail_count=3, omitted=4
    let lines: Vec<String> = (0..10).map(|i| format!("line{i}")).collect();
    let input = lines.join("\n");
    let result = truncate_lines(&input, 6);
    assert!(
        result.starts_with("line0\n"),
        "should start with first line"
    );
    assert!(result.contains("line2\n"), "should include head lines");
    assert!(result.ends_with("line9"), "should end with last line");
    assert!(
        result.contains("4 lines omitted"),
        "should report omitted count, got: {result}"
    );
}

#[test]
fn truncate_lines_marker_format() {
    let lines: Vec<String> = (0..20).map(|i| format!("line{i}")).collect();
    let input = lines.join("\n");
    let result = truncate_lines(&input, 10);
    // omitted = 20 - 5 - 5 = 10
    assert!(
        result.contains("[... 10 lines omitted ...]"),
        "should have exact marker format, got: {result}"
    );
}

#[test]
fn truncate_lines_single_line_over_limit_passthrough() {
    // 1 line, limit 1 → no truncation
    let input = "only one line";
    let result = truncate_lines(input, 1);
    assert_eq!(result, input);
}

#[test]
fn truncate_lines_zero_limit() {
    let input = "line1\nline2\nline3";
    let result = truncate_lines(input, 0);
    // head_count=0, tail_count=0, all 3 lines omitted
    assert!(result.contains("3 lines omitted"), "got: {result}");
}

// =========================================================================
// 5.2 — Default policies (consolidated table)
// =========================================================================

#[test]
fn default_policies_char_limits_match_spec_table() {
    let p = &*DEFAULT_POLICIES;
    assert_eq!(p.get("read_file").map(|p| p.max_chars), Some(50_000));
    assert_eq!(p.get("shell").map(|p| p.max_chars), Some(30_000));
    assert_eq!(p.get("grep").map(|p| p.max_chars), Some(20_000));
    assert_eq!(p.get("glob").map(|p| p.max_chars), Some(20_000));
    assert_eq!(p.get("edit_file").map(|p| p.max_chars), Some(10_000));
    assert_eq!(p.get("apply_patch").map(|p| p.max_chars), Some(10_000));
    assert_eq!(p.get("write_file").map(|p| p.max_chars), Some(1_000));
    assert_eq!(p.get("spawn_agent").map(|p| p.max_chars), Some(20_000));
}

#[test]
fn default_policies_modes_match_spec_table() {
    let p = &*DEFAULT_POLICIES;
    assert_eq!(
        p.get("read_file").map(|p| p.mode),
        Some(TruncationMode::HeadTail)
    );
    assert_eq!(
        p.get("shell").map(|p| p.mode),
        Some(TruncationMode::HeadTail)
    );
    assert_eq!(p.get("grep").map(|p| p.mode), Some(TruncationMode::Tail));
    assert_eq!(p.get("glob").map(|p| p.mode), Some(TruncationMode::Tail));
    assert_eq!(
        p.get("edit_file").map(|p| p.mode),
        Some(TruncationMode::Tail)
    );
    assert_eq!(
        p.get("apply_patch").map(|p| p.mode),
        Some(TruncationMode::Tail)
    );
    assert_eq!(
        p.get("write_file").map(|p| p.mode),
        Some(TruncationMode::Tail)
    );
    assert_eq!(
        p.get("spawn_agent").map(|p| p.mode),
        Some(TruncationMode::HeadTail)
    );
}

#[test]
fn default_policies_line_limits_match_spec_table() {
    let p = &*DEFAULT_POLICIES;
    assert_eq!(p.get("shell").map(|p| p.max_lines), Some(Some(256)));
    assert_eq!(p.get("grep").map(|p| p.max_lines), Some(Some(200)));
    assert_eq!(p.get("glob").map(|p| p.max_lines), Some(Some(500)));
    // These tools should NOT have line limits
    assert_eq!(p.get("read_file").map(|p| p.max_lines), Some(None));
    assert_eq!(p.get("edit_file").map(|p| p.max_lines), Some(None));
    assert_eq!(p.get("write_file").map(|p| p.max_lines), Some(None));
    assert_eq!(p.get("apply_patch").map(|p| p.max_lines), Some(None));
    assert_eq!(p.get("spawn_agent").map(|p| p.max_lines), Some(None));
}

// =========================================================================
// 5.2 — TruncationConfig
// =========================================================================

#[test]
fn truncation_config_default_has_empty_overrides() {
    let config = TruncationConfig::default();
    assert!(config.tool_output_limits.is_empty());
    assert!(config.tool_line_limits.is_empty());
}

#[test]
fn truncation_config_custom_char_limit_overrides_default() {
    let mut config = TruncationConfig::default();
    config.tool_output_limits.insert("shell".to_string(), 5_000);
    let input = "x".repeat(10_000);
    let result = truncate_tool_output(&input, "shell", &config);
    // Should use 5000 limit (not default 30000)
    assert!(
        result.contains("5000 characters were removed"),
        "should use overridden limit, got len={}",
        result.len()
    );
}

#[test]
fn truncation_config_custom_line_limit_overrides_default() {
    let mut config = TruncationConfig::default();
    config.tool_line_limits.insert("shell".to_string(), 4);
    // Create output with many short lines that fits under char limit
    let lines: Vec<String> = (0..100).map(|i| format!("line{i}")).collect();
    let input = lines.join("\n");
    let result = truncate_tool_output(&input, "shell", &config);
    // Should apply line limit of 4
    assert!(
        result.contains("lines omitted"),
        "should truncate by lines, got: {result}"
    );
}

// =========================================================================
// 5.3 — truncate_tool_output (full pipeline)
// =========================================================================

#[test]
fn truncate_tool_output_below_all_limits() {
    let config = TruncationConfig::default();
    let input = "small output";
    let result = truncate_tool_output(input, "shell", &config);
    assert_eq!(result, input, "should pass through unchanged");
}

#[test]
fn truncate_tool_output_char_truncation_runs_first() {
    // Pathological case from spec 5.3: 2 lines where each is very long
    let config = TruncationConfig::default();
    let long_line = "x".repeat(50_000);
    let input = format!("{long_line}\n{long_line}"); // 100,001 chars total
    let result = truncate_tool_output(&input, "shell", &config);
    // shell default is 30,000 chars — char truncation must fire
    assert!(
        result.len() < 40_000,
        "char truncation should bound the result, got len={}",
        result.len()
    );
}

#[test]
fn truncate_tool_output_line_truncation_runs_after_char() {
    let config = TruncationConfig::default();
    // Many short lines within char limit but exceeding line limit
    // shell: 30,000 chars, 256 lines
    let lines: Vec<String> = (0..500).map(|i| format!("line {i}")).collect();
    let input = lines.join("\n");
    // ~4000 chars (under 30,000), but 500 lines (over 256)
    let result = truncate_tool_output(&input, "shell", &config);
    assert!(
        result.contains("lines omitted"),
        "line truncation should fire, got: {result}"
    );
}

#[test]
fn truncate_tool_output_unknown_tool_uses_generous_default() {
    let config = TruncationConfig::default();
    let input = "x".repeat(100_000);
    let result = truncate_tool_output(&input, "unknown_tool", &config);
    // Unknown tools should get a fallback limit, not panic
    assert!(result.len() < input.len(), "should still truncate");
}

#[test]
fn truncate_tool_output_read_file_no_line_limit() {
    let config = TruncationConfig::default();
    // Many lines within char limit — read_file has no line limit
    let lines: Vec<String> = (0..1000).map(|i| format!("line {i}")).collect();
    let input = lines.join("\n");
    // ~8000 chars, well under read_file's 50,000 char limit
    let result = truncate_tool_output(&input, "read_file", &config);
    assert!(
        !result.contains("lines omitted"),
        "read_file should not have line truncation"
    );
}

#[test]
fn truncate_tool_output_grep_uses_tail_mode() {
    let config = TruncationConfig::default();
    let input = "x".repeat(30_000); // exceeds grep's 20,000 limit
    let result = truncate_tool_output(&input, "grep", &config);
    // Tail mode: starts with the warning marker
    assert!(
        result.starts_with("[WARNING"),
        "grep should use tail mode, got start: {}",
        &result[..50.min(result.len())]
    );
}

#[test]
fn truncate_tool_output_shell_uses_head_tail_mode() {
    let config = TruncationConfig::default();
    let input = "x".repeat(60_000); // exceeds shell's 30,000 limit
    let result = truncate_tool_output(&input, "shell", &config);
    // Head/tail mode: starts with kept head content
    assert!(
        result.starts_with("xxx"),
        "shell should use head_tail mode, got start: {}",
        &result[..50.min(result.len())]
    );
    assert!(
        result.contains("[WARNING"),
        "should contain truncation marker"
    );
}

// =========================================================================
// TruncationMode — serde and traits
// =========================================================================

#[test]
fn truncation_mode_debug_and_clone() {
    let mode = TruncationMode::HeadTail;
    let cloned = mode;
    assert_eq!(format!("{mode:?}"), "HeadTail");
    assert_eq!(mode, cloned);
}

#[test]
fn truncation_mode_serde_roundtrip() -> Result<(), serde_json::Error> {
    let mode = TruncationMode::Tail;
    let json = serde_json::to_string(&mode)?;
    let back: TruncationMode = serde_json::from_str(&json)?;
    assert_eq!(back, mode);
    Ok(())
}
