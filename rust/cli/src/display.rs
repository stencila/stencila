use common::eyre::Result;
use format::Format;

/// Apply syntax highlighting to the content and print to terminal
pub fn highlighted(content: &str, format: Format) -> Result<()> {
    use common::once_cell::sync::Lazy;
    use syntect::{
        easy::HighlightLines,
        highlighting::{Style, ThemeSet},
        parsing::SyntaxSet,
        util::as_24_bit_terminal_escaped,
    };

    if is_terminal::is_terminal(&std::io::stdout()) {
        // Consider whether to only bake in a subset of syntaxes and themes? See the following for examples of this
        // https://github.com/ducaale/xh/blob/master/build.rs
        // https://github.com/sharkdp/bat/blob/0b44aa6f68ab967dd5d74b7e02d306f2b8388928/src/assets.rs
        static SYNTAXES: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
        static THEMES: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);

        let ext = match format {
            Format::Json5 => "js".to_string(),
            Format::JsonLd => "json".to_string(),
            Format::Jats => "xml".to_string(),
            _ => format.get_extension(),
        };

        let syntax = SYNTAXES
            .find_syntax_by_extension(&ext)
            .unwrap_or_else(|| SYNTAXES.find_syntax_by_extension("txt").unwrap());

        let mut highlighter = HighlightLines::new(syntax, &THEMES.themes["Solarized (light)"]);
        for line in content.lines() {
            let ranges: Vec<(Style, &str)> = highlighter.highlight_line(line, &SYNTAXES)?;
            let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
            // Ensure terminal attributes are reset to their defaults otherwise
            // the styling on the last line will persist (e.g. in a REPL)
            println!("{}\x1b[0m", escaped);
        }
    } else {
        println!("{}", content)
    }

    Ok(())
}
