use codec::{
    common::{eyre::Result, tokio},
    format::Format,
    schema::{
        authorship, merge,
        shortcuts::{art, p, t},
        AuthorRole, AuthorRoleName, Person,
    },
    Codec, EncodeInfo,
};
use codec_markdown::MarkdownCodec;
use common_dev::insta::{assert_snapshot, assert_yaml_snapshot};

/// Test that mapping of Unicode characters is correct: ie uses character indices, not bytes indices
#[tokio::test]
async fn unicode() -> Result<()> {
    let codec = MarkdownCodec {};

    let (.., EncodeInfo { mapping, .. }) = codec.to_string(&art([p([t("1 👱")])]), None).await?;
    assert_eq!(mapping.entry_at(0).unwrap().range.len(), 3);

    let (.., EncodeInfo { mapping, .. }) = codec.to_string(&art([p([t("2 ❤️")])]), None).await?;
    assert_eq!(mapping.entry_at(0).unwrap().range.len(), 4);

    let (.., EncodeInfo { mapping, .. }) = codec.to_string(&art([p([t("4 🏳️‍🌈")])]), None).await?;
    assert_eq!(mapping.entry_at(0).unwrap().range.len(), 6);

    let (.., EncodeInfo { mapping, .. }) =
        codec.to_string(&art([p([t("7 👨‍👩‍👧‍👦")])]), None).await?;
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
    ---
    authors:
    - type: Person
      givenNames:
      - Alice
    - type: Person
      givenNames:
      - Bob
    ---

    Hello, world!
    "###);

    assert_snapshot!(mapping, @r###"
    start     end        offsets   node_type+property                   authorship
        94     95       (94, 95)   Text                                 (1, 0, 0)
        95    106        (1, 11)   Text                                 (2, 1, 2)
       106    107        (11, 1)   Text                                 (1, 0, 0)
        94    107       (-12, 0)   Text.value
        94    107         (0, 0)   Text
        94    107         (0, 0)   Paragraph.content
        94    108         (0, 1)   Paragraph
        94    109         (0, 1)   Article.content
         0    109       (-94, 0)   Article
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
    merge(&mut node, &edited, Some(Format::Markdown), Some(vec![bob]))?;

    assert_yaml_snapshot!(node, {
      ".authors[].lastModified.value" => "redacted",
      ".content[].authors[].lastModified.value" => "redacted"
    });

    let (md, EncodeInfo { mapping, .. }) = codec.to_string(&node, None).await?;

    assert_snapshot!(md, @r###"
    ---
    authors:
    - type: Person
      givenNames:
      - Alice
    - type: Person
      givenNames:
      - Bob
    ---

    ```python exec
    print('Hello, world!')
    ```
    "###);

    assert_snapshot!(mapping, @r###"
    start     end        offsets   node_type+property                   authorship
        97    103      (97, 103)   CodeChunk.programmingLanguage
       109    117       (12, 14)   CodeChunk                            (1, 0, 0)
       117    128        (8, 11)   CodeChunk                            (2, 1, 2)
       128    131        (11, 3)   CodeChunk                            (1, 0, 0)
       109    131       (-19, 0)   CodeChunk.code
        94    136       (-15, 5)   CodeChunk
        94    137         (0, 1)   Article.content
         0    137       (-94, 0)   Article
    "###);

    Ok(())
}
