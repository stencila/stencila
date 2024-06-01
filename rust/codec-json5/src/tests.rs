use codec::{
    common::tokio,
    schema::shortcuts::{art, p, t},
};

use super::*;

/// Test that workaround for bug in `json5-rs` works
///
/// See https://github.com/callum-oakley/json5-rs/issues/21
#[tokio::test]
async fn escaping_unicode_2028_and_2029() -> Result<()> {
    let codec = Json5Codec {};

    let doc1 = art([p([t("\u{2028}")])]);

    let (json5, ..) = codec
        .to_string(
            &doc1,
            Some(EncodeOptions {
                compact: Some(true),
                ..Default::default()
            }),
        )
        .await?;
    assert_eq!(
        json5,
        r#"{type: "Article",content: [{type: "Paragraph",content: [{type: "Text",value: { string: "\u2028" }}]}]}"#
    );

    let (doc2, ..) = codec.from_str(&json5, None).await?;
    assert_eq!(doc2, doc1);

    Ok(())
}
