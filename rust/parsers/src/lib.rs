use parser::{common::once_cell::sync::Lazy, format::Format, DefaultParser};

pub use parser::{ParseInfo, Parser};

/// Parse some code in a language
pub fn parse(code: &str, language: &str) -> ParseInfo {
    static PARSERS: Lazy<Vec<Box<dyn Parser>>> = Lazy::new(Vec::new);

    let format = Format::from_name(language);
    for parser in PARSERS.iter() {
        if parser.supports_language(&format) {
            return parser.parse(code, &format);
        }
    }

    DefaultParser::default().parse(code, &format)
}
