use stencila_codec::{
    DecodeOptions, EncodeOptions,
    eyre::{Result, bail},
    stencila_format::Format,
    stencila_schema::{
        Article, Block, Claim, ClaimType, Evidence, Node, Object, Primitive, Protocol, Question,
        Request, ResearchObjectRelation, ResearchObjectRelationKind,
        shortcuts::{p, t},
    },
};
use stencila_codec_markdown::{decode, encode};
use stencila_codec_text_trait::to_text;

fn decode_blocks(markdown: &str, format: Format) -> Result<Vec<Block>> {
    let (node, ..) = decode(
        markdown,
        Some(DecodeOptions {
            format: Some(format),
            ..Default::default()
        }),
    )?;

    let Node::Article(article) = node else {
        bail!("expected decoded Markdown to be an article")
    };

    Ok(article.content)
}

#[test]
fn decodes_untyped_smd_claim() -> Result<()> {
    let blocks = decode_blocks("::: claim #c1\n\nClaim text.\n\n:::\n", Format::Smd)?;

    let [Block::Claim(claim)] = blocks.as_slice() else {
        bail!("expected one claim block")
    };

    assert_eq!(claim.id.as_deref(), Some("c1"));
    assert_eq!(claim.claim_type, None);
    assert_eq!(to_text(&claim.content).trim(), "Claim text.");

    Ok(())
}

#[test]
fn decodes_smd_research_object_blocks() -> Result<()> {
    let blocks = decode_blocks(
        r#"::: evidence Evidence 1 #e1

Evidence text.

:::

::: question Question 1 #q1

Question text.

:::

::: protocol Protocol 1 #p1

Protocol text.

:::

::: request Request 1 #r1

Request text.

:::
"#,
        Format::Smd,
    )?;

    let [
        Block::Evidence(evidence),
        Block::Question(question),
        Block::Protocol(protocol),
        Block::Request(request),
    ] = blocks.as_slice()
    else {
        bail!("expected evidence, question, protocol, and request blocks")
    };

    assert_eq!(evidence.id.as_deref(), Some("e1"));
    assert_eq!(evidence.label.as_deref(), Some("Evidence 1"));
    assert_eq!(to_text(&evidence.content).trim(), "Evidence text.");
    assert_eq!(question.id.as_deref(), Some("q1"));
    assert_eq!(question.label.as_deref(), Some("Question 1"));
    assert_eq!(to_text(&question.content).trim(), "Question text.");
    assert_eq!(protocol.id.as_deref(), Some("p1"));
    assert_eq!(protocol.label.as_deref(), Some("Protocol 1"));
    assert_eq!(to_text(&protocol.content).trim(), "Protocol text.");
    assert_eq!(request.id.as_deref(), Some("r1"));
    assert_eq!(request.label.as_deref(), Some("Request 1"));
    assert_eq!(to_text(&request.content).trim(), "Request text.");

    Ok(())
}

#[test]
fn decodes_qmd_research_object_blocks() -> Result<()> {
    let blocks = decode_blocks(
        r#"::: {.claim #c1 label="Claim 1"}

## Claim title

Claim text.

:::

::: {.evidence #e1 label="Evidence 1"}

## Evidence title

Evidence text.

:::

::: {.question #q1 label="Question 1"}

## Question title

Question text.

:::

::: {.protocol #p1 label="Protocol 1"}

## Protocol title

Protocol text.

:::

::: {.request #r1 label="Request 1"}

## Request title

Request text.

:::
"#,
        Format::Qmd,
    )?;

    let [
        Block::Claim(claim),
        Block::Evidence(evidence),
        Block::Question(question),
        Block::Protocol(protocol),
        Block::Request(request),
    ] = blocks.as_slice()
    else {
        bail!("expected all five QMD research-object blocks")
    };

    assert_eq!(claim.claim_type, None);
    assert_eq!(claim.id.as_deref(), Some("c1"));
    assert_eq!(claim.label.as_deref(), Some("Claim 1"));
    assert_eq!(
        claim.options.title.as_ref().map(to_text).as_deref(),
        Some("Claim title")
    );
    assert_eq!(evidence.id.as_deref(), Some("e1"));
    assert_eq!(evidence.label.as_deref(), Some("Evidence 1"));
    assert_eq!(
        evidence.options.title.as_ref().map(to_text).as_deref(),
        Some("Evidence title")
    );
    assert_eq!(question.id.as_deref(), Some("q1"));
    assert_eq!(question.label.as_deref(), Some("Question 1"));
    assert_eq!(
        question.options.title.as_ref().map(to_text).as_deref(),
        Some("Question title")
    );
    assert_eq!(protocol.id.as_deref(), Some("p1"));
    assert_eq!(protocol.label.as_deref(), Some("Protocol 1"));
    assert_eq!(
        protocol.options.title.as_ref().map(to_text).as_deref(),
        Some("Protocol title")
    );
    assert_eq!(request.id.as_deref(), Some("r1"));
    assert_eq!(request.label.as_deref(), Some("Request 1"));
    assert_eq!(
        request.options.title.as_ref().map(to_text).as_deref(),
        Some("Request title")
    );

    Ok(())
}

#[test]
fn decodes_myst_research_object_directives() -> Result<()> {
    let blocks = decode_blocks(
        r#":::{claim} Claim title
:id: c1
:label: Claim 1

Claim text.
:::

:::{evidence} Evidence title
:id: e1
:label: Evidence 1

Evidence text.
:::

:::{question} Question title
:id: q1
:label: Question 1

Question text.
:::

:::{protocol} Protocol title
:id: p1
:label: Protocol 1

Protocol text.
:::

:::{request} Request title
:id: r1
:label: Request 1

Request text.
:::
"#,
        Format::Myst,
    )?;

    let [
        Block::Claim(claim),
        Block::Evidence(evidence),
        Block::Question(question),
        Block::Protocol(protocol),
        Block::Request(request),
    ] = blocks.as_slice()
    else {
        bail!("expected all five MyST research-object directives")
    };

    assert_eq!(claim.claim_type, None);
    assert_eq!(claim.id.as_deref(), Some("c1"));
    assert_eq!(claim.label.as_deref(), Some("Claim 1"));
    assert_eq!(
        claim.options.title.as_ref().map(to_text).as_deref(),
        Some("Claim title")
    );
    assert_eq!(evidence.id.as_deref(), Some("e1"));
    assert_eq!(evidence.label.as_deref(), Some("Evidence 1"));
    assert_eq!(
        evidence.options.title.as_ref().map(to_text).as_deref(),
        Some("Evidence title")
    );
    assert_eq!(question.id.as_deref(), Some("q1"));
    assert_eq!(question.label.as_deref(), Some("Question 1"));
    assert_eq!(
        question.options.title.as_ref().map(to_text).as_deref(),
        Some("Question title")
    );
    assert_eq!(protocol.id.as_deref(), Some("p1"));
    assert_eq!(protocol.label.as_deref(), Some("Protocol 1"));
    assert_eq!(
        protocol.options.title.as_ref().map(to_text).as_deref(),
        Some("Protocol title")
    );
    assert_eq!(request.id.as_deref(), Some("r1"));
    assert_eq!(request.label.as_deref(), Some("Request 1"));
    assert_eq!(
        request.options.title.as_ref().map(to_text).as_deref(),
        Some("Request title")
    );

    Ok(())
}

#[test]
fn decodes_typed_claims_in_all_dialects() -> Result<()> {
    let cases = [
        (
            Format::Smd,
            "::: theorem Theorem 1 #c1\n\nClaim text.\n\n:::\n",
            None,
        ),
        (
            Format::Qmd,
            "::: {.theorem #c1 label=\"Theorem 1\"}\n\n## Claim title\n\nClaim text.\n\n:::\n",
            Some("Claim title"),
        ),
        (
            Format::Myst,
            ":::{prf:theorem} Claim title\n:id: c1\n:label: Theorem 1\n\nClaim text.\n:::\n",
            Some("Claim title"),
        ),
    ];

    for (format, markdown, expected_title) in cases {
        let blocks = decode_blocks(markdown, format.clone())?;
        let [Block::Claim(claim)] = blocks.as_slice() else {
            bail!("expected one typed claim")
        };

        assert_eq!(claim.claim_type, Some(ClaimType::Theorem));
        assert_eq!(claim.id.as_deref(), Some("c1"));
        assert_eq!(claim.label.as_deref(), Some("Theorem 1"));
        assert_eq!(
            claim.options.title.as_ref().map(to_text).as_deref(),
            expected_title
        );
    }

    Ok(())
}

#[test]
fn decodes_myst_proof_extension_and_bare_proof_directives() -> Result<()> {
    let blocks = decode_blocks(
        r#"```{prf:theorem} Monotonicity
:id: theorem-monotonicity

The transformation is monotone.
```

```{proof} Monotonicity proof
:id: proof-monotonicity
:grounds: #theorem-monotonicity

The derivative is positive.
```
"#,
        Format::Myst,
    )?;

    let [Block::Claim(theorem), Block::Claim(proof)] = blocks.as_slice() else {
        bail!("expected theorem and proof claims")
    };
    assert_eq!(theorem.claim_type, Some(ClaimType::Theorem));
    assert_eq!(proof.claim_type, Some(ClaimType::Proof));
    assert_eq!(proof.id.as_deref(), Some("proof-monotonicity"));
    assert_eq!(
        proof.relations.as_deref().map(|relations| {
            relations
                .iter()
                .map(|relation| (relation.kind, relation.target.as_str()))
                .collect::<Vec<_>>()
        }),
        Some(vec![(
            ResearchObjectRelationKind::Grounds,
            "#theorem-monotonicity"
        )])
    );

    Ok(())
}

#[test]
fn decodes_all_relation_kinds_and_multi_targets() -> Result<()> {
    let cases = [
        (
            Format::Smd,
            r#"::: evidence
:supports: #c1, #c2
:supported_by: #c3
:opposes: #c4
:opposedBy: #c5
:addresses: #q1
:addressed-by: #q2
:follows: #p1
:grounds: #e1
:grounded-in: #s1
:requestFor: #s2
:request_target: #r1

Evidence text.

:::
"#,
        ),
        (
            Format::Qmd,
            r##"::: {.evidence supports="#c1, #c2" supported_by="#c3" opposes="#c4" opposedBy="#c5" addresses="#q1" addressed-by="#q2" follows="#p1" grounds="#e1" grounded-in="#s1" requestFor="#s2" request_target="#r1"}

Evidence text.

:::
"##,
        ),
        (
            Format::Myst,
            r#":::{evidence}
:supports: #c1, #c2
:supported_by: #c3
:opposes: #c4
:opposedBy: #c5
:addresses: #q1
:addressed-by: #q2
:follows: #p1
:grounds: #e1
:grounded-in: #s1
:requestFor: #s2
:request_target: #r1

Evidence text.
:::
"#,
        ),
    ];

    let expected = [
        (ResearchObjectRelationKind::Supports, "#c1"),
        (ResearchObjectRelationKind::Supports, "#c2"),
        (ResearchObjectRelationKind::SupportedBy, "#c3"),
        (ResearchObjectRelationKind::Opposes, "#c4"),
        (ResearchObjectRelationKind::OpposedBy, "#c5"),
        (ResearchObjectRelationKind::Addresses, "#q1"),
        (ResearchObjectRelationKind::AddressedBy, "#q2"),
        (ResearchObjectRelationKind::Follows, "#p1"),
        (ResearchObjectRelationKind::Grounds, "#e1"),
        (ResearchObjectRelationKind::IsGroundedIn, "#s1"),
        (ResearchObjectRelationKind::RequestFor, "#s2"),
        (ResearchObjectRelationKind::RequestTarget, "#r1"),
    ];

    for (format, markdown) in cases {
        let blocks = decode_blocks(markdown, format.clone())?;
        let [Block::Evidence(evidence)] = blocks.as_slice() else {
            bail!("expected one evidence block")
        };
        let actual = evidence
            .relations
            .iter()
            .flatten()
            .map(|relation| (relation.kind, relation.target.as_str()))
            .collect::<Vec<_>>();

        assert_eq!(actual, expected, "relations differed for {format}");
        assert_eq!(evidence.options.extra, None);
    }

    Ok(())
}

#[test]
fn preserves_typed_extra_attributes() -> Result<()> {
    let cases = [
        (
            Format::Smd,
            r#"::: question
:source: survey
:flag: true
:count: 3
:ratio: 1.5
:tags: ["a", "b"]

Question text.

:::
"#,
        ),
        (
            Format::Qmd,
            r#"::: {.question source="survey" flag=true count=3 ratio=1.5 tags=["a", "b"]}

Question text.

:::
"#,
        ),
        (
            Format::Myst,
            r#":::{question}
:source: survey
:flag: true
:count: 3
:ratio: 1.5
:tags: ["a", "b"]

Question text.
:::
"#,
        ),
    ];

    let expected = serde_json::json!({
        "source": "survey",
        "flag": true,
        "count": 3,
        "ratio": 1.5,
        "tags": ["a", "b"],
    });

    for (format, markdown) in cases {
        let blocks = decode_blocks(markdown, format.clone())?;
        let [Block::Question(question)] = blocks.as_slice() else {
            bail!("expected one question block")
        };
        let Some(extra) = &question.options.extra else {
            bail!("expected question extra attributes")
        };

        assert_eq!(
            serde_json::to_value(extra)?,
            expected,
            "extra differed for {format}"
        );
    }

    Ok(())
}

#[test]
fn semantically_round_trips_all_research_objects() -> Result<()> {
    let relation = |kind, target: &str| ResearchObjectRelation::new(kind, target.to_string());

    let mut claim = Claim::new(vec![p([t("Claim text.")])]);
    claim.id = Some("c1".to_string());
    claim.label = Some("Claim 1".to_string());
    claim.claim_type = Some(ClaimType::Statement);
    claim.options.title = Some(vec![t("Claim title")]);
    claim.relations = Some(vec![relation(
        ResearchObjectRelationKind::SupportedBy,
        "#e1",
    )]);
    claim.options.extra = Some(Object::from([("confidence", Primitive::Number(0.9))]));

    let mut evidence = Evidence::new(vec![p([t("Evidence text.")])]);
    evidence.id = Some("e1".to_string());
    evidence.label = Some("Evidence 1".to_string());
    evidence.options.title = Some(vec![t("Evidence title")]);
    evidence.relations = Some(vec![relation(ResearchObjectRelationKind::Supports, "#c1")]);
    evidence.options.extra = Some(Object::from([(
        "source",
        Primitive::String("survey".to_string()),
    )]));

    let mut question = Question::new(vec![p([t("Question text.")])]);
    question.id = Some("q1".to_string());
    question.label = Some("Question 1".to_string());
    question.options.title = Some(vec![t("Question title")]);
    question.relations = Some(vec![relation(
        ResearchObjectRelationKind::AddressedBy,
        "#c1",
    )]);
    question.options.extra = Some(Object::from([("open", Primitive::Boolean(true))]));

    let mut protocol = Protocol::new(vec![p([t("Protocol text.")])]);
    protocol.id = Some("p1".to_string());
    protocol.label = Some("Protocol 1".to_string());
    protocol.options.title = Some(vec![t("Protocol title")]);
    protocol.relations = Some(vec![relation(ResearchObjectRelationKind::Follows, "#p0")]);
    protocol.options.extra = Some(Object::from([("steps", Primitive::Integer(3))]));

    let mut request = Request::new(vec![p([t("Request text.")])]);
    request.id = Some("r1".to_string());
    request.label = Some("Request 1".to_string());
    request.options.title = Some(vec![t("Request title")]);
    request.relations = Some(vec![relation(
        ResearchObjectRelationKind::RequestTarget,
        "#c1",
    )]);
    request.options.extra = Some(Object::from([(
        "tags",
        Primitive::Array(
            [
                Primitive::String("review".to_string()),
                Primitive::String("urgent".to_string()),
            ]
            .into(),
        ),
    )]));

    let node = Node::Article(Article::new(vec![
        Block::Claim(claim),
        Block::Evidence(evidence),
        Block::Question(question),
        Block::Protocol(protocol),
        Block::Request(request),
    ]));
    let expected = serde_json::to_value(&node)?;

    for format in [Format::Markdown, Format::Smd, Format::Qmd, Format::Myst] {
        let (markdown, ..) = encode(
            &node,
            Some(EncodeOptions {
                format: Some(format.clone()),
                ..Default::default()
            }),
        )?;
        let (decoded, ..) = decode(
            &markdown,
            Some(DecodeOptions {
                format: Some(format.clone()),
                ..Default::default()
            }),
        )?;

        assert_eq!(
            serde_json::to_value(decoded)?,
            expected,
            "semantic round trip differed for {format}\n\n{markdown}"
        );
    }

    Ok(())
}

#[test]
fn does_not_capture_similarly_named_blocks() -> Result<()> {
    let cases = [
        (Format::Smd, "::: theorem-note\n\nText.\n\n:::\n"),
        (Format::Smd, "::: proofreading\n\nText.\n\n:::\n"),
        (Format::Smd, "::: evidence-note\n\nText.\n\n:::\n"),
        (Format::Qmd, "::: {.theorem-note}\n\nText.\n\n:::\n"),
        (Format::Qmd, "::: {.questionnaire}\n\nText.\n\n:::\n"),
        (Format::Myst, ":::{questionnaire}\n\nText.\n:::\n"),
    ];

    for (format, markdown) in cases {
        let blocks = decode_blocks(markdown, format.clone())?;
        assert!(
            blocks.iter().all(|block| !matches!(
                block,
                Block::Claim(..)
                    | Block::Evidence(..)
                    | Block::Question(..)
                    | Block::Protocol(..)
                    | Block::Request(..)
            )),
            "similarly named block was captured for {format}: {blocks:#?}"
        );
    }

    Ok(())
}

#[test]
fn decodes_nested_research_blocks_with_balanced_fences() -> Result<()> {
    let cases = [
        (
            Format::Smd,
            ":::: question\n\nOuter text.\n\n::: evidence\n\nInner text.\n\n:::\n\n::::\n",
        ),
        (
            Format::Qmd,
            ":::: {.question}\n\nOuter text.\n\n::: {.evidence}\n\nInner text.\n\n:::\n\n::::\n",
        ),
        (
            Format::Myst,
            ":::{question}\n\nOuter text.\n\n::::{evidence}\n\nInner text.\n\n::::\n\n:::\n",
        ),
    ];

    for (format, markdown) in cases {
        let blocks = decode_blocks(markdown, format.clone())?;
        let [Block::Question(question)] = blocks.as_slice() else {
            bail!("expected one outer question for {format}")
        };
        assert!(
            question
                .content
                .iter()
                .any(|block| matches!(block, Block::Evidence(..))),
            "expected nested evidence for {format}: {blocks:#?}"
        );
    }

    Ok(())
}
