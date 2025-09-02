use textwrap::{Options, termwidth, wrap};

/// Create a wrapped and formatted user message on the terminal
#[allow(clippy::print_stderr)]
pub fn message(message: &str, icon: Option<&str>) {
    let formatted_message = if std::env::var_os("NO_COLOR").is_none() {
        // Color content between backticks cyan when NO_COLOR is not set
        let mut result = String::new();
        let mut chars = message.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '`' {
                // Found opening backtick, look for closing backtick
                let mut code_content = String::new();
                let mut found_closing = false;

                for inner_ch in chars.by_ref() {
                    if inner_ch == '`' {
                        found_closing = true;
                        break;
                    }
                    code_content.push(inner_ch);
                }

                if found_closing && !code_content.is_empty() {
                    // Add orange coloring around the content
                    result.push_str(&format!("\x1b[38;5;208m{}\x1b[0m", code_content));
                } else {
                    // No closing backtick found or empty content, keep original
                    result.push('`');
                    result.push_str(&code_content);
                }
            } else {
                result.push(ch);
            }
        }
        result
    } else {
        message.to_string()
    };

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

    eprintln!("{}", wrap(&formatted_message, options,).join("\n"));
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
