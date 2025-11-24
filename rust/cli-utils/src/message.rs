use std::env;

use textwrap::{Options, termwidth, wrap};
use unicode_width::UnicodeWidthStr;

/// Convenience macro for creating formatted CLI messages.
///
/// This macro wraps [`message`] to allow `format!`-style string interpolation.
/// If the message starts with an emoji, it will be automatically detected
/// and used as an icon with proper indentation for wrapped lines.
///
/// # Examples
///
/// ```ignore
/// // Simple message with emoji icon
/// message!("✅ Operation completed");
///
/// // Message with format arguments
/// message!("📁 Processing file: {filename}");
///
/// // Warning with dynamic content
/// message!("⚠️ Found {count} errors in {path}");
/// ```
#[macro_export]
macro_rules! message {
    ($($arg:tt)*) => {
        stencila_cli_utils::message(&format!($($arg)*))
    };
}

/// Create a wrapped and formatted CLI messages.
///
/// If the message starts with an emoji, it is automatically detected and used
/// as an icon with proper indentation for wrapped lines.
///
/// # Example
///
/// ```ignore
/// message("✅ Operation completed successfully");
/// message("⚠️ Warning: this may take a while");
/// message("Plain message without icon");
/// ```
#[allow(clippy::print_stderr)]
pub fn message(message: &str) {
    // Check if message contains URLs to determine if we should skip wrapping
    let contains_url = message.contains("http://") || message.contains("https://");

    // Extract leading emoji if present
    let (emoji, text) = match extract_leading_emoji(message) {
        Some((emoji, rest)) => (Some(emoji), rest),
        None => (None, message),
    };

    // Colorize the message text (if not NO_COLOR)
    let colored = if env::var_os("NO_COLOR").is_none() {
        colorize_message(text)
    } else {
        text.to_string()
    };

    // Skip wrapping if message contains URLs to avoid breaking it
    if contains_url {
        if let Some(emoji) = emoji {
            let (padding, _) = emoji_padding(emoji);
            eprintln!("{emoji}{padding}{colored}");
        } else {
            eprintln!("{colored}");
        }
    } else if let Some(emoji) = emoji {
        let (padding, indent_width) = emoji_padding(emoji);
        let subsequent_indent = " ".repeat(indent_width + padding.len());

        let options = Options::new(termwidth())
            .initial_indent("")
            .subsequent_indent(&subsequent_indent);

        let wrapped = wrap(&colored, options).join("\n");
        eprintln!("{emoji}{padding}{wrapped}");
    } else {
        let options = Options::new(termwidth());
        eprintln!("{}", wrap(&colored, options).join("\n"));
    }
}

/// Calculate padding and indent width for an emoji.
///
/// Returns `(padding, indent_width)` where:
/// - `padding`: The space string to insert after the emoji (" " or "  ")
/// - `indent_width`: The visual width to use for subsequent line indentation
///
/// Emojis with variation selectors (like ℹ️, ☁️, ⚙️) render inconsistently
/// across terminals, so they get extra padding for a consistent visible gap.
fn emoji_padding(emoji: &str) -> (&'static str, usize) {
    let has_variation_selector = emoji.contains('\u{FE0F}');
    let raw_width = UnicodeWidthStr::width(emoji);

    let padding = if has_variation_selector || raw_width <= 1 {
        "  "
    } else {
        " "
    };

    // For variation selector emojis, use width 3; otherwise at least 2
    let indent_width = if has_variation_selector {
        3
    } else {
        raw_width.max(2)
    };

    (padding, indent_width)
}

/// Extract leading emoji from a message string.
///
/// Returns `Some((emoji, rest))` if the message starts with an emoji,
/// where `emoji` is the full emoji (including any variation selectors)
/// and `rest` is the remainder of the message with leading whitespace trimmed.
///
/// Returns `None` if the message doesn't start with an emoji.
fn extract_leading_emoji(message: &str) -> Option<(&str, &str)> {
    let mut chars = message.chars();
    let first = chars.next()?;

    if !is_emoji_char(first) {
        return None;
    }

    // Find where the emoji ends (include variation selectors and ZWJ sequences)
    let mut emoji_end = first.len_utf8();
    let rest = &message[emoji_end..];

    for c in rest.chars() {
        if c == '\u{FE0F}' || c == '\u{FE0E}' || c == '\u{200D}' || is_emoji_char(c) {
            // Variation selector, zero-width joiner, or continuation emoji
            emoji_end += c.len_utf8();
        } else {
            break;
        }
    }

    let emoji = &message[..emoji_end];
    let rest = message[emoji_end..].trim_start();

    Some((emoji, rest))
}

/// Check if a character is an emoji
///
/// This checks for common emoji ranges and variation selectors.
fn is_emoji_char(c: char) -> bool {
    matches!(c,
        // Emoticons
        '\u{1F600}'..='\u{1F64F}' |
        // Misc symbols and pictographs
        '\u{1F300}'..='\u{1F5FF}' |
        // Transport and map symbols
        '\u{1F680}'..='\u{1F6FF}' |
        // Supplemental symbols and pictographs
        '\u{1F900}'..='\u{1F9FF}' |
        // Symbols and pictographs extended-A
        '\u{1FA00}'..='\u{1FA6F}' |
        // Symbols and pictographs extended-B
        '\u{1FA70}'..='\u{1FAFF}' |
        // Dingbats
        '\u{2700}'..='\u{27BF}' |
        // Misc symbols (includes ☁️, ⚙️, ✅, etc.)
        '\u{2600}'..='\u{26FF}' |
        // Letterlike symbols (includes ℹ️)
        '\u{2100}'..='\u{214F}' |
        // Arrows and other symbols often used as emoji
        '\u{2190}'..='\u{21FF}' |
        // Enclosed alphanumerics
        '\u{2460}'..='\u{24FF}' |
        // Box drawing (used for some symbols)
        '\u{25A0}'..='\u{25FF}' |
        // Misc technical (includes some symbols)
        '\u{2300}'..='\u{23FF}' |
        // Regional indicator symbols (flags)
        '\u{1F1E0}'..='\u{1F1FF}' |
        // Variation selectors (emoji presentation)
        '\u{FE00}'..='\u{FE0F}'
    )
}

/// Colorize angle-bracket-enclosed words (e.g., <domain>) in green
fn colorize_angle_brackets(content: &str) -> String {
    let mut result = String::new();
    let mut chars = content.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '<' {
            // Found opening angle bracket, look for closing bracket
            let mut word = String::new();
            let mut found_closing = false;

            for inner_ch in chars.by_ref() {
                if inner_ch == '>' {
                    found_closing = true;
                    break;
                }
                word.push(inner_ch);
            }

            // Only colorize if it's a single word (no spaces) and not empty
            if found_closing && !word.is_empty() && !word.contains(char::is_whitespace) {
                // Reset blue, add green coloring for the angle-bracketed word, then restore blue
                result.push_str(&format!("\x1b[0m\x1b[92m<{word}>\x1b[0m\x1b[94m"));
            } else {
                // Not a valid pattern, keep original
                result.push('<');
                result.push_str(&word);
                if found_closing {
                    result.push('>');
                }
            }
        } else {
            result.push(ch);
        }
    }
    result
}

/// Add color to backticked content, asterisk pairs, and URLs
fn colorize_message(message: &str) -> String {
    let mut result = String::new();
    let mut chars = message.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '`' {
            // Found opening backtick, look for closing backtick
            let mut content = String::new();
            let mut found_closing = false;

            for inner_ch in chars.by_ref() {
                if inner_ch == '`' {
                    found_closing = true;
                    break;
                }
                content.push(inner_ch);
            }

            if found_closing && !content.is_empty() {
                // Add orange coloring around the content
                result.push_str(&format!("\x1b[38;5;208m{content}\x1b[0m"));
            } else {
                // No closing backtick found or empty content, keep original
                result.push('`');
                result.push_str(&content);
                if found_closing {
                    result.push('`');
                }
            }
        } else if ch == '*' {
            // Found opening asterisk, look for closing asterisk
            let mut content = String::new();
            let mut found_closing = false;

            for inner_ch in chars.by_ref() {
                if inner_ch == '*' {
                    found_closing = true;
                    break;
                }
                content.push(inner_ch);
            }

            if found_closing && !content.is_empty() {
                // Process content to color angle-bracket-enclosed words green
                let colored_content = colorize_angle_brackets(&content);
                // Add blue coloring around the content (angle brackets will override)
                result.push_str(&format!("\x1b[94m{colored_content}\x1b[0m"));
            } else {
                // No closing asterisk found or empty content, keep original
                result.push('*');
                result.push_str(&content);
                if found_closing {
                    result.push('*');
                }
            }
        } else if ch == 'h' {
            // Check if this is the start of a URL
            // First, peek ahead to see if we have "ttp://" or "ttps://"
            // We'll collect characters into a buffer to check, but only consume from
            // the iterator once we know it's a URL

            // Collect up to 7 more characters (for "ttps://") into a temporary buffer
            let mut temp_buffer = Vec::new();
            let mut temp_iter = chars.clone();
            for _ in 0..7 {
                if let Some(ch) = temp_iter.next() {
                    temp_buffer.push(ch);
                } else {
                    break;
                }
            }

            // Check if the buffer matches "ttp://" or "ttps://"
            let buffer_str: String = temp_buffer.iter().collect();
            let is_http = buffer_str.starts_with("ttp://");
            let is_https = buffer_str.starts_with("ttps://");

            if is_http || is_https {
                // This is a URL! Now consume the characters from the real iterator
                let prefix_len = if is_https { 7 } else { 6 };
                let mut url = String::from('h');

                // Consume the protocol prefix
                for _ in 0..prefix_len {
                    if let Some(ch) = chars.next() {
                        url.push(ch);
                    }
                }

                let mut trailing_whitespace = None;

                // Collect the rest of the URL until whitespace or end
                for next_ch in chars.by_ref() {
                    if next_ch.is_whitespace() {
                        trailing_whitespace = Some(next_ch);
                        break;
                    }
                    url.push(next_ch);
                }

                // Color the URL blue
                result.push_str(&format!("\x1b[94m{url}\x1b[0m"));

                // Add trailing whitespace if there was any
                if let Some(ws) = trailing_whitespace {
                    result.push(ws);
                }
            } else {
                // Not a URL, just push the 'h'
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colorize_backticks() {
        let input = "Use the `foo` function";
        let output = colorize_message(input);
        assert!(
            output.contains("\x1b[38;5;208mfoo\x1b[0m"),
            "Backticked content should be colored orange"
        );
    }

    #[test]
    fn test_colorize_unclosed_backticks() {
        let input = "Unclosed `backtick";
        let output = colorize_message(input);
        assert_eq!(
            output, "Unclosed `backtick",
            "Unclosed backticks should remain unchanged"
        );
    }

    #[test]
    fn test_colorize_https_url() {
        let input = "Visit https://example.com for more";
        let output = colorize_message(input);
        assert!(
            output.contains("\x1b[94mhttps://example.com\x1b[0m"),
            "HTTPS URLs should be colored blue"
        );
    }

    #[test]
    fn test_colorize_http_url() {
        let input = "Visit http://example.com for more";
        let output = colorize_message(input);
        assert!(
            output.contains("\x1b[94mhttp://example.com\x1b[0m"),
            "HTTP URLs should be colored blue"
        );
    }

    #[test]
    fn test_colorize_url_at_end() {
        let input = "Visit https://example.com";
        let output = colorize_message(input);
        assert!(
            output.contains("\x1b[94mhttps://example.com\x1b[0m"),
            "URLs at the end should be colored"
        );
    }

    #[test]
    fn test_colorize_url_with_path() {
        let input = "Check https://example.com/path/to/page";
        let output = colorize_message(input);
        assert!(
            output.contains("\x1b[94mhttps://example.com/path/to/page\x1b[0m"),
            "URLs with paths should be fully colored"
        );
    }

    #[test]
    fn test_colorize_mixed_content() {
        let input = "Use `command` to fetch https://example.com data";
        let output = colorize_message(input);
        assert!(
            output.contains("\x1b[38;5;208mcommand\x1b[0m"),
            "Backticked content should be colored"
        );
        assert!(
            output.contains("\x1b[94mhttps://example.com\x1b[0m"),
            "URL should be colored"
        );
    }

    #[test]
    fn test_colorize_not_url() {
        let input = "The word http does not start a URL";
        let output = colorize_message(input);
        assert_eq!(output, input, "Non-URL content should remain unchanged");
    }

    #[test]
    fn test_colorize_asterisks() {
        let input = "This is *important* text";
        let output = colorize_message(input);
        assert!(
            output.contains("\x1b[94mimportant\x1b[0m"),
            "Asterisk-wrapped content should be colored blue"
        );
    }

    #[test]
    fn test_colorize_unclosed_asterisks() {
        let input = "Unclosed *asterisk";
        let output = colorize_message(input);
        assert_eq!(
            output, "Unclosed *asterisk",
            "Unclosed asterisks should remain unchanged"
        );
    }

    #[test]
    fn test_colorize_mixed_backticks_and_asterisks() {
        let input = "Use `command` for *emphasis*";
        let output = colorize_message(input);
        assert!(
            output.contains("\x1b[38;5;208mcommand\x1b[0m"),
            "Backticked content should be colored orange"
        );
        assert!(
            output.contains("\x1b[94memphasis\x1b[0m"),
            "Asterisk-wrapped content should be colored blue"
        );
    }

    #[test]
    fn test_colorize_angle_brackets_in_asterisks() {
        let input = "*stencila site domain set <domain>*";
        let output = colorize_message(input);
        assert!(
            output.contains("\x1b[92m<domain>\x1b[0m"),
            "Angle-bracketed arguments should be colored green"
        );
        assert!(
            output.contains("\x1b[94m"),
            "Non-argument content should be colored blue"
        );
    }

    #[test]
    fn test_colorize_multiple_angle_brackets() {
        let input = "*command <arg1> and <arg2>*";
        let output = colorize_message(input);
        assert!(
            output.contains("\x1b[92m<arg1>\x1b[0m"),
            "First argument should be colored green"
        );
        assert!(
            output.contains("\x1b[92m<arg2>\x1b[0m"),
            "Second argument should be colored green"
        );
    }

    #[test]
    fn test_colorize_angle_brackets_with_spaces() {
        let input = "*text <not a word>*";
        let output = colorize_message(input);
        // Angle brackets with spaces should not be specially colored
        assert!(
            !output.contains("\x1b[92m<not a word>\x1b[0m"),
            "Angle brackets with spaces should not be specially colored"
        );
    }

    #[test]
    fn test_emoji_char_detection() {
        // Test narrow emojis with variation selectors
        assert!(is_emoji_char('\u{2139}')); // ℹ (info)
        assert!(is_emoji_char('\u{2601}')); // ☁ (cloud)
        assert!(is_emoji_char('\u{2699}')); // ⚙ (gear)

        // Test wide emojis
        assert!(is_emoji_char('\u{1F4C1}')); // 📁 (folder)
        assert!(is_emoji_char('\u{2705}')); // ✅ (check mark)

        // Test variation selector
        assert!(is_emoji_char('\u{FE0F}')); // variation selector
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_emoji_extraction() {
        // Test extraction of narrow emojis with variation selectors
        let (emoji, rest) = extract_leading_emoji("ℹ️ Info message").unwrap();
        assert_eq!(emoji, "ℹ️");
        assert_eq!(rest, "Info message");

        let (emoji, rest) = extract_leading_emoji("☁️ Cloud message").unwrap();
        assert_eq!(emoji, "☁️");
        assert_eq!(rest, "Cloud message");

        let (emoji, rest) = extract_leading_emoji("⚙️ Gear message").unwrap();
        assert_eq!(emoji, "⚙️");
        assert_eq!(rest, "Gear message");

        // Test extraction of wide emojis
        let (emoji, rest) = extract_leading_emoji("📁 Folder message").unwrap();
        assert_eq!(emoji, "📁");
        assert_eq!(rest, "Folder message");
    }
}
