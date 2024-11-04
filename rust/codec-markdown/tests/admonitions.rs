use codec::{
    common::{eyre::Result, tokio},
    Codec,
};
use codec_markdown::MarkdownCodec;
use common_dev::insta::assert_yaml_snapshot;

/// Regression test for cases where the first inline of the first paragraph
/// is not a `Text` node.
#[tokio::test]
async fn admonitions() -> Result<()> {
    let codec = MarkdownCodec {};

    let (node, ..) = codec
        .from_str(
            r#"
> [!note]
> A note
"#,
            None,
        )
        .await?;
    assert_yaml_snapshot!(node, @r#"
    type: Article
    content:
      - type: Admonition
        admonitionType: Note
        content:
          - type: Paragraph
            content:
              - type: Text
                value:
                  string: A note
    "#);

    let (node, ..) = codec
        .from_str(
            r#"
> [!note]
> `A note`
"#,
            None,
        )
        .await?;
    assert_yaml_snapshot!(node, @r#"
    type: Article
    content:
      - type: Admonition
        admonitionType: Note
        content:
          - type: Paragraph
            content:
              - type: CodeInline
                code:
                  string: A note
    "#);

    Ok(())
}
