use codec::{
    Codec,
    common::{eyre::Result, tokio},
    schema::{CodeChunk, Node},
};
use codec_markdown::MarkdownCodec;
use common_dev::insta::assert_snapshot;

/// Test that a sufficient number of enclosing backticks are encoded to escape
/// backticks in the code
///
/// See https://github.com/stencila/stencila/issues/2390
#[tokio::test]
async fn backticks() -> Result<()> {
    let codec = MarkdownCodec {};

    // No backticks in code
    let (md, ..) = codec
        .to_string(
            &Node::CodeChunk(CodeChunk::new("no backticks".into())),
            None,
        )
        .await?;
    assert_snapshot!(md, @r#"
    ```exec
    no backticks
    ```
    "#);

    // Single nested backticks in code
    let (md, ..) = codec
        .to_string(
            &Node::CodeChunk(CodeChunk::new("```\ncode\n```".into())),
            None,
        )
        .await?;
    assert_snapshot!(md, @r#"
    ````exec
    ```
    code
    ```
    ````
    "#);

    // Double nested backticks in code
    let (md, ..) = codec
        .to_string(
            &Node::CodeChunk(CodeChunk::new("````\n```\ncode\n```\n````".into())),
            None,
        )
        .await?;
    assert_snapshot!(md, @r#"
    `````exec
    ````
    ```
    code
    ```
    ````
    `````
    "#);

    // Just one backtick in code
    let (md, ..) = codec
        .to_string(&Node::CodeChunk(CodeChunk::new("`".into())), None)
        .await?;
    assert_snapshot!(md, @r#"
    ```exec
    `
    ```
    "#);

    // Three backticks with preceding whitespace in code
    let (md, ..) = codec
        .to_string(&Node::CodeChunk(CodeChunk::new("  ```".into())), None)
        .await?;
    assert_snapshot!(md, @r#"
    ````exec
      ```
    ````
    "#);

    Ok(())
}
