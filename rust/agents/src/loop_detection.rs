//! Tool-call loop detection (spec 2.10).
//!
//! Detects repeating patterns of tool calls so the session can inject a
//! steering warning and break the cycle. Patterns of length 1, 2, and 3
//! are checked within a sliding window of recent call signatures.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use stencila_models3::types::tool::ToolCall;

/// Build a deterministic signature for a tool call: `"name:args_hash"`.
///
/// Two calls with the same name and serialized arguments produce the same
/// signature. Different argument order in JSON objects may produce different
/// signatures — this is acceptable because it only causes false negatives
/// (missed loops), never false positives.
#[must_use]
pub fn tool_call_signature(call: &ToolCall) -> String {
    let args_str = call.arguments.to_string();
    let mut hasher = DefaultHasher::new();
    args_str.hash(&mut hasher);
    format!("{}:{}", call.name, hasher.finish())
}

/// Check the last `window_size` signatures for a repeating pattern.
///
/// Returns `Some(message)` describing the detected pattern if the full
/// window consists of a single repeating subsequence of length 1, 2, or 3.
/// Returns `None` if no pattern is found or the window is too small.
///
/// # Algorithm
///
/// For each candidate length `k` in `[1, 2, 3]`:
/// 1. The window must be evenly divisible by `k`.
/// 2. Extract the first `k` signatures as the candidate pattern.
/// 3. Verify every subsequent group of `k` signatures matches the candidate.
/// 4. If all match → loop detected.
#[must_use]
pub fn detect_loop(signatures: &[String], window_size: usize) -> Option<String> {
    if signatures.len() < window_size || window_size < 2 {
        return None;
    }

    let window = &signatures[signatures.len() - window_size..];

    for pattern_len in 1..=3 {
        if window_size % pattern_len != 0 {
            continue;
        }

        let pattern = &window[..pattern_len];
        let mut is_loop = true;

        let mut i = pattern_len;
        while i < window_size {
            if window[i..i + pattern_len] != *pattern {
                is_loop = false;
                break;
            }
            i += pattern_len;
        }

        if is_loop {
            let names: Vec<&str> = pattern
                .iter()
                .map(|s| s.split(':').next().unwrap_or("unknown"))
                .collect();
            return Some(format!(
                "Loop detected: the same tool call pattern [{}] has repeated {} times. \
                 Try a different approach or break the cycle.",
                names.join(" → "),
                window_size / pattern_len,
            ));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_call(name: &str, args: serde_json::Value) -> ToolCall {
        ToolCall {
            id: "c1".into(),
            name: name.into(),
            arguments: args,
            raw_arguments: None,
            parse_error: None,
        }
    }

    #[test]
    fn signature_deterministic() {
        let a = make_call("read_file", json!({"path": "/foo"}));
        let b = make_call("read_file", json!({"path": "/foo"}));
        assert_eq!(tool_call_signature(&a), tool_call_signature(&b));
    }

    #[test]
    fn signature_differs_for_different_args() {
        let a = make_call("read_file", json!({"path": "/foo"}));
        let b = make_call("read_file", json!({"path": "/bar"}));
        assert_ne!(tool_call_signature(&a), tool_call_signature(&b));
    }

    #[test]
    fn signature_differs_for_different_names() {
        let a = make_call("read_file", json!({"path": "/foo"}));
        let b = make_call("write_file", json!({"path": "/foo"}));
        assert_ne!(tool_call_signature(&a), tool_call_signature(&b));
    }

    #[test]
    fn detect_loop_empty() {
        assert!(detect_loop(&[], 10).is_none());
    }

    #[test]
    fn detect_loop_below_window() {
        let sigs: Vec<String> = (0..5).map(|_| "read_file:123".into()).collect();
        assert!(detect_loop(&sigs, 10).is_none());
    }

    #[test]
    fn detect_loop_pattern_1() {
        // Same signature repeated 10 times
        let sigs: Vec<String> = (0..10).map(|_| "read_file:123".into()).collect();
        let result = detect_loop(&sigs, 10);
        assert!(result.is_some());
        let msg = result.unwrap_or_default();
        assert!(msg.contains("read_file"));
        assert!(msg.contains("10 times"));
    }

    #[test]
    fn detect_loop_pattern_2() {
        // Alternating A-B pattern, 10 items
        let mut sigs = Vec::new();
        for _ in 0..5 {
            sigs.push("read_file:111".into());
            sigs.push("write_file:222".into());
        }
        let result = detect_loop(&sigs, 10);
        assert!(result.is_some());
        let msg = result.unwrap_or_default();
        assert!(msg.contains("read_file"));
        assert!(msg.contains("write_file"));
    }

    #[test]
    fn detect_loop_pattern_3() {
        // A-B-C cycling, 9 items with window=9
        let mut sigs = Vec::new();
        for _ in 0..3 {
            sigs.push("read_file:111".into());
            sigs.push("write_file:222".into());
            sigs.push("shell:333".into());
        }
        let result = detect_loop(&sigs, 9);
        assert!(result.is_some());
    }

    #[test]
    fn no_false_positive_varied_calls() {
        let sigs: Vec<String> = (0..10).map(|i| format!("tool_{i}:hash_{i}")).collect();
        assert!(detect_loop(&sigs, 10).is_none());
    }

    #[test]
    fn no_false_positive_almost_pattern() {
        // 9 identical + 1 different → not a pattern of length 1
        let mut sigs: Vec<String> = (0..9).map(|_| "read_file:123".into()).collect();
        sigs.push("write_file:456".into());
        assert!(detect_loop(&sigs, 10).is_none());
    }

    #[test]
    fn window_larger_than_signatures_returns_none() {
        let sigs: Vec<String> = vec!["a:1".into(), "a:1".into()];
        assert!(detect_loop(&sigs, 10).is_none());
    }

    #[test]
    fn window_size_one_returns_none() {
        let sigs: Vec<String> = vec!["a:1".into()];
        assert!(detect_loop(&sigs, 1).is_none());
    }
}
