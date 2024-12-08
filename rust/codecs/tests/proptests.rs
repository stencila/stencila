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
            ],
            properties: vec![
                // Admonition title is currently encoded as plain, unstructured text
                // only, so strip it.
                "Admonition.title".into(),
                // `CodeChunk.label` are not supported if there is no
                // `label_type` (which can be generated as an arbitrary combo)
                "CodeChunk.label".into(),
                // Arbitrary figures do not necessarily have `label_automatically == false`
                // when a label is present so need to strip label
                "Figure.label".into()
            ],
            ..Default::default()
        });

        assert_eq!(roundtrip(Format::Smd, &article, None, None).unwrap(), article);
    }

    /// Roundtrip test for IPYNB
    #[test]
    fn article_ipynb(article: Article) {
        let mut article = Node::Article(article);

        article.strip(&StripTargets {
            types: vec![
                // TODO: Determine why these can not be round tripped
                "Admonition".into(),
                "QuoteBlock".into(),
                "CodeBlock".into(),

                // TODO: Encoding to MyST is invalid
                // See https://github.com/stencila/stencila/issues/2467
                "CallBlock".into()
            ],
            properties: vec![
                // Code chunk captions currently only support simple Markdown.
                "CodeChunk.caption".into(),

                // TODO: Determine which of these can be round tripped
                // and for those that can not document why
                "Table.caption".into(),
                "Table.notes".into(),

                // Arbitrary code chunks and figures do not necessarily have `label_automatically == false`
                // when a label is present so need to strip label
                "CodeChunk.label".into(),
                "CodeChunk.label_type".into(),
                "Figure.label".into(),

                // MyST only supports an image as a figure so all content
                // must be stripped since it may include other node types
                "Figure.content".into(),

                // MyST currently does not support these
                "ForBlock.programming_language".into(),
                "IfBlockClause.programming_language".into(),
            ],
            ..Default::default()
        });

        assert_eq!(roundtrip(Format::Ipynb, &article, None, None).unwrap(), article);
    }

    /// Roundtrip test for Pandoc
    #[test]
    fn article_pandoc(article: Article) {
        let mut article = Node::Article(article);

        article.strip(&StripTargets {
            properties: vec![
                // These properties are currently encoded as plain, unstructured text
                // only, so we need to strip it for round trips to be same.
                "Admonition.title".into(),
                "CodeChunk.caption".into(),
                // The `programming_language` property of `CodeExpression`s is not
                // currently supported
                "CodeInline.programming_language".into(),
                // The `otherwise` property of `ForBlock`s is not yet supported
                "ForBlock.otherwise".into(),
                // Arbitrarily generated code chunks and figures do not necessarily have
                // `label_automatically == false` when `label` is `Some` so we need
                // to strip labels for round trips to be same
                "CodeChunk.label".into(),
                "Figure.label".into(),
                // Table notes not currently supported
                "Table.notes".into(),
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
                "CodeChunk".into(),
                "Figure".into(),
                "ForBlock".into(),
                "IfBlock".into(),
                "IncludeBlock".into(),
                "RawBlock".into(),
                "StyledBlock".into(),
                "Table".into()
            ],
            properties: vec![
                // Properties that are not yet supported
                "ListItem.is_checked".into()
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
