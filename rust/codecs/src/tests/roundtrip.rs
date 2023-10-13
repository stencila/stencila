#![allow(unused_imports)]

use codec::{
    common::{eyre::Result, once_cell::sync::Lazy, tokio::runtime::Runtime},
    format::Format,
    schema::{Article, Node},
    DecodeOptions, EncodeOptions,
};
use common_dev::{
    pretty_assertions::assert_eq,
    proptest::prelude::{proptest, ProptestConfig},
};
use node_strip::{StripNode, Targets};

use crate::get;

/// Do a roundtrip conversion to/from a format
#[allow(unused)]
fn roundtrip(
    format: Format,
    node: &Node,
    encode_options: Option<EncodeOptions>,
    decode_options: Option<DecodeOptions>,
) -> Result<Node> {
    static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().unwrap());
    RUNTIME.handle().block_on(async {
        let codec = get(None, Some(format), None)?;
        let (string, ..) = codec.to_string(node, encode_options).await?;
        let (node, ..) = codec.from_str(&string, decode_options).await?;
        Ok(node)
    })
}

#[cfg(feature = "proptest-min")]
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]
}

#[cfg(feature = "proptest-low")]
proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    /// Roundtrip test for JATS using `compact` option because whitespace
    /// can be added if not compact.
    /// Currently skipped because decoding of many types is not implemented.
    /// Will implement after this is merged into main.
    #[ignore]
    #[test]
    fn article_jats_compact(article: Article) {
        let mut article = Node::Article(article);

        article.strip(&Targets {
            // Strip headings because JATS does not support heading level (in <title> elem).
            // TODO: When strip supports props, strip only Heading.level
            types: vec![String::from("Heading")],
            ..Default::default()
        });

        assert_eq!(roundtrip(Format::Jats, &article, Some(EncodeOptions{
            compact: true,
            ..Default::default()
        }), None).unwrap(), article);
    }
}

#[cfg(feature = "proptest-high")]
proptest! {
    #![proptest_config(ProptestConfig::with_cases(250))]
}

#[cfg(feature = "proptest-max")]
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Roundtrip test for JSON
    #[test]
    fn article_json(article: Article) {
        let article = Node::Article(article);
        assert_eq!(roundtrip(Format::Json, &article, None, None).unwrap(), article);
    }

    /// Roundtrip test for JSON5
    /// Fails due to an error parsing strings containing only whitespace (?) so
    /// currently skipped.
    #[ignore]
    #[test]
    fn article_json5(article: Article) {
        let article = Node::Article(article);
        assert_eq!(roundtrip(Format::Json5, &article, None, None).unwrap(), article);
    }

    /// Roundtrip test for JSON5 with `compact` option.
    #[ignore]
    #[test]
    fn article_json5_compact(article: Article) {
        let article = Node::Article(article);
        assert_eq!(roundtrip(Format::Json5, &article, Some(EncodeOptions{
            compact: true,
            ..Default::default()
        }), None).unwrap(), article);
    }

    /// Roundtrip test for YAML
    #[test]
    fn article_yaml(article: Article) {
        let article = Node::Article(article);
        assert_eq!(roundtrip(Format::Yaml, &article, None, None).unwrap(), article);
    }
}
