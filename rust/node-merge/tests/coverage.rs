//! Every kind of node a container can hold survives being read back
//!
//! Rebuilding a container reads *all* of its items back out of the document, including
//! the ones no edit touches, so a node type that cannot make the round trip breaks a
//! merge that had nothing to do with it. That is a whole-schema property, and these
//! pin the cases that were actually broken.

use eyre::Result;

use stencila_node_merge::merge;
use stencila_schema::{Block, Inline, Node, Paragraph, Text, shortcuts::art};

/// An article of one paragraph with the given inlines
fn para(inlines: Vec<Inline>) -> Node {
    art([Block::Paragraph(Paragraph::new(inlines))])
}

#[test]
fn a_primitive_inline_beside_an_edit_does_not_break_the_merge() -> Result<()> {
    // `Inline` has primitive variants, and a container holding one used to fail to
    // read back, so an edit anywhere in that paragraph aborted the whole merge
    let left = para(vec![
        Inline::Text(Text::from("Answer: ")),
        Inline::Integer(41),
    ]);
    let right = para(vec![
        Inline::Text(Text::from("The answer: ")),
        Inline::Integer(41),
    ]);

    let merged = merge(&left, &right)?;
    assert_eq!(merged.report().replaces, 1, "{:?}", merged.report());

    Ok(())
}

#[test]
fn every_primitive_inline_survives_a_merge() -> Result<()> {
    for primitive in [
        Inline::Null(stencila_schema::Null),
        Inline::Boolean(true),
        Inline::Integer(-1),
        Inline::UnsignedInteger(1),
        Inline::Number(1.5),
    ] {
        let left = para(vec![Inline::Text(Text::from("Before")), primitive.clone()]);
        let right = para(vec![Inline::Text(Text::from("After")), primitive.clone()]);

        let merged = merge(&left, &right)
            .map_err(|error| eyre::eyre!("{primitive:?} broke the merge: {error}"))?;

        // Where the primitive ends up depends on how the comparison chose to pair the
        // paragraphs, which is not this test's business. What matters is that it is
        // still in the document, and still the value it was.
        assert!(
            contains(merged.node(), &primitive),
            "{primitive:?} was lost or altered by the merge"
        );
    }

    Ok(())
}

/// Whether an inline appears anywhere in a document
fn contains(node: &Node, inline: &Inline) -> bool {
    serde_json::to_string(node)
        .unwrap_or_default()
        .contains(&serde_json::to_string(inline).unwrap_or_default())
}

#[test]
fn inserting_into_an_absent_optional_container_works() -> Result<()> {
    // `Citation.content` is `Option<Vec<Inline>>`: a citation rendered on one side and
    // not the other gives a container that has to be created rather than rewritten.
    // Probing an absent optional property yields a null, which must not be taken for
    // an empty container (clearing one is an error) nor for a genuine null item.
    let cited = |content: Option<Vec<Inline>>| {
        para(vec![
            Inline::Text(Text::from("As shown ")),
            Inline::Citation(stencila_schema::Citation {
                options: Box::new(stencila_schema::CitationOptions {
                    content,
                    ..Default::default()
                }),
                ..stencila_schema::Citation::new("smith2020".to_string())
            }),
        ])
    };

    let left = cited(None);
    let right = cited(Some(vec![Inline::Text(Text::from("(Smith, 2020)"))]));

    let merged = merge(&left, &right)?;

    // The rendered citation is now in the document, wrapped in a suggestion
    let json = serde_json::to_string(merged.node())?;
    assert!(json.contains("(Smith, 2020)"), "the insertion was lost");
    assert!(json.contains("SuggestionInline"), "not wrapped: {json}");

    // No phantom null was introduced where the absent container used to be
    assert!(
        !json.contains(r#"{"type":"Null"}"#),
        "a null leaked in from the absent container: {json}"
    );

    // And the absent branch really was the one taken, so this test cannot quietly stop
    // exercising it. Rejecting the insertion leaves the property present but empty
    // rather than absent, which is the difference from the left document being noted.
    assert!(
        merged.report().unrepresentable.iter().any(|entry| matches!(
            entry.reason,
            stencila_node_merge::UnrepresentableReason::ContainerAbsentOnLeft { .. }
        )),
        "the absent container was not recorded: {:?}",
        merged.report()
    );

    Ok(())
}
