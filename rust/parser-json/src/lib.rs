use std::path::Path;

use parser::{
    common::{eyre::Result, serde_json},
    formats::Format,
    hash_utils::str_seahash,
    ParseInfo, Parser, ParserTrait,
};

/// A parser for JSON
///
/// Simply checks that that the JSON has no syntax errors (for the
/// purpose of language detection) and calculates a semantic digest (using
/// non-pretty serialization)
pub struct JsonParser {}

impl ParserTrait for JsonParser {
    fn spec() -> Parser {
        Parser {
            language: Format::Json,
        }
    }

    fn parse(code: &str, _path: Option<&Path>) -> Result<ParseInfo> {
        let mut parse_info = ParseInfo {
            language: Self::spec().language,
            ..Default::default()
        };
        match serde_json::from_str::<serde_json::Value>(code) {
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
