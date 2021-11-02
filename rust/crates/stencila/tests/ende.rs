///! Encoding-decoding tests
///!
///! These integration tests check that for each format the
///! `encode` and `decode` functions are consistent by doing a
///! round trip conversion of arbitrary instances of nodes.
///!
///! Uses `serde_json::to_value` for assertions because currently unable
///! to compare nodes directly. `serde_json::Value` provides most readable
///! way to compare.
use proptest::prelude::*;
use stencila::methods::{decode, encode};

mod strategies;
use strategies::{article, Freedom};

macro_rules! assert_json_eq {
    ($expr1:expr, $expr2:expr) => {
        pretty_assertions::assert_eq!(
            serde_json::to_value(&$expr1).unwrap(),
            serde_json::to_value(&$expr2).unwrap()
        );
    };
}

proptest! {
    // Tests for article formats

    // Given the relatively high randomness and complexity of each input
    // we reduce the number of test cases from the default of 256
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[cfg(all(feature="encode-pandoc", feature="decode-pandoc"))]
    #[test]
    fn pandoc(input in article(Freedom::Min)) {
        let pandoc = encode::pandoc::encode_node(&input).unwrap();
        let output = decode::pandoc::decode_pandoc(pandoc).unwrap();
        assert_json_eq!(input, output);
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
