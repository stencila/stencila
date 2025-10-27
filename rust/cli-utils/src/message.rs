use std::env;

use textwrap::{Options, termwidth, wrap};

/// Create a wrapped and formatted user message on the terminal
#[allow(clippy::print_stderr)]
pub fn message(message: &str, icon: Option<&str>) {
    // Check if message contains URLs to determine if we should skip wrapping
    let contains_url = message.contains("http://") || message.contains("https://");

    // Colorize the message (if not NO_COLOR)
    let colored = if env::var_os("NO_COLOR").is_none() {
        colorize_message(message)
    } else {
        message.to_string()
    };

    // Skip wrapping if message contains URLs to avoid breaking it
    if contains_url {
        if let Some(icon) = icon {
            eprintln!("{icon}  {colored}");
        } else {
            eprintln!("{colored}");
        }
    } else {
        let initial_indent = icon.map(|icon| [icon, "  "].concat());
        let subsequent_indent = initial_indent
            .as_ref()
            .map(|indent| " ".repeat(indent.chars().count()));

        let options = if let (Some(initial_indent), Some(subsequent_indent)) =
            (&initial_indent, &subsequent_indent)
        {
            Options::new(termwidth())
                .initial_indent(initial_indent)
                .subsequent_indent(subsequent_indent)
        } else {
            Options::new(termwidth())
        };

        eprintln!("{}", wrap(&colored, options).join("\n"));
    }
}

/// Add color to backticked content and URLs
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

#[macro_export]
macro_rules! message {
    ($str:literal, $($arg:tt)*) => {
        stencila_cli_utils::message(&format!($str, $($arg)*), None)
    };

    ($str:literal) => {
        stencila_cli_utils::message($str, None)
    };
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
}
