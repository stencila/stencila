use std::sync::LazyLock;

use stencila_parser::{DefaultParser, stencila_format::Format};

pub use stencila_parser::{ParseInfo, Parser, stencila_schema::CompilationDigest};

/// Parse some code in a language
pub fn parse(
    code: &str,
    language: &Option<String>,
    compilation_digest: &Option<CompilationDigest>,
) -> ParseInfo {
    static PARSERS: LazyLock<Vec<Box<dyn Parser>>> = LazyLock::new(Vec::new);

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
