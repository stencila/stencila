//! Shell tokenizer and scanning helpers used by the shell tool guard.
//!
//! This module provides:
//! - shlex-style tokenization with quote and nesting awareness
//! - separator splitting (`&&`, `||`, `;`, `&`, `\n`) that respects quoting
//! - command substitution extraction (`$(...)` and backticks)
//! - pipe splitting utility used by validator pre-processing

use thiserror::Error;

/// Quoting / nesting context encountered while parsing a token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuoteContext {
    Unquoted,
    SingleQuoted,
    DoubleQuoted,
    CommandSubstitution,
    BacktickSubstitution,
}

/// A shell token with the parsed value and contexts observed while building it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub value: String,
    pub contexts: Vec<QuoteContext>,
}

/// A shell token with source byte offsets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpannedToken {
    pub value: String,
    pub contexts: Vec<QuoteContext>,
    pub start: usize,
    pub end: usize,
}

/// Parse failures in shell tokenization / scanning.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParseError {
    #[error("unmatched single quote")]
    UnmatchedSingleQuote,

    #[error("unmatched double quote")]
    UnmatchedDoubleQuote,

    #[error("unmatched command substitution parentheses")]
    UnmatchedParentheses,

    #[error("unmatched backtick")]
    UnmatchedBacktick,

    #[error("invalid escape sequence")]
    InvalidEscapeSequence,

    #[error("missing argument for flag `{flag}`")]
    MissingArgument { flag: String },

    #[error("unrecognized env flag `{flag}`")]
    UnrecognizedEnvFlag { flag: String },
}

/// Tokenize a shell command string using shlex-style splitting.
pub fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    tokenize_spanned(input).map(|tokens| {
        tokens
            .into_iter()
            .map(|token| Token {
                value: token.value,
                contexts: token.contexts,
            })
            .collect()
    })
}

/// Tokenize a shell command string using shlex-style splitting, including
/// source byte offsets for each token.
pub fn tokenize_spanned(input: &str) -> Result<Vec<SpannedToken>, ParseError> {
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < input.len() {
        i = skip_whitespace(input, i);
        if i >= input.len() {
            break;
        }

        let token_start = i;

        let mut value = String::new();
        let mut contexts = Vec::new();
        let mut started = false;

        while i < input.len() {
            let Some((ch, ch_len)) = next_char(input, i) else {
                break;
            };

            if ch.is_whitespace() {
                break;
            }

            if input[i..].starts_with('\\') {
                let escaped_end = consume_escaped_any(input, i)?;
                value.push_str(&input[i + 1..escaped_end]);
                push_context(&mut contexts, QuoteContext::Unquoted);
                started = true;
                i = escaped_end;
                continue;
            }

            if input[i..].starts_with('"') {
                i = append_double_quoted(input, i, &mut value, &mut contexts)?;
                started = true;
                continue;
            }

            if input[i..].starts_with('\'') {
                let end = scan_single_quoted(input, i)?;
                value.push_str(&input[i + 1..end - 1]);
                push_context(&mut contexts, QuoteContext::SingleQuoted);
                started = true;
                i = end;
                continue;
            }

            if input[i..].starts_with("$((") {
                let end = scan_arithmetic_expansion(input, i)?;
                value.push_str(&input[i..end]);
                started = true;
                i = end;
                continue;
            }

            if input[i..].starts_with("$(") {
                let end = scan_command_substitution(input, i)?;
                value.push_str(&input[i..end]);
                push_context(&mut contexts, QuoteContext::CommandSubstitution);
                started = true;
                i = end;
                continue;
            }

            if input[i..].starts_with('`') {
                let end = scan_backtick_substitution(input, i)?;
                value.push_str(&input[i..end]);
                push_context(&mut contexts, QuoteContext::BacktickSubstitution);
                started = true;
                i = end;
                continue;
            }

            value.push(ch);
            push_context(&mut contexts, QuoteContext::Unquoted);
            started = true;
            i += ch_len;
        }

        if started {
            tokens.push(SpannedToken {
                value,
                contexts,
                start: token_start,
                end: i,
            });
        }
    }

    Ok(tokens)
}

/// Split by shell command separators: `&&`, `||`, `;`, `&`, and newline.
///
/// Longest-match-first is applied for `&&` and `||` before single-char `&`.
pub fn split_on_separators(input: &str) -> Result<Vec<String>, ParseError> {
    let mut segments = Vec::new();
    let mut start = 0;
    let mut i = 0;

    while i < input.len() {
        if input[i..].starts_with('\\') {
            i = consume_escaped_any(input, i)?;
            continue;
        }

        if input[i..].starts_with('\'') {
            i = scan_single_quoted(input, i)?;
            continue;
        }

        if input[i..].starts_with('"') {
            i = scan_double_quoted(input, i)?;
            continue;
        }

        if input[i..].starts_with('`') {
            i = scan_backtick_substitution(input, i)?;
            continue;
        }

        if input[i..].starts_with("$((") {
            i = scan_arithmetic_expansion(input, i)?;
            continue;
        }

        if input[i..].starts_with("$(") {
            i = scan_command_substitution(input, i)?;
            continue;
        }

        if input[i..].starts_with("&&") || input[i..].starts_with("||") {
            push_owned_segment(&mut segments, &input[start..i]);
            i += 2;
            start = i;
            continue;
        }

        let Some((ch, ch_len)) = next_char(input, i) else {
            break;
        };

        if matches!(ch, ';' | '&' | '\n') {
            push_owned_segment(&mut segments, &input[start..i]);
            i += ch_len;
            start = i;
            continue;
        }

        i += ch_len;
    }

    push_owned_segment(&mut segments, &input[start..]);
    Ok(segments)
}

/// Extract top-level command substitutions from unquoted and double-quoted
/// contexts.
///
/// Substitutions inside single quotes are treated as literals and not extracted.
pub fn extract_substitutions(input: &str) -> Result<Vec<String>, ParseError> {
    let mut substitutions = Vec::new();
    let mut i = 0;

    while i < input.len() {
        if input[i..].starts_with('\\') {
            i = consume_escaped_any(input, i)?;
            continue;
        }

        if input[i..].starts_with('\'') {
            i = scan_single_quoted(input, i)?;
            continue;
        }

        if input[i..].starts_with('"') {
            i = extract_substitutions_in_double(input, i, &mut substitutions)?;
            continue;
        }

        if input[i..].starts_with("$((") {
            i = scan_arithmetic_expansion(input, i)?;
            continue;
        }

        if input[i..].starts_with("$(") {
            let end = scan_command_substitution(input, i)?;
            substitutions.push(input[i + 2..end - 1].to_string());
            i = end;
            continue;
        }

        if input[i..].starts_with('`') {
            let end = scan_backtick_substitution(input, i)?;
            substitutions.push(input[i + 1..end - 1].to_string());
            i = end;
            continue;
        }

        let Some((_, ch_len)) = next_char(input, i) else {
            break;
        };
        i += ch_len;
    }

    Ok(substitutions)
}

/// Split by pipes while respecting shell quoting and nesting context.
///
/// This API is intentionally infallible for convenience; malformed input falls
/// back to returning the original string as a single segment.
pub fn pipe_split(input: &str) -> Vec<&str> {
    match pipe_split_checked(input) {
        Ok(parts) => parts,
        Err(_) => vec![input],
    }
}

/// Checked variant of [`pipe_split`] that surfaces parse failures.
pub fn pipe_split_checked(input: &str) -> Result<Vec<&str>, ParseError> {
    let mut segments = Vec::new();
    let mut start = 0;
    let mut i = 0;

    while i < input.len() {
        if input[i..].starts_with('\\') {
            i = consume_escaped_any(input, i)?;
            continue;
        }

        if input[i..].starts_with('\'') {
            i = scan_single_quoted(input, i)?;
            continue;
        }

        if input[i..].starts_with('"') {
            i = scan_double_quoted(input, i)?;
            continue;
        }

        if input[i..].starts_with('`') {
            i = scan_backtick_substitution(input, i)?;
            continue;
        }

        if input[i..].starts_with("$((") {
            i = scan_arithmetic_expansion(input, i)?;
            continue;
        }

        if input[i..].starts_with("$(") {
            i = scan_command_substitution(input, i)?;
            continue;
        }

        if input[i..].starts_with("||") {
            i += 2;
            continue;
        }

        let Some((ch, ch_len)) = next_char(input, i) else {
            break;
        };
        if ch == '|' {
            push_borrowed_segment(&mut segments, &input[start..i]);
            i += ch_len;
            start = i;
            continue;
        }

        i += ch_len;
    }

    push_borrowed_segment(&mut segments, &input[start..]);
    Ok(segments)
}

fn append_double_quoted(
    input: &str,
    start: usize,
    out: &mut String,
    contexts: &mut Vec<QuoteContext>,
) -> Result<usize, ParseError> {
    let mut i = start + 1;
    push_context(contexts, QuoteContext::DoubleQuoted);

    while i < input.len() {
        if input[i..].starts_with('"') {
            return Ok(i + 1);
        }

        if input[i..].starts_with('\\') {
            let Some((escaped, escaped_len)) = next_char(input, i + 1) else {
                return Err(ParseError::InvalidEscapeSequence);
            };

            match escaped {
                '"' | '\\' | '$' | '`' => {
                    out.push(escaped);
                    i += 1 + escaped_len;
                }
                '\n' => {
                    // Line continuation inside double quotes.
                    i += 1 + escaped_len;
                }
                _ => {
                    // Preserve literal backslash for non-special escapes.
                    out.push('\\');
                    out.push(escaped);
                    i += 1 + escaped_len;
                }
            }
            continue;
        }

        if input[i..].starts_with("$((") {
            let end = scan_arithmetic_expansion(input, i)?;
            out.push_str(&input[i..end]);
            i = end;
            continue;
        }

        if input[i..].starts_with("$(") {
            let end = scan_command_substitution(input, i)?;
            out.push_str(&input[i..end]);
            push_context(contexts, QuoteContext::CommandSubstitution);
            i = end;
            continue;
        }

        if input[i..].starts_with('`') {
            let end = scan_backtick_substitution(input, i)?;
            out.push_str(&input[i..end]);
            push_context(contexts, QuoteContext::BacktickSubstitution);
            i = end;
            continue;
        }

        let Some((ch, ch_len)) = next_char(input, i) else {
            break;
        };
        out.push(ch);
        i += ch_len;
    }

    Err(ParseError::UnmatchedDoubleQuote)
}

fn extract_substitutions_in_double(
    input: &str,
    start: usize,
    substitutions: &mut Vec<String>,
) -> Result<usize, ParseError> {
    let mut i = start + 1;

    while i < input.len() {
        if input[i..].starts_with('"') {
            return Ok(i + 1);
        }

        if input[i..].starts_with('\\') {
            let Some((escaped, escaped_len)) = next_char(input, i + 1) else {
                return Err(ParseError::InvalidEscapeSequence);
            };
            match escaped {
                '"' | '\\' | '$' | '`' | '\n' => {
                    i += 1 + escaped_len;
                }
                _ => {
                    // Non-special escapes are literal in shell double quotes.
                    i += 1 + escaped_len;
                }
            }
            continue;
        }

        if input[i..].starts_with("$((") {
            i = scan_arithmetic_expansion(input, i)?;
            continue;
        }

        if input[i..].starts_with("$(") {
            let end = scan_command_substitution(input, i)?;
            substitutions.push(input[i + 2..end - 1].to_string());
            i = end;
            continue;
        }

        if input[i..].starts_with('`') {
            let end = scan_backtick_substitution(input, i)?;
            substitutions.push(input[i + 1..end - 1].to_string());
            i = end;
            continue;
        }

        let Some((_, ch_len)) = next_char(input, i) else {
            break;
        };
        i += ch_len;
    }

    Err(ParseError::UnmatchedDoubleQuote)
}

fn scan_single_quoted(input: &str, start: usize) -> Result<usize, ParseError> {
    let mut i = start + 1;
    while i < input.len() {
        let Some((ch, ch_len)) = next_char(input, i) else {
            break;
        };
        if ch == '\'' {
            return Ok(i + ch_len);
        }
        i += ch_len;
    }
    Err(ParseError::UnmatchedSingleQuote)
}

fn scan_double_quoted(input: &str, start: usize) -> Result<usize, ParseError> {
    let mut i = start + 1;

    while i < input.len() {
        if input[i..].starts_with('"') {
            return Ok(i + 1);
        }

        if input[i..].starts_with('\\') {
            let Some((escaped, escaped_len)) = next_char(input, i + 1) else {
                return Err(ParseError::InvalidEscapeSequence);
            };
            match escaped {
                '"' | '\\' | '$' | '`' | '\n' => {
                    i += 1 + escaped_len;
                }
                _ => {
                    // Non-special escapes are literal in shell double quotes.
                    i += 1 + escaped_len;
                }
            }
            continue;
        }

        if input[i..].starts_with("$((") {
            i = scan_arithmetic_expansion(input, i)?;
            continue;
        }

        if input[i..].starts_with("$(") {
            i = scan_command_substitution(input, i)?;
            continue;
        }

        if input[i..].starts_with('`') {
            i = scan_backtick_substitution(input, i)?;
            continue;
        }

        let Some((_, ch_len)) = next_char(input, i) else {
            break;
        };
        i += ch_len;
    }

    Err(ParseError::UnmatchedDoubleQuote)
}

fn scan_command_substitution(input: &str, start: usize) -> Result<usize, ParseError> {
    let mut i = start + 2;
    let mut depth = 1usize;

    while i < input.len() {
        if input[i..].starts_with('\\') {
            i = consume_escaped_any(input, i)?;
            continue;
        }

        if input[i..].starts_with('\'') {
            i = scan_single_quoted(input, i)?;
            continue;
        }

        if input[i..].starts_with('"') {
            i = scan_double_quoted(input, i)?;
            continue;
        }

        if input[i..].starts_with('`') {
            i = scan_backtick_substitution(input, i)?;
            continue;
        }

        if input[i..].starts_with("$((") {
            i = scan_arithmetic_expansion(input, i)?;
            continue;
        }

        if input[i..].starts_with("$(") {
            depth += 1;
            i += 2;
            continue;
        }

        let Some((ch, ch_len)) = next_char(input, i) else {
            break;
        };

        match ch {
            '(' => {
                depth += 1;
                i += ch_len;
            }
            ')' => {
                depth = depth.saturating_sub(1);
                i += ch_len;
                if depth == 0 {
                    return Ok(i);
                }
            }
            _ => i += ch_len,
        }
    }

    Err(ParseError::UnmatchedParentheses)
}

fn scan_arithmetic_expansion(input: &str, start: usize) -> Result<usize, ParseError> {
    // Starts with `$((`
    let mut i = start + 3;
    let mut depth = 2usize;

    while i < input.len() {
        if input[i..].starts_with('\\') {
            i = consume_escaped_any(input, i)?;
            continue;
        }

        if input[i..].starts_with('\'') {
            i = scan_single_quoted(input, i)?;
            continue;
        }

        if input[i..].starts_with('"') {
            i = scan_double_quoted(input, i)?;
            continue;
        }

        if input[i..].starts_with('`') {
            i = scan_backtick_substitution(input, i)?;
            continue;
        }

        if input[i..].starts_with("$((") {
            depth += 2;
            i += 3;
            continue;
        }

        if input[i..].starts_with("$(") {
            i = scan_command_substitution(input, i)?;
            continue;
        }

        let Some((ch, ch_len)) = next_char(input, i) else {
            break;
        };

        match ch {
            '(' => {
                depth += 1;
                i += ch_len;
            }
            ')' => {
                depth = depth.saturating_sub(1);
                i += ch_len;
                if depth == 0 {
                    return Ok(i);
                }
            }
            _ => i += ch_len,
        }
    }

    Err(ParseError::UnmatchedParentheses)
}

fn scan_backtick_substitution(input: &str, start: usize) -> Result<usize, ParseError> {
    let mut i = start + 1;

    while i < input.len() {
        if input[i..].starts_with('\\') {
            i = consume_escaped_any(input, i)?;
            continue;
        }

        if input[i..].starts_with('`') {
            return Ok(i + 1);
        }

        if input[i..].starts_with("$((") {
            i = scan_arithmetic_expansion(input, i)?;
            continue;
        }

        if input[i..].starts_with("$(") {
            i = scan_command_substitution(input, i)?;
            continue;
        }

        if input[i..].starts_with('\'') {
            i = scan_single_quoted(input, i)?;
            continue;
        }

        if input[i..].starts_with('"') {
            i = scan_double_quoted(input, i)?;
            continue;
        }

        let Some((_, ch_len)) = next_char(input, i) else {
            break;
        };
        i += ch_len;
    }

    Err(ParseError::UnmatchedBacktick)
}

fn consume_escaped_any(input: &str, start: usize) -> Result<usize, ParseError> {
    let Some((_, next_len)) = next_char(input, start + 1) else {
        return Err(ParseError::InvalidEscapeSequence);
    };
    Ok(start + 1 + next_len)
}

fn push_context(contexts: &mut Vec<QuoteContext>, context: QuoteContext) {
    if !contexts.contains(&context) {
        contexts.push(context);
    }
}

fn push_owned_segment(segments: &mut Vec<String>, segment: &str) {
    let trimmed = segment.trim();
    if !trimmed.is_empty() {
        segments.push(trimmed.to_string());
    }
}

fn push_borrowed_segment<'a>(segments: &mut Vec<&'a str>, segment: &'a str) {
    let trimmed = segment.trim();
    if !trimmed.is_empty() {
        segments.push(trimmed);
    }
}

fn skip_whitespace(input: &str, mut i: usize) -> usize {
    while i < input.len() {
        let Some((ch, ch_len)) = next_char(input, i) else {
            break;
        };
        if !ch.is_whitespace() {
            break;
        }
        i += ch_len;
    }
    i
}

fn next_char(input: &str, i: usize) -> Option<(char, usize)> {
    input[i..].chars().next().map(|ch| (ch, ch.len_utf8()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token_values(tokens: &[Token]) -> Vec<String> {
        tokens.iter().map(|token| token.value.clone()).collect()
    }

    #[test]
    fn tokenize_simple_command() {
        let tokens = match tokenize("git status") {
            Ok(tokens) => tokens,
            Err(error) => panic!("unexpected tokenize error: {error}"),
        };
        assert_eq!(token_values(&tokens), vec!["git", "status"]);
    }

    #[test]
    fn tokenize_quoted_and_escaped() {
        let tokens = match tokenize(r#"echo "hello world" hello\ world"#) {
            Ok(tokens) => tokens,
            Err(error) => panic!("unexpected tokenize error: {error}"),
        };

        assert_eq!(
            token_values(&tokens),
            vec!["echo", "hello world", "hello world"]
        );
        assert!(tokens[1].contexts.contains(&QuoteContext::DoubleQuoted));
    }

    #[test]
    fn tokenize_double_quoted_non_special_escapes_are_literal() {
        let tokens = match tokenize(r#"echo "\$HOME \`date\` C:\\tmp\foo""#) {
            Ok(tokens) => tokens,
            Err(error) => panic!("unexpected tokenize error: {error}"),
        };

        assert_eq!(
            token_values(&tokens),
            vec!["echo", "$HOME `date` C:\\tmp\\foo"]
        );
    }

    #[test]
    fn tokenize_nested_quotes_and_substitutions() {
        let tokens = match tokenize(r#"echo "$(printf '%s' "$(whoami)")""#) {
            Ok(tokens) => tokens,
            Err(error) => panic!("unexpected tokenize error: {error}"),
        };

        assert_eq!(tokens[0].value, "echo");
        assert_eq!(tokens[1].value, "$(printf '%s' \"$(whoami)\")");
        assert!(tokens[1].contexts.contains(&QuoteContext::DoubleQuoted));
        assert!(
            tokens[1]
                .contexts
                .contains(&QuoteContext::CommandSubstitution)
        );
    }

    #[test]
    fn tokenize_reports_unmatched_quotes_and_parens() {
        assert!(matches!(
            tokenize("echo 'unterminated"),
            Err(ParseError::UnmatchedSingleQuote)
        ));
        assert!(matches!(
            tokenize("echo \"unterminated"),
            Err(ParseError::UnmatchedDoubleQuote)
        ));
        assert!(matches!(
            tokenize("echo $(unterminated"),
            Err(ParseError::UnmatchedParentheses)
        ));
    }

    #[test]
    fn split_separators_respects_quotes_and_nesting() {
        let parts = match split_on_separators(r#"echo "a && b" && echo $(echo "x ; y") ; done"#) {
            Ok(parts) => parts,
            Err(error) => panic!("unexpected split error: {error}"),
        };
        assert_eq!(
            parts,
            vec!["echo \"a && b\"", "echo $(echo \"x ; y\")", "done"]
        );
    }

    #[test]
    fn extract_substitutions_respects_single_quotes() {
        let substitutions =
            match extract_substitutions(r#"echo '$(rm -rf /)' \"$(whoami)\" $(pwd) `id`"#) {
                Ok(substitutions) => substitutions,
                Err(error) => panic!("unexpected extraction error: {error}"),
            };

        assert_eq!(substitutions, vec!["whoami", "pwd", "id"]);
    }

    #[test]
    fn pipe_split_respects_context() {
        let parts = match pipe_split_checked(r#"echo "a|b" | grep x || echo y | wc -l"#) {
            Ok(parts) => parts,
            Err(error) => panic!("unexpected pipe split error: {error}"),
        };
        assert_eq!(parts, vec!["echo \"a|b\"", "grep x || echo y", "wc -l"]);
    }
}
