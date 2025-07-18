use textwrap::{termwidth, wrap, Options};

/// Create a wrapped user message on the terminal
#[allow(clippy::print_stderr)]
pub fn message(message: &str, icon: Option<&str>) {
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

    eprintln!("{}", wrap(message, options,).join("\n"));
}

#[macro_export]
macro_rules! message {
    ($str:literal, $($arg:tt)*) => {
        cli_utils::message(&format!($str, $($arg)*), None)
    };

    ($str:literal) => {
        cli_utils::message($str, None)
    };
}
