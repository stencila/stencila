use insta::assert_snapshot;
use stencila_codec::{
    Codec,
    eyre::Result,
    stencila_schema::{CodeBlock, CodeChunk, Node},
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
    assert_snapshot!(md, @r"
    ```
    no backticks
    ```
    ");

    // Single nested backticks in code
    let (md, ..) = codec
        .to_string(
            &Node::CodeBlock(CodeBlock::new("```\ncode\n```".into())),
            None,
        )
        .await?;
    assert_snapshot!(md, @r"
    ````
    ```
    code
    ```
    ````
    ");

    // Double nested backticks in code
    let (md, ..) = codec
        .to_string(
            &Node::CodeBlock(CodeBlock::new("````\n```\ncode\n```\n````".into())),
            None,
        )
        .await?;
    assert_snapshot!(md, @r"
    `````
    ````
    ```
    code
    ```
    ````
    `````
    ");

    // Just one backtick in code
    let (md, ..) = codec
        .to_string(&Node::CodeBlock(CodeBlock::new("`".into())), None)
        .await?;
    assert_snapshot!(md, @r"
    ```
    `
    ```
    ");

    // Three backticks with preceding whitespace in code
    let (md, ..) = codec
        .to_string(&Node::CodeBlock(CodeBlock::new("  ```".into())), None)
        .await?;
    assert_snapshot!(md, @r"
    ````
      ```
    ````
    ");

    Ok(())
}

#[tokio::test]
async fn preserve_code_block_id_on_encoding() -> Result<()> {
    let codec = MarkdownCodec {};

    let mut block = CodeBlock::new("echo hello".into());
    block.programming_language = Some("sh".into());
    block.id = Some("script-ref".into());

    let (md, ..) = codec.to_string(&Node::CodeBlock(block), None).await?;
    assert_snapshot!(md, @r"
    ```sh #script-ref
    echo hello
    ```
    ");

    Ok(())
}

#[tokio::test]
async fn preserve_code_chunk_id_on_encoding() -> Result<()> {
    let codec = MarkdownCodec {};

    let mut chunk = CodeChunk::default();
    chunk.code = "print('hello')".into();
    chunk.id = Some("chunk-ref".into());

    let (md, ..) = codec.to_string(&Node::CodeChunk(chunk), None).await?;
    assert_snapshot!(md, @r"
    ```exec #chunk-ref
    print('hello')
    ```
    ");

    Ok(())
}
