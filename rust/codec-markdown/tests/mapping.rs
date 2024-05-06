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

    assert_yaml_snapshot!(node, @r###"
    ---
    type: Article
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
    provenance:
      - type: ProvenanceCount
        provenanceCategory: Hw
        characterCount: 2
        characterPercent: 15
      - type: ProvenanceCount
        provenanceCategory: HwHe
        characterCount: 11
        characterPercent: 84
    content:
      - type: Paragraph
        content:
          - type: Text
            value:
              string: "Hello, world!"
              authorship:
                - - 1
                  - 0
                  - 0
                  - 1
                - - 2
                  - 1
                  - 2
                  - 11
                - - 1
                  - 0
                  - 0
                  - 1
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
        provenance:
          - type: ProvenanceCount
            provenanceCategory: Hw
            characterCount: 2
            characterPercent: 15
          - type: ProvenanceCount
            provenanceCategory: HwHe
            characterCount: 11
            characterPercent: 84
    "###);

    let (md, EncodeInfo { mapping, .. }) = codec.to_string(&node, None).await?;

    assert_snapshot!(md, @r###"
    ---
    provenance:
    - type: ProvenanceCount
      provenanceCategory: Hw
      characterCount: 2
      characterPercent: 15
    - type: ProvenanceCount
      provenanceCategory: HwHe
      characterCount: 11
      characterPercent: 84
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
       401    402     (401, 402)   Text                                 (1, 0, 0)
       402    413        (1, 11)   Text                                 (2, 1, 2)
       413    414        (11, 1)   Text                                 (1, 0, 0)
       401    414       (-12, 0)   Text.value
       401    414         (0, 0)   Text
       401    414         (0, 0)   Paragraph.content
       401    415         (0, 1)   Paragraph
       401    416         (0, 1)   Article.content
         0    416      (-401, 0)   Article
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

    assert_yaml_snapshot!(node, @r###"
    ---
    type: Article
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
    provenance:
      - type: ProvenanceCount
        provenanceCategory: Hw
        characterCount: 11
        characterPercent: 50
      - type: ProvenanceCount
        provenanceCategory: HwHe
        characterCount: 11
        characterPercent: 50
    content:
      - type: CodeChunk
        code:
          string: "print('Hello, world!')"
          authorship:
            - - 1
              - 0
              - 0
              - 8
            - - 2
              - 1
              - 2
              - 11
            - - 1
              - 0
              - 0
              - 3
        programmingLanguage: python
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
        provenance:
          - type: ProvenanceCount
            provenanceCategory: Hw
            characterCount: 11
            characterPercent: 50
          - type: ProvenanceCount
            provenanceCategory: HwHe
            characterCount: 11
            characterPercent: 50
    "###);

    let (md, EncodeInfo { mapping, .. }) = codec.to_string(&node, None).await?;

    assert_snapshot!(md, @r###"
    ---
    provenance:
    - type: ProvenanceCount
      provenanceCategory: Hw
      characterCount: 11
      characterPercent: 50
    - type: ProvenanceCount
      provenanceCategory: HwHe
      characterCount: 11
      characterPercent: 50
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
       405    411     (405, 411)   CodeChunk.programmingLanguage
       417    425       (12, 14)   CodeChunk                            (1, 0, 0)
       425    436        (8, 11)   CodeChunk                            (2, 1, 2)
       436    439        (11, 3)   CodeChunk                            (1, 0, 0)
       417    439       (-19, 0)   CodeChunk.code
       402    444       (-15, 5)   CodeChunk
       402    445         (0, 1)   Article.content
         0    445      (-402, 0)   Article
    "###);

    Ok(())
}
