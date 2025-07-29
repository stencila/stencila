use std::io::{Write, stdout};

/// Clear the terminal
#[allow(clippy::print_stdout)]
pub fn clear_terminal() {
    print!("\x1B[2J\x1B[1;1H");
    stdout().flush().ok();
}

/// Strip ANSI escape sequences from a string
pub fn strip_ansi_escapes(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            // Skip the escape sequence
            if chars.next() == Some('[') {
                // Skip until we find a letter
                for c in chars.by_ref() {
                    if c.is_alphabetic() || c == 'm' {
                        break;
                    }
                }
            }
        } else {
            result.push(ch);
        }
    }
    result
}
