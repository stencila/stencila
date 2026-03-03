//! Shell evaluator: inner command extraction and pattern evaluation.

use std::borrow::Cow;

use super::tokenizer::{
    ParseError, SpannedToken, extract_substitutions, split_on_separators, tokenize_spanned,
};
use crate::tool_guard::GuardVerdict;

/// Maximum command size accepted by the shell guard extraction pipeline.
pub const MAX_COMMAND_BYTES: usize = 8_192;

/// Maximum combined wrapper/substitution nesting depth.
pub const MAX_NESTING_DEPTH: usize = 5;

const PARSE_REASON: &str = "Unable to safely parse command for guard evaluation";
const PARSE_SUGGESTION: &str = "Simplify the command to use standard quoting and syntax.";
const LENGTH_REASON: &str = "Command exceeds maximum length for shell guard evaluation";
const LENGTH_SUGGESTION: &str = "Break the command into smaller steps.";
const NESTING_REASON: &str = "Command nesting exceeds maximum depth for shell guard evaluation";
const NESTING_SUGGESTION: &str = "Simplify the command to reduce nesting.";

const SHELL_WRAPPER_BINARIES: &[&str] =
    &["bash", "sh", "zsh", "dash", "fish", "csh", "tcsh", "ksh"];

/// Extract shell command segments for guard evaluation.
///
/// Output includes:
/// - top-level segments split on separators (`&&`, `||`, `;`, `&`, `\n`)
/// - extracted command substitutions from those segments (`$(...)`, backticks)
///
/// Wrapper commands (`bash -c`, `cmd /c`, `powershell -Command`, etc.) are
/// recursively unwrapped before splitting/evaluating.
pub fn extract_commands(raw: &str) -> Result<Vec<String>, GuardVerdict> {
    extract_commands_inner(raw, 0)
}

enum EnvStripResult {
    RawStart(usize),
    Synthetic(String),
}

fn extract_commands_inner(raw: &str, depth: usize) -> Result<Vec<String>, GuardVerdict> {
    if raw.len() > MAX_COMMAND_BYTES {
        return Err(deny_length());
    }

    if depth > MAX_NESTING_DEPTH {
        return Err(deny_nesting());
    }

    let tokens = tokenize_spanned(raw).map_err(deny_parse_error)?;
    let stripped_raw = match strip_env_prefix(&tokens, raw).map_err(deny_parse_error)? {
        EnvStripResult::RawStart(start) => Cow::Borrowed(&raw[start..]),
        EnvStripResult::Synthetic(command) => Cow::Owned(command),
    };

    let stripped_tokens = tokenize_spanned(stripped_raw.as_ref()).map_err(deny_parse_error)?;
    if let Some(inner) = detect_wrapper(&stripped_tokens).map_err(deny_parse_error)? {
        return extract_commands_inner(&inner, depth + 1);
    }

    let segments = split_on_separators(stripped_raw.as_ref()).map_err(deny_parse_error)?;

    let mut all = Vec::with_capacity(segments.len() * 2);
    for segment in &segments {
        all.push(segment.clone());
    }
    for segment in &segments {
        let substitutions = extract_substitutions(segment).map_err(deny_parse_error)?;
        for substitution in substitutions {
            let nested = extract_commands_inner(&substitution, depth + 1)?;
            all.extend(nested);
        }
    }

    Ok(all)
}

fn strip_env_prefix(tokens: &[SpannedToken], raw: &str) -> Result<EnvStripResult, ParseError> {
    let raw_len = raw.len();

    if tokens
        .first()
        .map(|token| binary_basename(&token.value))
        .as_deref()
        != Some("env")
    {
        return Ok(EnvStripResult::RawStart(0));
    }

    let mut i = 1;
    let mut split_string_fallback: Option<String> = None;

    while i < tokens.len() {
        let value = tokens[i].value.as_str();

        if value == "--" {
            i += 1;
            break;
        }

        if matches!(value, "-i" | "--ignore-environment" | "-0" | "--null") {
            i += 1;
            continue;
        }

        if matches!(
            value,
            "-u" | "--unset" | "-C" | "--chdir" | "-S" | "--split-string"
        ) {
            if i + 1 >= tokens.len() {
                return Err(ParseError::MissingArgument {
                    flag: value.to_string(),
                });
            }
            if matches!(value, "-S" | "--split-string") {
                split_string_fallback = Some(tokens[i + 1].value.clone());
            }

            i += 2;

            continue;
        }

        // Reject any unrecognized flag (including dash-prefixed tokens with
        // `=` like `-x=1` or `--bogus=1`) before treating `=` as an
        // env assignment.
        if value.starts_with('-') {
            return Err(ParseError::UnrecognizedEnvFlag {
                flag: value.to_string(),
            });
        }

        if value.contains('=') {
            i += 1;
            continue;
        }

        return Ok(EnvStripResult::RawStart(tokens[i].start));
    }

    if i < tokens.len() {
        return Ok(EnvStripResult::RawStart(tokens[i].start));
    }

    if let Some(split_string) = split_string_fallback {
        if !split_string.trim().is_empty() {
            return Ok(EnvStripResult::Synthetic(split_string));
        }
    }

    Ok(EnvStripResult::RawStart(raw_len))
}

fn detect_wrapper(tokens: &[SpannedToken]) -> Result<Option<String>, ParseError> {
    if tokens.is_empty() {
        return Ok(None);
    }

    let basename = binary_basename(&tokens[0].value);

    if SHELL_WRAPPER_BINARIES.contains(&basename.as_str()) {
        return detect_unix_wrapper(tokens);
    }

    let normalized = normalize_windows_binary(&basename);
    if normalized == "cmd" {
        return detect_cmd_wrapper(tokens);
    }
    if normalized == "powershell" || normalized == "pwsh" {
        return detect_powershell_wrapper(tokens);
    }

    Ok(None)
}

fn detect_unix_wrapper(tokens: &[SpannedToken]) -> Result<Option<String>, ParseError> {
    for i in 1..tokens.len() {
        let token = tokens[i].value.as_str();

        // `--` ends option parsing; `-c` after this is positional/script data,
        // not a wrapper flag.
        if token == "--" {
            return Ok(None);
        }

        if token == "-c" {
            if i + 1 >= tokens.len() {
                return Err(ParseError::MissingArgument {
                    flag: "-c".to_string(),
                });
            }
            return Ok(Some(tokens[i + 1].value.clone()));
        }

        if is_combined_c_flag(token) {
            if i + 1 >= tokens.len() {
                return Err(ParseError::MissingArgument {
                    flag: token.to_string(),
                });
            }
            return Ok(Some(tokens[i + 1].value.clone()));
        }

        if token.starts_with('-') {
            continue;
        }

        return Ok(None);
    }

    Ok(None)
}

fn detect_cmd_wrapper(tokens: &[SpannedToken]) -> Result<Option<String>, ParseError> {
    for i in 1..tokens.len() {
        let token = tokens[i].value.as_str();

        if token.eq_ignore_ascii_case("/c") {
            if i + 1 >= tokens.len() {
                return Err(ParseError::MissingArgument {
                    flag: token.to_string(),
                });
            }
            // `cmd /c` treats all remaining text as the command.
            return Ok(Some(join_remaining_tokens(tokens, i + 1)));
        }

        if token.starts_with('/') {
            continue;
        }

        return Ok(None);
    }

    Ok(None)
}

fn detect_powershell_wrapper(tokens: &[SpannedToken]) -> Result<Option<String>, ParseError> {
    for i in 1..tokens.len() {
        let token = tokens[i].value.as_str();

        if token.eq_ignore_ascii_case("-command") {
            if i + 1 >= tokens.len() {
                return Err(ParseError::MissingArgument {
                    flag: token.to_string(),
                });
            }
            // `powershell -Command` treats all remaining text as the command.
            return Ok(Some(join_remaining_tokens(tokens, i + 1)));
        }

        if token.starts_with('-') {
            continue;
        }

        return Ok(None);
    }

    Ok(None)
}

/// Join all token values from `start` onward with spaces.
///
/// Used for `cmd /c` and `powershell -Command` where all remaining
/// arguments form the inner command string.
fn join_remaining_tokens(tokens: &[SpannedToken], start: usize) -> String {
    tokens[start..]
        .iter()
        .map(|t| t.value.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_combined_c_flag(token: &str) -> bool {
    token.starts_with('-')
        && token.len() >= 2
        && token.len() <= 6
        && token[1..].chars().all(|ch| ch.is_ascii_alphabetic())
        && token[1..].contains('c')
}

/// Extract the basename from a potentially path-qualified binary.
///
/// Handles both Unix (`/usr/bin/bash`) and Windows (`C:\Windows\cmd.exe`)
/// path separators.
fn binary_basename(binary: &str) -> String {
    binary
        .rsplit(|ch: char| ch == '/' || ch == '\\')
        .next()
        .unwrap_or(binary)
        .to_string()
}

fn normalize_windows_binary(binary: &str) -> String {
    let lowered = binary.to_ascii_lowercase();
    lowered
        .strip_suffix(".exe")
        .unwrap_or(lowered.as_str())
        .to_string()
}

fn deny_parse_error(_error: ParseError) -> GuardVerdict {
    GuardVerdict::Deny {
        reason: PARSE_REASON,
        suggestion: PARSE_SUGGESTION,
        rule_id: "shell.parse_error",
    }
}

fn deny_length() -> GuardVerdict {
    GuardVerdict::Deny {
        reason: LENGTH_REASON,
        suggestion: LENGTH_SUGGESTION,
        rule_id: "shell.command_too_long",
    }
}

fn deny_nesting() -> GuardVerdict {
    GuardVerdict::Deny {
        reason: NESTING_REASON,
        suggestion: NESTING_SUGGESTION,
        rule_id: "shell.max_nesting_depth",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deny_rule_id(result: Result<Vec<String>, GuardVerdict>) -> String {
        match result {
            Ok(value) => panic!("expected deny, got ok: {value:?}"),
            Err(GuardVerdict::Deny { rule_id, .. }) => rule_id.to_string(),
            Err(other) => panic!("expected deny, got {other:?}"),
        }
    }

    #[test]
    fn env_prefix_stripping_variants() {
        let commands =
            extract_commands(r#"env -u VAR -C /tmp -S "x y" NAME=VALUE -- bash -c "rm -rf /""#)
                .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        assert_eq!(commands, vec!["rm -rf /"]);

        let commands = extract_commands(r#"env NAME=VALUE bash -lc "echo hi""#)
            .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        assert_eq!(commands, vec!["echo hi"]);

        let commands = extract_commands(r#"env -- bash -c "echo hi""#)
            .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        assert_eq!(commands, vec!["echo hi"]);

        let commands = extract_commands("env -i ls -la")
            .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        assert_eq!(commands, vec!["ls -la"]);

        let commands = extract_commands(r#"env -S "bash -c 'rm -rf /'""#)
            .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        assert_eq!(commands, vec!["rm -rf /"]);
    }

    #[test]
    fn env_prefix_missing_argument_and_unknown_flag_denied() {
        assert_eq!(
            deny_rule_id(extract_commands("env -u")),
            "shell.parse_error"
        );
        assert_eq!(
            deny_rule_id(extract_commands("env -C")),
            "shell.parse_error"
        );
        assert_eq!(
            deny_rule_id(extract_commands("env -S")),
            "shell.parse_error"
        );
        assert_eq!(
            deny_rule_id(extract_commands("env -x bash -c 'echo hi'")),
            "shell.parse_error"
        );

        // Dash-prefixed tokens with `=` must not be treated as env
        // assignments — they are unrecognized flags.
        assert_eq!(
            deny_rule_id(extract_commands("env -x=1 bash -c 'echo hi'")),
            "shell.parse_error"
        );
        assert_eq!(
            deny_rule_id(extract_commands("env --bogus=1 bash -c 'echo hi'")),
            "shell.parse_error"
        );
    }

    #[test]
    fn unix_wrapper_detection_and_missing_c_argument() {
        let standalone = extract_commands(r#"bash -c "rm -rf /""#)
            .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        assert_eq!(standalone, vec!["rm -rf /"]);

        let combined = extract_commands(r#"bash -lc "git status""#)
            .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        assert_eq!(combined, vec!["git status"]);

        let separate = extract_commands(r#"bash -l -c "git status""#)
            .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        assert_eq!(separate, vec!["git status"]);

        // `-cl` also contains `c` and must be detected as a wrapper.
        let c_not_last = extract_commands(r#"bash -cl "git status""#)
            .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        assert_eq!(c_not_last, vec!["git status"]);

        assert_eq!(
            deny_rule_id(extract_commands("bash -c")),
            "shell.parse_error"
        );
        assert_eq!(
            deny_rule_id(extract_commands("bash -lc")),
            "shell.parse_error"
        );

        // `--` ends option parsing: `-c` after it is positional, not a wrapper
        // command string flag.
        let not_wrapper = extract_commands(r#"bash -- -c "rm -rf /""#)
            .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        assert_eq!(not_wrapper, vec![r#"bash -- -c "rm -rf /""#]);
    }

    #[test]
    fn nested_wrappers_and_depth_limit() {
        fn wrap_bash_c(inner: &str) -> String {
            let escaped = inner.replace('\\', "\\\\").replace('"', "\\\"");
            format!("bash -c \"{escaped}\"")
        }

        let mut depth_five = "echo ok".to_string();
        for _ in 0..5 {
            depth_five = wrap_bash_c(&depth_five);
        }
        let commands = extract_commands(&depth_five)
            .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        assert_eq!(commands, vec!["echo ok"]);

        let mut depth_six = "echo nope".to_string();
        for _ in 0..6 {
            depth_six = wrap_bash_c(&depth_six);
        }
        assert_eq!(
            deny_rule_id(extract_commands(&depth_six)),
            "shell.max_nesting_depth"
        );
    }

    #[test]
    fn windows_wrapper_detection_case_insensitive_with_exe_suffix() {
        let cmd = extract_commands(r#"CMD.EXE /C "rm -rf /""#)
            .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        assert_eq!(cmd, vec!["rm -rf /"]);

        let powershell = extract_commands(r#"PowerShell.exe -command "rm -rf /""#)
            .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        assert_eq!(powershell, vec!["rm -rf /"]);
    }

    #[test]
    fn windows_wrapper_joins_all_remaining_arguments() {
        // `cmd /c` and `powershell -Command` consume all remaining tokens
        // as the inner command, not just the next token.
        let cmd = extract_commands("cmd /c rm -rf /")
            .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        assert_eq!(cmd, vec!["rm -rf /"]);

        let pwsh = extract_commands("pwsh -Command rm -rf /")
            .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        assert_eq!(pwsh, vec!["rm -rf /"]);
    }

    #[test]
    fn path_qualified_wrappers_are_detected() {
        // Unix path-qualified shell wrappers.
        let bash = extract_commands(r#"/bin/bash -c "rm -rf /""#)
            .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        assert_eq!(bash, vec!["rm -rf /"]);

        let sh = extract_commands(r#"/usr/bin/sh -c "echo hi""#)
            .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        assert_eq!(sh, vec!["echo hi"]);

        // Path-qualified env.
        let env = extract_commands(r#"/usr/bin/env bash -c "echo hi""#)
            .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        assert_eq!(env, vec!["echo hi"]);

        // Windows-style path with forward slashes (as seen in cross-platform
        // contexts — native backslash paths are treated as escape sequences by
        // the shell tokenizer, which is correct for Unix shell semantics).
        let cmd = extract_commands(r#"C:/Windows/System32/cmd.exe /c "echo hi""#)
            .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        assert_eq!(cmd, vec!["echo hi"]);
    }

    #[test]
    fn separator_splitting_and_quoting_behavior() {
        let commands = extract_commands("a && b || c ; d\ne & f")
            .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        assert_eq!(commands, vec!["a", "b", "c", "d", "e", "f"]);

        let quoted = extract_commands(r#"echo "a && b"; echo 'x || y'"#)
            .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        assert_eq!(quoted, vec![r#"echo "a && b""#, "echo 'x || y'"]);
    }

    #[test]
    fn substitution_extraction_behavior() {
        let literal = extract_commands(r#"echo '$(rm -rf /)'"#)
            .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        assert_eq!(literal, vec![r#"echo '$(rm -rf /)'"#]);

        let extracted = extract_commands(r#"echo "$(rm -rf /)""#)
            .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        assert_eq!(extracted, vec![r#"echo "$(rm -rf /)""#, "rm -rf /"]);

        // Backtick substitution inside double quotes is also extracted.
        let backtick = extract_commands(r#"echo "`rm -rf /`""#)
            .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        assert_eq!(backtick, vec![r#"echo "`rm -rf /`""#, "rm -rf /"]);

        // Backtick in single quotes is NOT extracted.
        let backtick_single = extract_commands(r#"echo '`rm -rf /`'"#)
            .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        assert_eq!(backtick_single, vec![r#"echo '`rm -rf /`'"#]);
    }

    #[test]
    fn newline_split_example() {
        let commands = extract_commands("git status\nrm -rf /")
            .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
        assert_eq!(commands, vec!["git status", "rm -rf /"]);
    }

    #[test]
    fn input_length_limit_denied() {
        let long = "x".repeat(MAX_COMMAND_BYTES + 1);
        assert_eq!(
            deny_rule_id(extract_commands(&long)),
            "shell.command_too_long"
        );
    }
}
