use std::{fmt::Display, sync::LazyLock};

use clap::ValueEnum;
use eyre::{Result, bail};
use serde::Serialize;
use syntect::{
    easy::HighlightLines, highlighting::ThemeSet, parsing::SyntaxSet,
    util::as_24_bit_terminal_escaped,
};

use stencila_format::Format;

use crate::ToStdout;

/// A message for a user
pub struct Code {
    /// The format of the code
    format: Format,

    /// The content of the code
    content: String,
}

/// A format that can be used to output a serializable value
#[derive(Debug, Clone, ValueEnum)]
pub enum AsFormat {
    Json,
    Yaml,
    Toml,
}

impl From<AsFormat> for Format {
    fn from(value: AsFormat) -> Self {
        match value {
            AsFormat::Json => Format::Json,
            AsFormat::Yaml => Format::Yaml,
            AsFormat::Toml => Format::Toml,
        }
    }
}

impl Code {
    pub fn new(format: Format, content: &str) -> Self {
        Self {
            format,
            content: content.into(),
        }
    }

    pub fn new_from<S>(format: Format, value: &S) -> Result<Self>
    where
        S: Serialize,
    {
        let content = match format {
            Format::Json => serde_json::to_string_pretty(value)?,
            Format::Yaml => serde_yaml::to_string(value)?,
            Format::Toml => toml::to_string(value)?,
            _ => bail!("Unsupported serialization format: {format}"),
        };

        Ok(Self { format, content })
    }
}

impl Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.content)
    }
}

impl ToStdout for Code {
    fn to_terminal(&self) -> impl std::fmt::Display {
        // Consider whether to only bake in a subset of syntaxes and themes? See the following for examples of this
        // https://github.com/ducaale/xh/blob/master/build.rs
        // https://github.com/sharkdp/bat/blob/0b44aa6f68ab967dd5d74b7e02d306f2b8388928/src/assets.rs
        static SYNTAXES: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
        static THEMES: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

        let ext = match self.format {
            Format::Dom => "html".to_string(),
            Format::Json5 => "js".to_string(),
            Format::Shell => "bash".to_string(),
            _ => {
                if self.format.is_json_flavor() {
                    "json".to_string()
                } else if self.format.is_markdown_flavor() {
                    "md".to_string()
                } else if self.format.is_xml_flavor() {
                    "xml".to_string()
                } else {
                    self.format.extension()
                }
            }
        };

        let syntax = SYNTAXES
            .find_syntax_by_extension(&ext)
            .or(SYNTAXES.find_syntax_by_name(self.format.name()))
            .unwrap_or_else(|| {
                SYNTAXES
                    .find_syntax_by_extension("txt")
                    .expect("should always be a txt theme")
            });

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
        format!("{highlighted}\x1b[0m")
    }
}
