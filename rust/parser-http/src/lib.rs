use std::path::Path;

use parser::{
    common::eyre::Result,
    formats::Format,
    graph_triples::{execution_digest_from_content, Resource, ResourceInfo},
    utils::{apply_tags_all, parse_var_interps},
    Parser, ParserTrait,
};

/// A parser for HTTP requests
pub struct HttpParser;

impl ParserTrait for HttpParser {
    fn spec() -> Parser {
        Parser {
            language: Format::Http,
        }
    }

    fn parse(resource: Resource, path: &Path, code: &str) -> Result<ResourceInfo> {
        let mut resource_info = ResourceInfo::default(resource);
        resource_info.relations = Some(parse_var_interps(code, path));
        resource_info.compile_digest = Some(execution_digest_from_content(code));
        apply_tags_all(path, Format::Http, code, "#", None, &mut resource_info);

        Ok(resource_info)
    }
}
