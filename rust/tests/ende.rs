///! Encoding-decoding tests
///!
///! These integration tests check that for each format the
///! `encode` and `decode` functions are consistent by doing a
///! round trip conversion of arbitrary instances of nodes.
///!
///! Uses `serde_json::to_value` for assertions because currently unable
///! to compare nodes directly. `serde_json::Value` provides most readable
///! way to compare.
use pretty_assertions::assert_eq;
use proptest::prelude::*;
use stencila::methods::{decode, encode};
use stencila_schema::{BlockContent, Node};

mod strategies;
use strategies::{article, code_chunk, node, Freedom};

proptest! {
    // Tests for generic data serialization formats

    // Given the high confidence for encoding/decoding for the
    // following formats the number of test cases is minimal
    #![proptest_config(ProptestConfig::with_cases(5))]

    #[cfg(all(feature="encode-json", feature="decode-json"))]
    #[test]
    fn json(input in node(Freedom::Max)) {
        let content = encode::json::encode(&input, None).unwrap();
        let output = decode::json::decode(&content).unwrap();
        assert_eq!(
            serde_json::to_value(&input).unwrap(),
            serde_json::to_value(&output).unwrap()
        )
    }

    #[cfg(all(feature="encode-yaml", feature="decode-yaml"))]
    #[test]
    fn yaml(input in node(Freedom::Max)) {
        let content = encode::yaml::encode(&input).unwrap();
        let output = decode::yaml::decode(&content).unwrap();
        assert_eq!(
            serde_json::to_value(&input).unwrap(),
            serde_json::to_value(&output).unwrap()
        )
    }
}

proptest! {
    // Tests for RPNGs
    //
    // RPNGs can be used for all node types but theses tests
    // focus on the types for which they are most predominately used.
    // Given the slowness of generating PNGs only use very few cases.
    #![proptest_config(ProptestConfig::with_cases(3))]

    #[cfg(all(feature="encode-rpng", feature="decode-rpng"))]
    #[test]
    fn rpng(chunk in code_chunk(Freedom::Max)) {
        let input = if let BlockContent::CodeChunk(chunk) = chunk {
            Node::CodeChunk(chunk)
        } else {
            panic!("Whaaat?!@#!!")
        };
        let content = tokio::runtime::Runtime::new().unwrap().block_on(async {
            encode::rpng::encode(&input, "data://").await.unwrap()
        });
        let output = decode::rpng::decode(&content).unwrap();
        assert_eq!(
            serde_json::to_value(&input).unwrap(),
            serde_json::to_value(&output).unwrap()
        )
    }
}

proptest! {
    // Tests for article formats

    // Given the relatively high randomness and complexity of each input
    // we reduce the number of test cases from the default of 256
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[cfg(all(feature="encode-html", feature="decode-html"))]
    #[test]
    fn html(input in article(Freedom::Max)) {
        let content = encode::html::encode(&input, None).unwrap();
        let output = decode::html::decode(&content, false).unwrap();
        assert_eq!(
            serde_json::to_value(&input).unwrap(),
            serde_json::to_value(&output).unwrap()
        )
    }

    #[cfg(all(feature="encode-md", feature="decode-md"))]
    #[test]
    fn md(input in article(Freedom::Min)) {
        let content = encode::md::encode(&input).unwrap();
        let output = decode::md::decode(&content).unwrap();
        assert_eq!(
            serde_json::to_value(&input).unwrap(),
            serde_json::to_value(&output).unwrap()
        )
    }

    #[cfg(all(feature="encode-rmd", feature="decode-rmd"))]
    #[test]
    fn rmd(input in article(Freedom::Min)) {
        let content = encode::rmd::encode(&input).unwrap();
        let output = decode::rmd::decode(&content).unwrap();
        assert_eq!(
            serde_json::to_value(&input).unwrap(),
            serde_json::to_value(&output).unwrap()
        )
    }

    #[cfg(all(feature="encode-pandoc", feature="decode-pandoc"))]
    #[test]
    fn pandoc(input in article(Freedom::Min)) {
        let pandoc = encode::pandoc::encode_node(&input).unwrap();
        let output = decode::pandoc::decode_pandoc(pandoc).unwrap();
        assert_eq!(
            serde_json::to_value(&input).unwrap(),
            serde_json::to_value(&output).unwrap()
        )
    }
}


proptest! {
    // Tests for formats that spawn Pandoc

    // Given the relative slowness of spawning a new process for
    // pandoc, only a few tests cases.
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[ignore = "End-to-end not yet completely working"] 
    #[cfg(all(feature="encode-latex", feature="decode-latex"))]
    #[test]
    fn latex(input in article(Freedom::Min)) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let content = encode::latex::encode(&input).await.unwrap();
            let output = decode::latex::decode(&content).await.unwrap();
            assert_eq!(
                serde_json::to_value(&input).unwrap(),
                serde_json::to_value(&output).unwrap()
            )
        })
    }

    #[ignore = "End-to-end not yet completely working"] 
    #[cfg(all(feature="encode-docx", feature="decode-docx"))]
    #[test]
    fn docx(input in article(Freedom::Min)) {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let path = "file://temp.docx".to_string();
            encode::docx::encode(&input, &path).await.unwrap();
            let output = decode::docx::decode(&path).await.unwrap();
            assert_eq!(
                serde_json::to_value(&input).unwrap(),
                serde_json::to_value(&output).unwrap()
            )
        })
    }
}
