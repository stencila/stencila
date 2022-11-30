use codec::{
    common::eyre::Result, stencila_schema::Node, utils::vec_string, Codec, CodecTrait,
    DecodeOptions, EncodeOptions,
};

mod decode;
mod encode;

// A codec for programming language scripts
pub struct ScriptCodec;

impl CodecTrait for ScriptCodec {
    fn spec() -> Codec {
        Codec {
            status: "beta".to_string(),
            formats: vec_string!["bash", "js", "py", "r", "sh", "sql", "zsh"],
            root_types: vec_string!["Article"],
            unsupported_types: vec_string!["If", "For", "Div", "Span"],
            ..Default::default()
        }
    }

    fn from_str(str: &str, options: Option<DecodeOptions>) -> Result<Node> {
        decode::decode(str, options)
    }

    fn to_string(node: &Node, options: Option<EncodeOptions>) -> Result<String> {
        encode::encode(node, options)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use test_snaps::{
        insta::{assert_json_snapshot, assert_snapshot},
        snapshot_fixtures_path_content,
    };

    use super::*;

    #[test]
    fn decode_and_encode_articles() {
        snapshot_fixtures_path_content("articles/scripts/*", |path: &Path, content| {
            let format = path
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let article = ScriptCodec::from_str(
                content,
                Some(DecodeOptions {
                    format: Some(format.clone()),
                }),
            )
            .unwrap();
            assert_json_snapshot!(article);

            let script = ScriptCodec::to_string(
                &article,
                Some(EncodeOptions {
                    format: Some(format),
                    ..Default::default()
                }),
            )
            .unwrap();
            assert_snapshot!(script);
        });
    }
}
