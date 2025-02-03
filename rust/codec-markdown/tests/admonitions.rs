use codec::{
    common::{eyre::Result, tokio},
    Codec,
};
use codec_markdown::MarkdownCodec;
use common_dev::insta::assert_yaml_snapshot;

/// Regression test for cases where the first inline of the first paragraph
/// is not a `Text` node.
#[tokio::test]
async fn non_text_first_inline() -> Result<()> {
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

/// Test that lists in lists admonitions do not break out of
/// the admonition
#[tokio::test]
async fn contains_lists() -> Result<()> {
    let codec = MarkdownCodec {};

    let (node, ..) = codec
        .from_str(
            r#"
> [!info]+ Thinking
>
> Para one
> 
> - apple
> - pear
> 
> Para two
> 
> 1. one
> 2. two
> 
> Para three
"#,
            None,
        )
        .await?;
    assert_yaml_snapshot!(node, @r#"
    type: Article
    content:
      - type: Admonition
        admonitionType: Info
        title:
          - type: Text
            value:
              string: Thinking
        isFolded: true
        content:
          - type: Paragraph
            content:
              - type: Text
                value:
                  string: Para one
          - type: List
            items:
              - type: ListItem
                content:
                  - type: Paragraph
                    content:
                      - type: Text
                        value:
                          string: apple
              - type: ListItem
                content:
                  - type: Paragraph
                    content:
                      - type: Text
                        value:
                          string: pear
            order: Unordered
          - type: Paragraph
            content:
              - type: Text
                value:
                  string: Para two
          - type: List
            items:
              - type: ListItem
                content:
                  - type: Paragraph
                    content:
                      - type: Text
                        value:
                          string: one
              - type: ListItem
                content:
                  - type: Paragraph
                    content:
                      - type: Text
                        value:
                          string: two
            order: Ascending
          - type: Paragraph
            content:
              - type: Text
                value:
                  string: Para three
    "#);

    Ok(())
}
