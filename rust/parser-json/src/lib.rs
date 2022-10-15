use std::path::Path;

use parser::{
    common::{eyre::Result, serde_json},
    formats::Format,
    graph_triples::{Resource, ResourceInfo},
    Parser, ParserTrait,
};

/// A parser for JSON
///
/// Simply checks that that the JSON has no syntax errors (for the
/// purpose of language detection).
pub struct JsonParser {}

impl ParserTrait for JsonParser {
    fn spec() -> Parser {
        Parser {
            language: Format::Json,
        }
    }

    fn parse(resource: Resource, _path: &Path, code: &str) -> Result<ResourceInfo> {
        let mut resource_info = ResourceInfo::default(resource);
        if serde_json::from_str::<serde_json::Value>(code).is_err() {
            resource_info.syntax_errors = Some(true);
        }
        Ok(resource_info)
    }
}
