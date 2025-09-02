use insta::assert_snapshot;
use stencila_codec::{
    Codec,
    eyre::Result,
    stencila_schema::{CodeBlock, Node},
};
use stencila_codec_markdown::MarkdownCodec;

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
            &Node::CodeBlock(CodeBlock::new("no backticks".into())),
            None,
        )
        .await?;
    assert_snapshot!(md, @r#"
    ```
    no backticks
    ```
    "#);

    // Single nested backticks in code
    let (md, ..) = codec
        .to_string(
            &Node::CodeBlock(CodeBlock::new("```\ncode\n```".into())),
            None,
        )
        .await?;
    assert_snapshot!(md, @r#"
    ````
    ```
    code
    ```
    ````
    "#);

    // Double nested backticks in code
    let (md, ..) = codec
        .to_string(
            &Node::CodeBlock(CodeBlock::new("````\n```\ncode\n```\n````".into())),
            None,
        )
        .await?;
    assert_snapshot!(md, @r#"
    `````
    ````
    ```
    code
    ```
    ````
    `````
    "#);

    // Just one backtick in code
    let (md, ..) = codec
        .to_string(&Node::CodeBlock(CodeBlock::new("`".into())), None)
        .await?;
    assert_snapshot!(md, @r#"
    ```
    `
    ```
    "#);

    // Three backticks with preceding whitespace in code
    let (md, ..) = codec
        .to_string(&Node::CodeBlock(CodeBlock::new("  ```".into())), None)
        .await?;
    assert_snapshot!(md, @r#"
    ````
      ```
    ````
    "#);

    Ok(())
}
