use parser::{common::once_cell::sync::Lazy, format::Format, DefaultParser};

pub use parser::{schema::CompilationDigest, ParseInfo, Parser};

/// Parse some code in a language
pub fn parse(
    code: &str,
    language: &Option<String>,
    compilation_digest: &Option<CompilationDigest>,
) -> ParseInfo {
    static PARSERS: Lazy<Vec<Box<dyn Parser>>> = Lazy::new(Vec::new);

    let format = language
        .as_ref()
        .map(|language| Format::from_name(language))
        .unwrap_or(Format::Unknown);

    for parser in PARSERS.iter() {
        if parser.supports_language(&format) {
            return parser.parse(code, &format, compilation_digest);
        }
    }

    DefaultParser.parse(code, &format, compilation_digest)
}
