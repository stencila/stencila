use std::path::Path;

use parser::{
    common::{eyre::Result, json5, serde_json},
    formats::Format,
    hash_utils::str_seahash,
    ParseInfo, Parser, ParserTrait,
};

/// A parser for JSON5
///
/// Simply checks that that the supplied code has no syntax errors (for the
/// purpose of language detection)  and calculates a semantic digest (using
/// non-pretty serialization).
pub struct Json5Parser {}

impl ParserTrait for Json5Parser {
    fn spec() -> Parser {
        Parser {
            language: Format::Json5,
        }
    }

    fn parse(code: &str, _path: Option<&Path>) -> Result<ParseInfo> {
        let mut parse_info = ParseInfo {
            language: Self::spec().language,
            ..Default::default()
        };
        match json5::from_str::<serde_json::Value>(code) {
            Ok(value) => {
                let json = serde_json::to_string(&value)?;
                parse_info.semantic_digest = str_seahash(&json)?;
            }
            Err(..) => {
                parse_info.syntax_errors = true;
            }
        }
        Ok(parse_info)
    }
}
