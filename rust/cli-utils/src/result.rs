use serde::Serialize;

/// A result which should be printed to the console
pub type Result = eyre::Result<Value>;

/// A result with nothing to be displayed
pub fn nothing() -> Result {
    Ok(Value {
        ..Default::default()
    })
}

/// A result with a value to be displayed
pub fn value<Type>(value: Type) -> Result
where
    Type: Serialize,
{
    Ok(Value {
        value: Some(serde_json::to_value(&value)?),
        ..Default::default()
    })
}

/// A result with content to be displayed
pub fn content(format: &str, content: &str) -> Result {
    Ok(Value {
        format: Some(format.into()),
        content: Some(content.into()),
        ..Default::default()
    })
}

/// A result with content or value to be displayed
pub fn new<Type>(format: &str, content: &str, value: Type) -> Result
where
    Type: Serialize,
{
    Ok(Value {
        format: Some(format.into()),
        content: Some(content.into()),
        value: Some(serde_json::to_value(&value)?),
    })
}

/// A value resulting from a command
#[derive(Debug, Default)]
pub struct Value {
    /// The value to be displayed
    pub value: Option<serde_json::Value>,

    /// The content to be displayed
    pub content: Option<String>,

    /// Format of the content
    pub format: Option<String>,
}

/// Printing without prettiness
#[cfg(not(feature = "pretty"))]
pub mod print {
    use super::*;

    /// Print a value
    pub fn value(value: Value, _formats: &[String]) -> eyre::Result<()> {
        match value {
            Value {
                content: Some(content),
                ..
            } => println!("{}", content),
            Value {
                value: Some(value), ..
            } => println!("{}", serde_json::to_string_pretty(&value)?),
            _ => (),
        };
        Ok(())
    }

    /// Print an error
    pub fn error(error: eyre::Report) {
        eprintln!("ERROR {:?}", error);
    }
}

/// Printing with prettiness
#[cfg(feature = "pretty")]
pub mod print {
    use super::*;

    /// Print a value
    pub fn value(value: Value, formats: &[String]) -> eyre::Result<()> {
        let Value {
            content,
            format,
            value,
        } = &value;

        // Nothing to display
        if content.is_none() && value.is_none() {
            return Ok(());
        }

        // Try to display in preferred format
        for preference in formats {
            if let (Some(content), Some(format)) = (content, format) {
                if format == preference {
                    return match format.as_str() {
                        "md" => markdown(content),
                        _ => highlight(format, content),
                    };
                }
            }
            if let Some(value) = value {
                if let Some(content) = match preference.as_str() {
                    "json" => serde_json::to_string_pretty(&value).ok(),
                    "yaml" => serde_yaml::to_string(&value)
                        .map(|yaml| yaml.trim_start_matches("---\n").to_string())
                        .ok(),
                    _ => None,
                } {
                    return highlight(preference, &content);
                }
            }
        }

        // Fallback to displaying content if available, otherwise value as JSON.
        if let (Some(content), Some(format)) = (content, format) {
            match format.as_str() {
                "md" => return markdown(content),
                _ => return highlight(format, content),
            };
        } else if let Some(value) = value {
            let json = serde_json::to_string_pretty(&value)?;
            return highlight("json", &json);
        }

        Ok(())
    }

    // Print Markdown to the terminal
    pub fn markdown(content: &str) -> eyre::Result<()> {
        if atty::isnt(atty::Stream::Stdout) {
            println!("{}", content)
        } else {
            let skin = termimad::MadSkin::default();
            println!("{}", skin.term_text(content));
        }

        Ok(())
    }

    // Apply syntax highlighting and print to terminal
    pub fn highlight(format: &str, content: &str) -> eyre::Result<()> {
        use once_cell::sync::Lazy;
        use syntect::{
            easy::HighlightLines,
            highlighting::{Style, ThemeSet},
            parsing::SyntaxSet,
            util::as_24_bit_terminal_escaped,
        };

        if atty::isnt(atty::Stream::Stdout) {
            println!("{}", content)
        } else {
            // TODO: Only bake in a subset of syntaxes and themes? See the following for examples of this
            // https://github.com/ducaale/xh/blob/master/build.rs
            // https://github.com/sharkdp/bat/blob/0b44aa6f68ab967dd5d74b7e02d306f2b8388928/src/assets.rs
            static SYNTAXES: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
            static THEMES: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

            let syntax = SYNTAXES
                .find_syntax_by_extension(format)
                .unwrap_or_else(|| SYNTAXES.find_syntax_by_extension("txt").unwrap());

            let mut highlighter = HighlightLines::new(syntax, &THEMES.themes["Solarized (light)"]);
            for line in content.lines() {
                let ranges: Vec<(Style, &str)> = highlighter.highlight(line, &SYNTAXES);
                let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
                println!("{}", escaped);
            }
        }

        Ok(())
    }

    /// Print an error
    pub fn error(error: eyre::Report) {
        use ansi_term::Color::{Blue, Red};
        use color_eyre::{Help, SectionExt};

        let title = format!("CLI: {}", error);
        let body = format!(
            "Version: {}\nOS: {}\n\nPlease describe the error a little more...",
            env!("CARGO_PKG_VERSION").to_string(),
            std::env::consts::OS
        );
        let issue_url = format!(
            "https://github.com/stencila/stencila/issues/new?title={}&body={}",
            urlencoding::encode(&title),
            urlencoding::encode(&body)
        );

        let error = error.with_section(move || {
            format!(
                "Report issue: {}.\nRead docs: {}.",
                Blue.paint(issue_url),
                Blue.paint("https://help.stenci.la")
            )
            .header("Help:")
        });
        eprintln!("{} {:?}", Red.bold().paint("ERROR"), error);
    }
}
