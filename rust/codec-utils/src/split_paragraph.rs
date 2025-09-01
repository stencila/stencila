use eyre::{Result, eyre};

/// Parses a string into column width and a sentence-splitting flag.
///
/// If the string begins or ends with 's', sets `split_sentences` to true; otherwise false.
///
/// # Arguments
///
/// * `arg` - A string like "80" or "50s".
///
/// # Returns
///
/// * `Ok((width, split_sentences))` on success.
/// * `Err` with a message on parse failure.
pub fn parse_args(arg: &str) -> Result<(u32, bool)> {
    let arg = arg.trim();
    if arg == "s" {
        Ok((1000, true))
    } else if let Some(num_part) = arg.strip_suffix('s').or(arg.strip_prefix('s')) {
        num_part
            .parse::<u32>()
            .map(|n| (n, true))
            .map_err(|_| eyre!("Invalid number: {}", num_part))
    } else {
        arg.parse::<u32>()
            .map(|n| (n, false))
            .map_err(|_| eyre!("Invalid number: {}", arg))
    }
}

/// Splits a single-line LaTeX paragraph into multiple lines.
///
/// # Arguments
///
/// * `paragraph` - The input paragraph string.
/// * `col_width` - Maximum number of characters per line.
/// * `split_sentences` - If true, lines will end at sentence boundaries when possible.
///
/// # Returns
///
/// A vector of strings, each representing a wrapped line.
pub fn split(paragraph: &str, col_width: u32, split_sentences: bool) -> Vec<String> {
    let width = col_width as usize;
    if paragraph.len() <= width && !split_sentences {
        return vec![paragraph.to_string()];
    }

    // Helper: word-wrap a single chunk (sentence) to lines of max `width`.
    fn wrap_chunk(chunk: &str, width: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current = String::new();
        for word in chunk.split_whitespace() {
            if !current.is_empty() {
                let next_len = current.len() + 1 + word.len();
                if next_len > width {
                    lines.push(current.clone());
                    current.clear();
                } else {
                    current.push(' ');
                }
            }
            current.push_str(word);
        }
        if !current.is_empty() {
            lines.push(current);
        }
        lines
    }

    if split_sentences {
        // Sentence splitting: split at '.', '!', '?' followed by whitespace
        let mut sentences = Vec::new();
        let mut buf = String::new();
        let mut chars = paragraph.chars().peekable();
        while let Some(c) = chars.next() {
            buf.push(c);
            if (c == '.' || c == '!' || c == '?')
                && let Some(&next) = chars.peek()
                && next.is_whitespace()
            {
                let trimmed = buf.trim().to_string();
                if !trimmed.is_empty() {
                    sentences.push(trimmed);
                }
                buf.clear();
                // consume whitespace
                while let Some(&w) = chars.peek() {
                    if w.is_whitespace() {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
        }
        if !buf.trim().is_empty() {
            sentences.push(buf.trim().to_string());
        }

        let mut result = Vec::new();
        let mut current_line = String::new();

        for sentence in sentences {
            let sent = sentence.trim();
            if sent.len() > width {
                if !current_line.is_empty() {
                    result.push(current_line.clone());
                    current_line.clear();
                }
                result.extend(wrap_chunk(sent, width));
            } else if current_line.is_empty() {
                current_line.push_str(sent);
            } else if current_line.len() + 1 + sent.len() <= width {
                current_line.push(' ');
                current_line.push_str(sent);
            } else {
                result.push(current_line.clone());
                current_line.clear();
                current_line.push_str(sent);
            }
        }
        if !current_line.is_empty() {
            result.push(current_line);
        }
        result
    } else {
        // Just word-wrap the whole paragraph
        wrap_chunk(paragraph, width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_args() -> Result<()> {
        assert_eq!(parse_args("80")?, (80, false));
        assert_eq!(parse_args("50s")?, (50, true));
        assert_eq!(parse_args("s")?, (1000, true));
        assert!(parse_args("xyz").is_err());

        Ok(())
    }

    #[test]
    fn test_no_wrap_needed() {
        let p = "Short paragraph.";
        let lines = split(p, 50, false);
        assert_eq!(lines, vec![p]);
    }

    #[test]
    fn test_simple_wrap() {
        let p = "This is a simple test of the split_paragraph function.";
        let lines = split(p, 25, false);
        assert_eq!(
            lines,
            vec![
                "This is a simple test of".to_string(),
                "the split_paragraph".to_string(),
                "function.".to_string(),
            ]
        );
    }

    #[test]
    fn test_sentence_boundary_split() {
        let p = "First sentence. Second one is a bit longer than first. Third.";
        let lines = split(p, 25, true);
        assert_eq!(
            lines,
            vec![
                "First sentence.".to_string(),
                "Second one is a bit".to_string(),
                "longer than first.".to_string(),
                "Third.".to_string(),
            ]
        );
    }

    #[test]
    fn test_long_sentence_wrapping() {
        let p = "ThisSentenceIsWayTooLongToFitInTheWidthLimitSoItMustBeWrappedSomehow.";
        let lines = split(p, 10, true);
        // Since sentence longer than width, it will wrap on words (here single long word)
        assert_eq!(
            lines,
            vec![
                "ThisSentenceIsWayTooLongToFitInTheWidthLimitSoItMustBeWrappedSomehow.".to_string(),
            ]
        );
    }
}
