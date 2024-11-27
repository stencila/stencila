//! Property-based tests of roundtrip conversion
//!
//! For each format, these tests generate arbitrary `Article`s, encode each article to the
//! format, decode it back from the format, and then asserts that the decoded article
//! is the same as the original.
//!
//! There are four levels of randomness/complexity: min, low, high, and max. Usually, codecs
//! are initially tested in the `min` level, and then moved up as high as the format will
//! allow.

#![allow(unused_imports)]

use codec::{
    common::{eyre::Result, futures::executor::block_on, once_cell::sync::Lazy, tokio::runtime},
    format::Format,
    schema::{Article, AudioObject, Node},
    DecodeOptions, EncodeOptions,
};
use common_dev::{
    pretty_assertions::assert_eq,
    proptest::prelude::{proptest, ProptestConfig},
};
use node_strip::{StripNode, StripTargets};

/// Do a roundtrip conversion to/from a format
#[allow(unused)]
fn roundtrip(
    format: Format,
    node: &Node,
    encode_options: Option<EncodeOptions>,
    decode_options: Option<DecodeOptions>,
) -> Result<Node> {
    block_on(async {
        let codec = codecs::get(None, Some(&format), None)?;

        let encode_options = Some(EncodeOptions {
            format: Some(format.clone()),
            ..encode_options.unwrap_or_default()
        });

        let decode_options = Some(DecodeOptions {
            format: Some(format),
            ..decode_options.unwrap_or_default()
        });

        let node = if codec.supports_to_bytes() && codec.supports_from_bytes() {
            let (bytes, ..) = codec.to_bytes(node, encode_options).await?;
            let (node, ..) = codec.from_bytes(&bytes, decode_options).await?;
            node
        } else {
            let (string, ..) = codec.to_string(node, encode_options).await?;
            let (node, ..) = codec.from_str(&string, decode_options).await?;
            node
        };

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

    /// Roundtrip test for Stencila Markdown
    #[test]
    fn article_smd(article: Article) {
        let mut article = Node::Article(article);

        article.strip(&StripTargets {
            types: vec![
                // TODO Resolve why block quotes are causing failures
                // in round trips and re-enable
                // See https://github.com/stencila/stencila/issues/1924
                "QuoteBlock".into(),
                // TODO: Work out why admonitions are not round-tripped
                // properly in some instances
                "Admonition".into()
            ],
            properties: vec![
                // `CodeChunk.label` are not supported if there is no
                // `label_type` (which can be generated as an arbitrary combo)
                "CodeChunk.label".into(),
                // TODO workout why figure labels are not round-tripped
                // properly in some instances
                "Figure.label".into()
            ],
            ..Default::default()
        });

        assert_eq!(roundtrip(Format::Smd, &article, None, None).unwrap(), article);
    }

    /// Roundtrip test for Pandoc
    #[test]
    fn article_pandoc(article: Article) {
        let mut article = Node::Article(article);

        article.strip(&StripTargets {
            types: vec![
                // TODO: these node types are not yet fully implemented
                // so strip them from round-trip conversions
                "Admonition".into(),
                "CallBlock".into(),
                "Claim".into(),
                "CodeChunk".into(),
                "ForBlock".into(),
                "IfBlock".into(),
                "IncludeBlock".into(),
                "InstructionBlock".into(),
                "Section".into(),
            ],
            properties: vec![
                // Language is not currently supported for inline code
                "CodeInline.programming_language".into(),
                // Table notes not currently supported
                "Table.notes".into()
            ],
            ..Default::default()
        });

        assert_eq!(roundtrip(Format::Pandoc, &article, None, None).unwrap(), article);
    }
}

// Level `high` for highly structured formats that can perform roundtrip conversion
// for most most node types and their values.
#[cfg(feature = "proptest-high")]
proptest! {
    #![proptest_config(ProptestConfig::with_cases(250))]

    /// Roundtrip test for JATS using `compact` option because whitespace
    /// can be added if not compact.
    #[test]
    fn article_jats(article: Article) {
        let mut article = Node::Article(article);

        article.strip(&StripTargets {
            types: vec![
                // TODO Remove these as implemented
                "CallBlock".into(),
                "Claim".into(),
                "CodeBlock".into(),
                "CodeChunk".into(),
                "Figure".into(),
                "ForBlock".into(),
                "IfBlock".into(),
                "IncludeBlock".into(),
                "List".into(),
                "MathBlock".into(),
                "RawBlock".into(),
                "StyledBlock".into(),
                "Table".into()
            ],
            ..Default::default()
        });

        assert_eq!(roundtrip(Format::Jats, &article, Some(EncodeOptions{
            standalone: Some(true),
            compact: Some(true),
            ..Default::default()
        }), None).unwrap(), article);
    }
}

// Level `max` for data serialization formats that can perform
// roundtrip conversion for all node types and their values.
//
// Due to the large size and complexity of the generated, arbitrary documents,
// to avoid long run times, a relatively low number of cases are tested.
#[cfg(feature = "proptest-max")]
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Roundtrip test for CBOR
    #[test]
    fn article_cbor(article: Article) {
        let article = Node::Article(article);
        assert_eq!(roundtrip(Format::Cbor, &article, None, None).unwrap(), article);
    }

    /// Roundtrip test for CBOR with Zstandard compression
    #[test]
    fn article_cbor_zst(article: Article) {
        let article = Node::Article(article);
        assert_eq!(roundtrip(Format::CborZst, &article, None, None).unwrap(), article);
    }

    /// Roundtrip test for JSON
    #[test]
    fn article_json(article: Article) {
        let article = Node::Article(article);
        assert_eq!(roundtrip(Format::Json, &article, None, None).unwrap(), article);
    }

    /// Roundtrip test for JSON5
    #[test]
    fn article_json5(article: Article) {
        let article = Node::Article(article);
        assert_eq!(roundtrip(Format::Json5, &article, None, None).unwrap(), article);
    }

    /// Roundtrip test for JSON5 with `compact` option.
    #[test]
    fn article_json5_compact(article: Article) {
        let article = Node::Article(article);
        assert_eq!(roundtrip(Format::Json5, &article, Some(EncodeOptions{
            compact: Some(true),
            ..Default::default()
        }), None).unwrap(), article);
    }

    /// Roundtrip test for JSON-LD
    #[test]
    fn article_jsonld(article: Article) {
        let article = Node::Article(article);
        assert_eq!(roundtrip(Format::JsonLd, &article, None, None).unwrap(), article);
    }

    /// Roundtrip test for JSON-LD with `compact` option.
    #[test]
    fn article_jsonld_compact(article: Article) {
        let article = Node::Article(article);
        assert_eq!(roundtrip(Format::JsonLd, &article, Some(EncodeOptions{
            compact: Some(true),
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
