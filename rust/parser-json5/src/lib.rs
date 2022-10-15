use std::path::Path;

use parser::{
    common::{eyre::Result, json5, serde_json},
    formats::Format,
    graph_triples::{Resource, ResourceInfo},
    Parser, ParserTrait,
};

/// A parser for JSON5
///
/// Simply checks that that the supplied code has no syntax errors (for the
/// purpose of language detection).
pub struct Json5Parser {}

impl ParserTrait for Json5Parser {
    fn spec() -> Parser {
        Parser {
            language: Format::Json5,
        }
    }

    fn parse(resource: Resource, _path: &Path, code: &str) -> Result<ResourceInfo> {
        let mut resource_info = ResourceInfo::default(resource);
        if json5::from_str::<serde_json::Value>(code).is_err() {
            resource_info.syntax_errors = Some(true);
        }
        Ok(resource_info)
    }
}
