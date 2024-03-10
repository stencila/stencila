use syntect::{
    easy::HighlightLines, highlighting::ThemeSet, parsing::SyntaxSet,
    util::as_24_bit_terminal_escaped,
};

use common::{once_cell::sync::Lazy, serde::Serialize};
use format::Format;

use crate::ToStdout;

/// A message for a user
#[derive(Serialize)]
#[serde(crate = "common::serde")]
pub struct Code {
    /// The format of the code
    format: Format,

    /// The content of the code
    content: String,
}

impl Code {
    pub fn new(format: Format, content: &str) -> Self {
        Self {
            format,
            content: content.into(),
        }
    }
}

impl ToStdout for Code {
    fn to_terminal(&self) -> impl std::fmt::Display {
        // Consider whether to only bake in a subset of syntaxes and themes? See the following for examples of this
        // https://github.com/ducaale/xh/blob/master/build.rs
        // https://github.com/sharkdp/bat/blob/0b44aa6f68ab967dd5d74b7e02d306f2b8388928/src/assets.rs
        static SYNTAXES: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
        static THEMES: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

        let ext = match self.format {
            Format::Dom => "html".to_string(),
            Format::Json5 => "js".to_string(),
            Format::JsonLd => "json".to_string(),
            Format::Jats => "xml".to_string(),
            _ => self.format.extension(),
        };

        let syntax = SYNTAXES
            .find_syntax_by_extension(&ext)
            .unwrap_or_else(|| SYNTAXES.find_syntax_by_extension("txt").unwrap());

        let mut highlighted = String::new();
        let mut highlighter = HighlightLines::new(syntax, &THEMES.themes["Solarized (light)"]);
        for line in self.content.lines() {
            // Long lines can take a very long time to highlight so skip those
            let line = if line.len() > 500 {
                line.to_string()
            } else if let Ok(ranges) = highlighter.highlight_line(line, &SYNTAXES) {
                as_24_bit_terminal_escaped(&ranges[..], false)
            } else {
                line.to_string()
            };

            highlighted.push_str(&line);
            highlighted.push('\n');
        }

        // Ensure terminal attributes are reset to their defaults otherwise
        // the styling on the last line will persist (e.g. in a REPL)
        format!("{}\x1b[0m", highlighted)
    }
}
