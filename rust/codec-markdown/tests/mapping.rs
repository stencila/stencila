use codec::{
    Codec, EncodeInfo,
    eyre::Result,
    format::Format,
    schema::{
        AuthorRole, AuthorRoleName, Person, authorship, merge,
        shortcuts::{art, p, t},
    },
};
use codec_markdown::MarkdownCodec;
use insta::{assert_snapshot, assert_yaml_snapshot};

/// Test that mapping of Unicode characters is correct: ie uses character indices, not bytes indices
#[tokio::test]
#[allow(clippy::unwrap_used)]
async fn unicode() -> Result<()> {
    let codec = MarkdownCodec {};

    let (.., EncodeInfo { mapping, .. }) = codec.to_string(&art([p([t("1 👱")])]), None).await?;
    assert_eq!(mapping.entry_at(0).unwrap().range.len(), 3);

    let (.., EncodeInfo { mapping, .. }) = codec.to_string(&art([p([t("2 ❤️")])]), None).await?;
    assert_eq!(mapping.entry_at(0).unwrap().range.len(), 4);

    let (.., EncodeInfo { mapping, .. }) = codec.to_string(&art([p([t("4 🏳️‍🌈")])]), None).await?;
    assert_eq!(mapping.entry_at(0).unwrap().range.len(), 6);

    let (.., EncodeInfo { mapping, .. }) = codec.to_string(&art([p([t("7 👨‍👩‍👧‍👦")])]), None).await?;
    assert_eq!(mapping.entry_at(0).unwrap().range.len(), 9);

    Ok(())
}

/// Test that mapping of paragraph with authorship information is correct
#[tokio::test]
async fn paragraph() -> Result<()> {
    let codec = MarkdownCodec {};

    let alice = AuthorRole::person(
        Person {
            given_names: Some(vec!["Alice".to_string()]),
            ..Default::default()
        },
        AuthorRoleName::Writer,
    );
    let bob = AuthorRole::person(
        Person {
            given_names: Some(vec!["Bob".to_string()]),
            ..Default::default()
        },
        AuthorRoleName::Writer,
    );

    let (mut node, ..) = codec.from_str("Hi!", None).await?;
    authorship(&mut node, vec![alice])?;

    let (edited, ..) = codec.from_str("Hello, world!", None).await?;
    merge(&mut node, &edited, Some(Format::Markdown), Some(vec![bob]))?;

    assert_yaml_snapshot!(node, {
      ".authors[].lastModified.value" => "redacted",
      ".content[].authors[].lastModified.value" => "redacted"
    });

    let (md, EncodeInfo { mapping, .. }) = codec.to_string(&node, None).await?;

    assert_snapshot!(md, @r###"
    Hello, world!
    "###);

    assert_snapshot!(mapping, @r###"
    start     end        offsets   node_type+property                   authorship
         0      1         (0, 1)   Text                                 (1, 0, 0)
         1     12        (1, 11)   Text                                 (2, 1, 2)
        12     13        (11, 1)   Text                                 (1, 0, 0)
         0     13       (-12, 0)   Text.value
         0     13         (0, 0)   Text
         0     13         (0, 0)   Paragraph.content
         0     14         (0, 1)   Paragraph
         0     15         (0, 1)   Article.content
         0     15         (0, 0)   Article
    "###);

    Ok(())
}

/// Test that mapping of a code chunk with authorship information is correct
#[tokio::test]
async fn code_chunk() -> Result<()> {
    let codec = MarkdownCodec {};

    let alice = AuthorRole::person(
        Person {
            given_names: Some(vec!["Alice".to_string()]),
            ..Default::default()
        },
        AuthorRoleName::Writer,
    );
    let bob = AuthorRole::person(
        Person {
            given_names: Some(vec!["Bob".to_string()]),
            ..Default::default()
        },
        AuthorRoleName::Writer,
    );

    let (mut node, ..) = codec
        .from_str(
            r#"
```python exec
print('Hi!')
```
"#,
            None,
        )
        .await?;
    authorship(&mut node, vec![alice])?;

    let (edited, ..) = codec
        .from_str(
            r#"
```python exec
print('Hello, world!')
```
"#,
            None,
        )
        .await?;
    merge(&mut node, &edited, Some(Format::Smd), Some(vec![bob]))?;

    assert_yaml_snapshot!(node, {
      ".authors[].lastModified.value" => "redacted",
      ".content[].authors[].lastModified.value" => "redacted"
    });

    let (md, EncodeInfo { mapping, .. }) = codec.to_string(&node, None).await?;

    assert_snapshot!(md, @r###"
    ```python exec
    print('Hello, world!')
    ```
    "###);

    assert_snapshot!(mapping, @r###"
    start     end        offsets   node_type+property                   authorship
         3      9         (3, 9)   CodeChunk.programmingLanguage
        15     23       (12, 14)   CodeChunk                            (1, 0, 0)
        23     34        (8, 11)   CodeChunk                            (2, 1, 2)
        34     37        (11, 3)   CodeChunk                            (1, 0, 0)
        15     37       (-19, 0)   CodeChunk.code
         0     42       (-15, 5)   CodeChunk
         0     43         (0, 1)   Article.content
         0     43         (0, 0)   Article
    "###);

    Ok(())
}
