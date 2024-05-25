use codec::{
    common::{eyre::Result, tokio},
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
    merge(&mut node, &edited, Some(vec![bob]))?;

    assert_yaml_snapshot!(node, {
      ".authors[].lastModified.value" => "redacted",
      ".content[].authors[].lastModified.value" => "redacted"
    });

    let (md, EncodeInfo { mapping, .. }) = codec.to_string(&node, None).await?;

    assert_snapshot!(md, @r###"
    ---
    authors:
    - type: AuthorRole
      author:
        type: Person
        givenNames:
        - Alice
      roleName: Writer
    - type: AuthorRole
      author:
        type: Person
        givenNames:
        - Bob
      roleName: Writer
    ---

    Hello, world!
    "###);

    assert_snapshot!(mapping, @r###"
    start     end        offsets   node_type+property                   authorship
       202    203     (202, 203)   Text                                 (1, 0, 0)
       203    214        (1, 11)   Text                                 (2, 1, 2)
       214    215        (11, 1)   Text                                 (1, 0, 0)
       202    215       (-12, 0)   Text.value
       202    215         (0, 0)   Text
       202    215         (0, 0)   Paragraph.content
       202    216         (0, 1)   Paragraph
       202    217         (0, 1)   Article.content
         0    217      (-202, 0)   Article
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
    merge(&mut node, &edited, Some(vec![bob]))?;

    assert_yaml_snapshot!(node, {
      ".authors[].lastModified.value" => "redacted",
      ".content[].authors[].lastModified.value" => "redacted"
    });

    let (md, EncodeInfo { mapping, .. }) = codec.to_string(&node, None).await?;

    assert_snapshot!(md, @r###"
    ---
    authors:
    - type: AuthorRole
      author:
        type: Person
        givenNames:
        - Alice
      roleName: Writer
    - type: AuthorRole
      author:
        type: Person
        givenNames:
        - Bob
      roleName: Writer
    ---

    ```python exec
    print('Hello, world!')
    ```
    "###);

    assert_snapshot!(mapping, @r###"
    start     end        offsets   node_type+property                   authorship
       205    211     (205, 211)   CodeChunk.programmingLanguage
       217    225       (12, 14)   CodeChunk                            (1, 0, 0)
       225    236        (8, 11)   CodeChunk                            (2, 1, 2)
       236    239        (11, 3)   CodeChunk                            (1, 0, 0)
       217    239       (-19, 0)   CodeChunk.code
       202    244       (-15, 5)   CodeChunk
       202    245         (0, 1)   Article.content
         0    245      (-202, 0)   Article
    "###);

    Ok(())
}
