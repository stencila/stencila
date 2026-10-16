//! Content correspondence survives section wrapping, flattening, and small edits.

use eyre::{Result, eyre};
use pretty_assertions::assert_eq;
use stencila_node_compare::{
    AlignmentSignal, BudgetScope, CompareError, CompareOptions, Comparison, Difference, MatchRule,
    align_with_options, compare,
};
use stencila_node_path::{NodePath, NodeSlot};
use stencila_node_type::{NodeProperty, NodeType};
use stencila_schema::{
    Block, Figure, Node, Paragraph, Section,
    shortcuts::{art, h1, h2, p, sec, stb, t},
};

fn content_path(indices: &[usize]) -> NodePath {
    let mut path = NodePath::new();
    path.extend(
        indices
            .iter()
            .flat_map(|index| {
                [
                    NodeSlot::Property(NodeProperty::Content),
                    NodeSlot::Index(*index),
                ]
            })
            .collect::<Vec<_>>(),
    );
    path
}

fn identified_section(id: &str, content: Vec<Block>) -> Block {
    Block::Section(Section {
        id: Some(id.to_string()),
        ..Section::new(content)
    })
}

fn identified_paragraph(id: &str, text: &str) -> Block {
    Block::Paragraph(Paragraph {
        id: Some(id.to_string()),
        ..Paragraph::new(vec![t(text)])
    })
}

/// Every fixture also checks coverage, determinism, and swap symmetry.
fn comparison(left: &Node, right: &Node) -> Result<Comparison> {
    let result = compare(left, right)?;
    result.alignment().validate(left, right)?;
    assert_eq!(compare(left, right)?, result);
    assert_eq!(compare(right, left)?.invert(), result);
    Ok(result)
}

fn assert_pair(result: &Comparison, left: &[usize], right: &[usize]) -> Result<()> {
    let (_, paired, _) = result
        .alignment()
        .pairs()
        .find(|(node, ..)| node.path == content_path(left))
        .ok_or_else(|| eyre!("Expected a pair for {:?}", content_path(left)))?;
    assert_eq!(paired.path, content_path(right));
    Ok(())
}

#[test]
fn removing_section_wrappers_preserves_the_paragraph() -> Result<()> {
    for depth in [1, 4] {
        let mut wrapped = p([t("An unchanged paragraph inside section wrappers.")]);
        for _ in 0..depth {
            wrapped = sec([wrapped]);
        }
        let left = art([wrapped]);
        let right = art([p([t("An unchanged paragraph inside section wrappers.")])]);
        let result = comparison(&left, &right)?;
        assert_pair(&result, &vec![0; depth + 1], &[0])?;
        assert!(
            !result
                .differences()
                .iter()
                .any(|difference| matches!(difference, Difference::NodeTypeChanged { .. }))
        );
        assert!(result.differences().iter().any(|difference| matches!(
            difference,
            Difference::ParentChanged { left, .. } if left.node_type == NodeType::Paragraph
        )));
    }
    Ok(())
}

#[test]
fn flattened_sections_pair_headings_and_edited_paragraphs() -> Result<()> {
    let left = art([sec([
        h1([t("Results")]),
        sec([
            h2([t("Auxin signaling activates transport")]),
            p([t(
                "Auxin signaling activates the transport mechanism5 in these cells.",
            )]),
        ]),
    ])]);
    let right = art([
        sec([h1([t("Results")])]),
        sec([
            h1([t("Auxin signaling activates transport")]),
            p([t(
                "Auxin signaling activates the transport mechanism 5 in these cells.",
            )]),
        ]),
    ]);
    let result = comparison(&left, &right)?;
    assert_pair(&result, &[0, 1, 0], &[1, 0])?;
    assert_pair(&result, &[0, 1, 1], &[1, 1])?;
    let (_, _, info) = result
        .alignment()
        .pairs()
        .find(|(node, ..)| node.path == content_path(&[0, 1, 0]))
        .ok_or_else(|| eyre!("Expected the moved heading"))?;
    assert_eq!(info.rule, MatchRule::CrossParentReconciliation);
    assert!(
        info.evidence
            .iter()
            .any(|evidence| evidence.signal == AlignmentSignal::TextSimilarity)
    );
    // Matching must preserve both the heading level and raw text differences.
    for property in [NodeProperty::Level, NodeProperty::Value] {
        assert!(result.differences().iter().any(|difference| matches!(
            difference,
            Difference::ValueChanged { location, .. } if location.property == Some(property)
        )));
    }
    Ok(())
}

#[test]
fn a_short_heading_can_move_when_only_its_level_changes() -> Result<()> {
    let left = art([sec([h2([t("Results")])])]);
    let right = art([h1([t("Results")])]);
    assert_pair(&comparison(&left, &right)?, &[0, 0], &[0])
}

#[test]
fn weak_or_empty_text_does_not_identify_a_move() -> Result<()> {
    for (before, after) in [
        ("Results", "Result"),
        ("", "Unrelated"),
        (
            "Auxin signaling controls transport in these cells.",
            "A cloud of dust obscured the distant galaxy.",
        ),
        // Different short strings can share every trigram
        ("ACGAC", "CGACG"),
    ] {
        let left = art([
            identified_section("first", vec![p([t(before)])]),
            identified_section("second", Vec::new()),
        ]);
        let right = art([
            identified_section("first", Vec::new()),
            identified_section("second", vec![p([t(after)])]),
        ]);
        let result = comparison(&left, &right)?;
        assert!(
            !result
                .alignment()
                .pairs()
                .any(|(node, ..)| node.path == content_path(&[0, 0]))
        );
    }
    Ok(())
}

#[test]
fn a_figure_with_an_edited_caption_moves_to_the_end() -> Result<()> {
    let figure = |caption: &str| {
        Block::Figure(Figure {
            caption: Some(vec![p([t(caption)])]),
            ..Figure::new(Vec::new())
        })
    };
    let left = art([identified_section(
        "results",
        vec![figure(
            "Figure 1. Auxin signaling activates transport in these cells.",
        )],
    )]);
    let right = art([
        identified_section("results", Vec::new()),
        figure("Figure 1. Auxin signaling activates transport in these plant cells."),
    ]);
    let result = comparison(&left, &right)?;
    assert_pair(&result, &[0, 0], &[1])?;
    assert!(result.differences().iter().any(|difference| matches!(
        difference,
        Difference::ParentChanged { left, .. } if left.node_type == NodeType::Figure
    )));
    Ok(())
}

#[test]
fn ambiguous_similar_moves_remain_unpaired() -> Result<()> {
    let original = "Auxin signaling activates the transport mechanism in these cells.";
    let left = art([
        identified_section("first", vec![p([t(original)])]),
        identified_section("second", Vec::new()),
    ]);
    for alternatives in [
        vec![original, original],
        vec![
            "Auxin signaling activates the transport mechanism in these cells. A",
            "Auxin signaling activates the transport mechanism in these cells. B",
        ],
    ] {
        let right = art([
            identified_section("first", Vec::new()),
            identified_section(
                "second",
                alternatives.into_iter().map(|text| p([t(text)])).collect(),
            ),
        ]);
        let result = comparison(&left, &right)?;
        assert!(
            !result
                .alignment()
                .pairs()
                .any(|(node, ..)| node.path == content_path(&[0, 0]))
        );
    }
    Ok(())
}

#[test]
fn a_section_nested_only_on_one_side_is_not_aligned_twice() -> Result<()> {
    let text = "A paragraph that is long enough to be a meaningful move candidate.";
    let left = art([
        identified_paragraph("a", text),
        identified_section("x", vec![p([t(text)])]),
        p([t("Tail")]),
    ]);
    let right = art([
        p([t("Tail")]),
        sec([identified_section(
            "x",
            vec![identified_paragraph("a", text)],
        )]),
    ]);
    let result = comparison(&left, &right)?;
    // The container is reconciled before its descendants
    assert_pair(&result, &[1], &[1, 0])?;
    assert_pair(&result, &[1, 0], &[1, 0, 0])
}

#[test]
fn a_section_inside_a_styled_block_keeps_its_heading() -> Result<()> {
    let heading = "Methods for measuring auxin transport in plant cells";
    let body = "The measurements were repeated across three independent experiments.";
    let left = art([
        identified_section("x", vec![h1([t(heading)]), p([t(body)])]),
        p([t("Tail")]),
    ]);
    let right = art([
        p([t("Tail")]),
        stb(
            "note",
            [identified_section(
                "x",
                vec![h1([t(heading)]), p([t(body)])],
            )],
        ),
        h1([t(
            "Methods for measuring auxin transport in some plant cells",
        )]),
    ]);
    let result = comparison(&left, &right)?;
    assert_pair(&result, &[0], &[1, 0])?;
    assert_pair(&result, &[0, 0], &[1, 0, 0])
}

#[test]
fn an_exact_move_out_of_a_styled_block_beats_a_similar_paragraph() -> Result<()> {
    let exact = "Auxin signaling activates the transport mechanism in these cells.";
    let similar = "Auxin signaling activates the transport mechanism in these plant cells.";
    let left = art([
        identified_section("first", vec![stb("note", [p([t(exact)])])]),
        identified_section("second", vec![p([t(similar)])]),
    ]);
    let right = art([
        identified_section("first", Vec::new()),
        identified_section("second", Vec::new()),
        p([t(exact)]),
    ]);
    let result = comparison(&left, &right)?;
    assert_pair(&result, &[0, 0, 0], &[2])?;
    assert!(
        !result
            .alignment()
            .pairs()
            .any(|(node, ..)| node.path == content_path(&[1, 0]))
    );
    Ok(())
}

#[test]
fn similarity_reconciliation_obeys_the_cell_budget() -> Result<()> {
    let left = art([
        identified_section(
            "first",
            vec![p([t("A distinctive paragraph about auxin transport.")])],
        ),
        identified_section("second", Vec::new()),
    ]);
    let right = art([
        identified_section("first", Vec::new()),
        identified_section(
            "second",
            vec![
                p([t("A distinctive paragraph about auxin transport, revised.")]),
                p([t("A wholly unrelated paragraph about the weather.")]),
            ],
        ),
    ]);
    let options = CompareOptions {
        alignment_cell_budget: 1,
        ..Default::default()
    };
    for (left, right) in [(&left, &right), (&right, &left)] {
        let Err(error) = align_with_options(left, right, &options) else {
            return Err(eyre!("Expected the budget to be exhausted"));
        };
        // The error names the reconciliation phase rather than an empty path
        assert!(matches!(
            &error,
            CompareError::BudgetExhausted {
                scope: BudgetScope::Reconciliation {
                    node_type: NodeType::Paragraph
                },
                ..
            }
        ));
        assert!(error.to_string().starts_with("Reconciling `Paragraph`"));
    }
    Ok(())
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(64))]

    /// Restructuring repeated and edited blocks must preserve coverage and symmetry.
    #[test]
    fn varied_section_nesting_preserves_invariants(
        cases in proptest::collection::vec((0u8..4, 0u8..4, 0u8..4, proptest::bool::ANY), 1..8)
    ) {
        let wrap = |mut block: Block, depth: u8| {
            for _ in 0..depth { block = sec([block]); }
            block
        };
        let left = art(cases.iter().map(|(label, depth, _, _)| {
            wrap(p([t(format!("Observation {label}: auxin signaling activates transport in these cells."))]), *depth)
        }).collect::<Vec<_>>());
        let right = art(cases.iter().rev().map(|(label, _, depth, edited)| {
            let suffix = if *edited { " Revised." } else { "" };
            wrap(p([t(format!("Observation {label}: auxin signaling activates transport in these cells.{suffix}"))]), *depth)
        }).collect::<Vec<_>>());
        comparison(&left, &right).map_err(|error| proptest::test_runner::TestCaseError::fail(error.to_string()))?;
    }
}
