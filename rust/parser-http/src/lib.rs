use std::path::Path;

use parser::{
    apply_all_comment_tags, common::eyre::Result, formats::Format, parse_file_interps,
    parse_var_interps, ParseInfo, Parser, ParserTrait,
};

/// A parser for HTTP requests
pub struct HttpParser;

impl ParserTrait for HttpParser {
    fn spec() -> Parser {
        Parser {
            language: Format::Http,
        }
    }

    fn parse(code: &str, path: Option<&Path>) -> Result<ParseInfo> {
        let mut parse_info = ParseInfo {
            language: Self::spec().language,
            execution_dependencies: [
                parse_var_interps(code, path),
                parse_file_interps(code, path),
            ]
            .concat(),
            ..Default::default()
        };
        apply_all_comment_tags(&mut parse_info, code, path, "#");

        Ok(parse_info)
    }
}
