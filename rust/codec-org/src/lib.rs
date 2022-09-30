use codec::{
    common::{async_trait::async_trait, eyre::Result},
    stencila_schema::Node,
    utils::vec_string,
    Codec, CodecTrait, DecodeOptions, EncodeOptions,
};
use codec_pandoc::{decode, encode, PandocCodec};

/// A codec for [Org Mode](https://orgmode.org)
///
/// This is currently very preliminary and does not include support/integration with
/// [org-babel](https://orgmode.org/worg/org-contrib/babel/).
///
/// Currently delegates to [`PandocCodec`]. Rust implementations of Org Mode parsers,
/// which could be used instead if necessary, include:
///     
/// - [`org-rs`](https://github.com/org-rs/org-rs)
/// - [`orgize`](https://github.com/PoiScript/orgize)
pub struct OrgCodec;

#[async_trait]
impl CodecTrait for OrgCodec {
    fn spec() -> Codec {
        let pandoc_codec = PandocCodec::spec();
        Codec {
            status: "alpha".to_string(),
            formats: vec_string!["org"],
            root_types: vec_string!["Article"],
            unsupported_types: [
                pandoc_codec.unsupported_types,
                // This list of unsupported node types based on prop test in `../tests/prop.rs`
                vec_string![
                    "AudioObject",
                    "CodeChunk",
                    "CodeExpression",
                    "NontextualAnnotation",
                    "Parameter",
                    "Quote",
                    "VideoObject"
                ],
            ]
            .concat(),
            ..Default::default()
        }
    }

    async fn from_str_async(str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        decode(str, None, "org", &[]).await
    }

    async fn to_string_async(node: &Node, options: Option<EncodeOptions>) -> Result<String> {
        encode(node, None, "org", &[], options).await
    }
}

#[cfg(test)]
mod tests {
    use test_snaps::{insta::assert_json_snapshot, snapshot_fixtures_content};
    use test_utils::common::tokio;

    use super::*;

    #[test]
    fn articles() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        snapshot_fixtures_content("articles/*.org", |org| {
            let node =
                runtime.block_on(async { OrgCodec::from_str_async(org, None).await.unwrap() });
            assert_json_snapshot!(node);
        });
    }

    #[test]
    fn fragments() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        snapshot_fixtures_content("fragments/org/*.org", |org| {
            let node =
                runtime.block_on(async { OrgCodec::from_str_async(org, None).await.unwrap() });
            assert_json_snapshot!(node);
        });
    }
}
