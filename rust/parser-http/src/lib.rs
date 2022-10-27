use std::path::Path;

use parser::{
    common::eyre::Result,
    formats::Format,
    graph_triples::{resources::ResourceDigest, Resource, ResourceInfo},
    utils::{parse_var_interps, apply_tags_all},
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
        resource_info.compile_digest = Some(ResourceDigest::from_strings(code, None));
        apply_tags_all(path, Format::Http, code, "#", None, &mut resource_info);

        Ok(resource_info)
    }
}
