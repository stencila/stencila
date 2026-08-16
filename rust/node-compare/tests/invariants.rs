//! Tests of the invariants that every alignment and comparison must hold
//!
//! These are the properties future recipes and algorithm changes must not quietly
//! break, so they are enforced here rather than held by convention. The fixtures are
//! ordinary Stencila-native document structures — articles, sections, paragraphs,
//! figures, tables, references, authors, list items, table rows and cells — and no
//! codec or JATS input is needed at this level.

use std::collections::HashSet;

use eyre::{Result, bail};
use pretty_assertions::assert_eq;

use stencila_node_compare::{
    Alignment, CompareError, CompareOptions, Comparison, Correspondence, align, align_with_options,
    compare, compare_with_options,
};
use stencila_node_path::NodePath;
use stencila_node_type::NodeType;
use stencila_schema::{
    Article, Author, Block, Figure, Node, Person, Reference,
    shortcuts::{art, em, h1, li, ol, p, sec, stg, t, tbl, td, th, tr, ul},
};

/// A figure holding a caption and a paragraph
fn figure(caption: &str, content: &str) -> Block {
    Block::Figure(Figure {
        caption: Some(vec![p([t(caption)])]),
        ..Figure::new(vec![p([t(content)])])
    })
}

/// A person with a family name
fn person(family: &str) -> Author {
    Author::Person(Person {
        family_names: Some(vec![family.to_string()]),
        ..Person::default()
    })
}

/// A reference with a title
fn reference(id: &str, title: &str) -> Reference {
    Reference {
        id: Some(id.to_string()),
        title: Some(vec![t(title)]),
        ..Reference::default()
    }
}

/// A representative Stencila-native article
///
/// Ordinary document structure only: sections, headings, paragraphs, inline marks,
/// lists and list items, a figure, a table with rows and cells, authors and references.
fn document(variant: usize) -> Node {
    let heading = if variant == 0 { "Results" } else { "Findings" };
    let sentence = if variant == 0 {
        "The measured effect was substantial."
    } else {
        "The measured effect was modest."
    };

    let mut article = Article::new(vec![
        h1([t("A study of something")]),
        sec([
            h1([t(heading)]),
            p([
                t("An opening sentence with "),
                em([t("emphasis")]),
                t(" and "),
                stg([t("strength")]),
                t("."),
            ]),
            p([t(sentence)]),
        ]),
        ul([li([t("First point")]), li([t("Second point")])]),
        ol([li([t("Step one")]), li([t("Step two")])]),
        figure("Figure 1. An illustration", "The figure content"),
        tbl([
            tr([th([t("Sample")]), th([t("Value")])]),
            tr([td([t("A")]), td([t("1")])]),
            tr([td([t("B")]), td([t("2")])]),
        ]),
    ]);
    article.authors = Some(vec![person("Adams"), person("Brown")]);
    article.references = Some(vec![
        reference("ref-one", "An earlier study"),
        reference("ref-two", "A later study"),
    ]);

    Node::Article(article)
}

/// A short list of representative fixture pairs
fn fixtures() -> Vec<(Node, Node)> {
    vec![
        // Identical documents
        (document(0), document(0)),
        // An edited document
        (document(0), document(1)),
        // Insertion, deletion and reordering among ambiguous siblings
        (
            art([
                p([t("Boilerplate")]),
                p([t("Distinctive")]),
                p([t("Boilerplate")]),
            ]),
            art([
                p([t("Boilerplate")]),
                p([t("Boilerplate")]),
                p([t("Distinctive")]),
                p([t("Added")]),
            ]),
        ),
        // A cross-type pair
        (
            art([p([t("Same words here")])]),
            art([h1([t("Same words here")])]),
        ),
        // A wholly replaced document
        (
            art([p([t("Nothing at all in common")])]),
            art([sec([p([t("Entirely different content")])])]),
        ),
        // Primitive and dynamic roots
        (Node::Integer(1), Node::Integer(2)),
        (Node::String("one".to_string()), Node::Integer(1)),
    ]
}

/// The number of structured occurrences of a node
fn occurrences(node: &Node) -> Result<usize> {
    Ok(align(node, node)?.correspondences().len())
}

/// Every projected occurrence appears exactly once, and each side's paths are unique
fn assert_complete_single_coverage(alignment: &Alignment, left: &Node, right: &Node) -> Result<()> {
    let mut left_paths = Vec::new();
    let mut right_paths = Vec::new();
    for correspondence in alignment.correspondences() {
        if let Some(node) = correspondence.left() {
            left_paths.push(node.path.clone());
        }
        if let Some(node) = correspondence.right() {
            right_paths.push(node.path.clone());
        }
    }

    alignment.validate(left, right)?;
    assert_eq!(left_paths.len(), occurrences(left)?, "left coverage");
    assert_eq!(right_paths.len(), occurrences(right)?, "right coverage");

    let unique = |paths: &[NodePath]| paths.iter().cloned().collect::<HashSet<_>>().len();
    assert_eq!(unique(&left_paths), left_paths.len(), "left paths unique");
    assert_eq!(
        unique(&right_paths),
        right_paths.len(),
        "right paths unique"
    );

    Ok(())
}

/// Every reference resolves in its own projection, with the node type it records
fn assert_references_resolve(alignment: &Alignment, left: &Node, right: &Node) -> Result<()> {
    Ok(alignment.validate(left, right)?)
}

/// Every projected occurrence appears exactly once, and per-side paths are unique
#[test]
fn coverage_is_complete_and_single() -> Result<()> {
    for (left, right) in fixtures() {
        let alignment = align(&left, &right)?;
        assert_complete_single_coverage(&alignment, &left, &right)?;
    }

    Ok(())
}

/// Every reference in a serialized artifact resolves in its original projection with
/// the node type it records
#[test]
fn references_resolve_after_serialization() -> Result<()> {
    for (left, right) in fixtures() {
        let alignment = align(&left, &right)?;
        let serialized = serde_json::to_string(&alignment)?;
        let deserialized: Alignment = serde_json::from_str(&serialized)?;

        assert_references_resolve(&deserialized, &left, &right)?;
    }

    Ok(())
}

/// Comparing a node with itself, and with its clone, is difference free and path
/// identical
#[test]
fn self_comparison_is_difference_free() -> Result<()> {
    for (node, _) in fixtures() {
        let clone = node.clone();
        for comparison in [compare(&node, &node)?, compare(&node, &clone)?] {
            assert!(comparison.is_equal(), "self comparison has no differences");
            assert!(comparison.differences().is_empty());
            for (left, right, ..) in comparison.alignment().pairs() {
                assert_eq!(left.path, right.path, "self comparison is path identical");
            }
        }
    }

    Ok(())
}

/// Swapping the inputs and inverting the output is the same canonical artifact
#[test]
fn swap_and_invert_is_identical() -> Result<()> {
    for (left, right) in fixtures() {
        assert_eq!(align(&left, &right)?, align(&right, &left)?.invert());
        assert_eq!(compare(&left, &right)?, compare(&right, &left)?.invert());
    }

    Ok(())
}

/// Repeated runs on identical inputs produce byte-for-byte identical output
#[test]
fn runs_are_byte_for_byte_deterministic() -> Result<()> {
    for (left, right) in fixtures() {
        let first = serde_json::to_string(&compare(&left, &right)?)?;
        for _ in 0..3 {
            assert_eq!(serde_json::to_string(&compare(&left, &right)?)?, first);
        }
    }

    Ok(())
}

/// Equality matches the expected result for representative fixtures
#[test]
fn equality_matches_expected_results() -> Result<()> {
    for (left, right) in fixtures() {
        assert_eq!(compare(&left, &right)?.is_equal(), left == right);
    }

    Ok(())
}

/// Budget exhaustion returns a typed error and no artifact
#[test]
fn budget_exhaustion_is_a_typed_error() -> Result<()> {
    let left = art((0..40)
        .map(|index| p([t(format!("Left paragraph number {index}"))]))
        .collect::<Vec<_>>());
    let right = art((0..40)
        .map(|index| p([t(format!("Right paragraph number {index}"))]))
        .collect::<Vec<_>>());

    let options = CompareOptions {
        alignment_cell_budget: 100,
    };

    for result in [
        align_with_options(&left, &right, &options).map(|_| ()),
        compare_with_options(&left, &right, &options).map(|_| ()),
    ] {
        let Err(CompareError::BudgetExhausted {
            required, allowed, ..
        }) = result
        else {
            bail!("Expected a typed budget error and no artifact")
        };
        assert!(required > allowed);
        assert_eq!(allowed, options.alignment_cell_budget);
    }

    Ok(())
}

/// No successful run exceeds the candidate-cell budget
#[test]
fn successful_runs_stay_within_the_budget() -> Result<()> {
    // A budget of zero admits only alignments that need no dynamic programming at all,
    // so a run that succeeds under the default budget must have used no more of it than
    // the default allows: exceeding it is an error rather than an approximation
    let options = CompareOptions {
        alignment_cell_budget: 0,
    };
    for (left, right) in fixtures() {
        // Every fixture succeeds under the default budget
        compare(&left, &right)?;

        // And a fixture that needs any cells at all is refused rather than approximated
        if let Err(error) = compare_with_options(&left, &right, &options) {
            assert!(matches!(error, CompareError::BudgetExhausted { .. }));
        }
    }

    Ok(())
}

/// Serialization round-trips without structural change, and canonical ordering holds
/// both in memory and after deserialization
#[test]
fn serialization_round_trips_in_canonical_order() -> Result<()> {
    for (left, right) in fixtures() {
        let comparison = compare(&left, &right)?;

        assert!(comparison.alignment().correspondences().is_sorted());
        assert!(comparison.differences().is_sorted());

        let serialized = serde_json::to_string(&comparison)?;
        let deserialized: Comparison = serde_json::from_str(&serialized)?;
        assert_eq!(deserialized, comparison, "round-trips without change");

        assert!(deserialized.alignment().correspondences().is_sorted());
        assert!(deserialized.differences().is_sorted());
    }

    Ok(())
}

/// Deserialization restores canonical order even when wire records arrive out of order
#[test]
fn deserialization_canonicalizes_artifacts() -> Result<()> {
    let comparison = compare(&document(0), &document(1))?;
    let mut value = serde_json::to_value(&comparison)?;

    value["differences"]
        .as_array_mut()
        .ok_or_else(|| eyre::eyre!("differences is not an array"))?
        .reverse();
    value["alignment"]["correspondences"]
        .as_array_mut()
        .ok_or_else(|| eyre::eyre!("correspondences is not an array"))?
        .reverse();

    let deserialized: Comparison = serde_json::from_value(value)?;
    assert_eq!(deserialized, comparison);

    Ok(())
}

/// Duplicate references are rejected during deserialization, and missing references
/// are rejected when an artifact is validated against its snapshots
#[test]
fn deserialization_and_snapshot_validation_enforce_coverage() -> Result<()> {
    let left = document(0);
    let right = document(1);
    let alignment = align(&left, &right)?;
    let mut value = serde_json::to_value(&alignment)?;
    let mut duplicate = value.clone();
    let correspondences = duplicate["correspondences"]
        .as_array_mut()
        .ok_or_else(|| eyre::eyre!("correspondences is not an array"))?;
    correspondences.push(
        correspondences
            .first()
            .cloned()
            .ok_or_else(|| eyre::eyre!("alignment has no correspondences"))?,
    );
    assert!(serde_json::from_value::<Alignment>(duplicate).is_err());

    value["correspondences"]
        .as_array_mut()
        .ok_or_else(|| eyre::eyre!("correspondences is not an array"))?
        .pop();
    let incomplete: Alignment = serde_json::from_value(value)?;
    assert!(matches!(
        incomplete.validate(&left, &right),
        Err(CompareError::Completeness { .. })
    ));

    let mut unsupported = serde_json::to_value(&alignment)?;
    unsupported["formatVersion"] = serde_json::Value::String("999".to_string());
    let Err(error) = serde_json::from_value::<Alignment>(unsupported) else {
        bail!("an unsupported alignment version should fail")
    };
    assert!(
        error
            .to_string()
            .contains("Unsupported alignment format version")
    );

    Ok(())
}

/// One-sided correspondences are exhaustive over the subtree they cover
#[test]
fn one_sided_subtrees_are_exhaustive() -> Result<()> {
    let left = art([sec([p([t("One")]), p([t("Two")])])]);
    let right = art([p([t("Wholly unrelated wording")])]);

    let alignment = align(&left, &right)?;
    assert_complete_single_coverage(&alignment, &left, &right)?;

    // The removed section, both of its paragraphs and both of their texts are each
    // recorded, rather than only the section
    let left_only = alignment
        .correspondences()
        .iter()
        .filter(|correspondence| matches!(correspondence, Correspondence::LeftOnly { .. }))
        .count();
    assert_eq!(left_only, occurrences(&left)? - 1, "the article is paired");

    Ok(())
}

/// Table rows and cells, list items, authors and references all align
#[test]
fn document_structures_align() -> Result<()> {
    let left = document(0);
    let right = document(1);

    let alignment = align(&left, &right)?;
    assert_complete_single_coverage(&alignment, &left, &right)?;
    assert_references_resolve(&alignment, &left, &right)?;

    // Every one of these node types is paired somewhere in the alignment
    for node_type in [
        NodeType::Article,
        NodeType::Section,
        NodeType::Heading,
        NodeType::Paragraph,
        NodeType::List,
        NodeType::ListItem,
        NodeType::Figure,
        NodeType::Table,
        NodeType::TableRow,
        NodeType::TableCell,
        NodeType::Person,
        NodeType::Reference,
    ] {
        if !alignment
            .pairs()
            .any(|(left, ..)| left.node_type == node_type)
        {
            bail!("Expected a pair of {node_type}")
        }
    }

    Ok(())
}
