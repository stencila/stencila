//! Local identifiers and cross-references compare through the document alignment.

use eyre::{Result, ensure};
use serde_json::{Value, json};
use stencila_node_compare::{
    Comparison, Correspondence, Difference, ScalarValue, ValueState, compare,
};
use stencila_node_type::{NodeProperty, NodeType};
use stencila_schema::Node;

fn document(content: Value, references: Value) -> Result<Node> {
    Ok(serde_json::from_value(json!({
        "type": "Article", "content": content, "references": references
    }))?)
}

fn text(value: &str) -> Value {
    json!({"type": "Text", "value": value})
}

fn reference(id: &str, title: &str) -> Value {
    json!({"type": "Reference", "id": id, "title": [text(title)]})
}

fn paragraph(content: Value) -> Value {
    json!({"type": "Paragraph", "content": content})
}

fn citation(target: &str) -> Value {
    json!({"type": "Citation", "target": target})
}

fn link(target: &str) -> Value {
    json!({"type": "Link", "target": target, "content": [text("See the results")]})
}

/// Require the same complete artifact in both comparison directions.
fn symmetric_comparison(left: &Node, right: &Node) -> Result<Comparison> {
    let comparison = compare(left, right)?;
    assert_eq!(comparison, compare(right, left)?.invert());
    Ok(comparison)
}

fn target_changes(comparison: &Comparison) -> Vec<&Difference> {
    comparison
        .differences()
        .iter()
        .filter(|difference| difference.property() == Some(NodeProperty::Target))
        .collect()
}

#[test]
fn renamed_reference_targets_preserve_citation_pairing() -> Result<()> {
    for rendered in [false, true] {
        let article = |id| {
            let mut citation = citation(id);
            if rendered {
                citation["content"] = json!([text("[1]")]);
            }
            document(
                json!([paragraph(json!([citation]))]),
                json!([reference(id, "Auxin signalling")]),
            )
        };
        let comparison = symmetric_comparison(&article("c14")?, &article("b13")?)?;
        assert_eq!(
            comparison
                .alignment()
                .pairs()
                .filter(|(left, ..)| left.node_type == NodeType::Citation)
                .count(),
            1,
            "Renaming a reference must not unpair its citations"
        );
        assert!(comparison.is_equal(), "{comparison:#?}");
    }
    Ok(())
}

#[test]
fn renamed_targets_in_citation_groups_are_equal() -> Result<()> {
    let article = |first, second| {
        document(
            json!([paragraph(
                json!([{"type": "CitationGroup", "items": [citation(first), citation(second)]}])
            )]),
            json!([
                reference(first, "Auxin signalling"),
                reference(second, "Root growth")
            ]),
        )
    };
    let comparison = symmetric_comparison(&article("c1", "c2")?, &article("b0", "b1")?)?;
    assert!(comparison.is_equal(), "{comparison:#?}");
    Ok(())
}

#[test]
fn resolved_citation_copies_do_not_make_reference_ids_ambiguous() -> Result<()> {
    let article = |id| {
        let reference = reference(id, "Auxin signalling");
        let mut citation = citation(id);
        citation["cites"] = reference.clone();
        document(
            json!([paragraph(json!([citation.clone(), citation]))]),
            json!([reference]),
        )
    };
    let comparison = symmetric_comparison(&article("c1")?, &article("b0")?)?;
    assert!(comparison.is_equal(), "{comparison:#?}");
    Ok(())
}

#[test]
fn renamed_internal_link_targets_are_equal() -> Result<()> {
    let article = |id| {
        document(
            json!([
                paragraph(json!([link(&format!("#{id}"))])),
                {"type": "Section", "id": id, "content": [paragraph(json!([text("Results")]))]}
            ]),
            json!([]),
        )
    };
    assert!(symmetric_comparison(&article("s1")?, &article("results")?)?.is_equal());
    Ok(())
}

#[test]
fn local_id_changes_and_presence_are_ignored() -> Result<()> {
    for right_id in [json!("new"), Value::Null, json!("")] {
        let node = |id| {
            serde_json::from_value::<Node>(
                json!({"type": "Paragraph", "id": id, "content": [text("Body")]}),
            )
        };
        let comparison = symmetric_comparison(&node(json!("old"))?, &node(right_id)?)?;
        assert!(comparison.is_equal(), "{comparison:#?}");
    }
    Ok(())
}

#[test]
fn citing_a_different_aligned_reference_reports_original_targets() -> Result<()> {
    let article = |first, second, target| {
        document(
            json!([paragraph(json!([citation(target)]))]),
            json!([
                reference(first, "Auxin signalling"),
                reference(second, "Root growth")
            ]),
        )
    };
    let comparison =
        symmetric_comparison(&article("c1", "c2", "c1")?, &article("b0", "b1", "b1")?)?;
    assert!(!comparison.alignment().has_one_sided());
    let changes = target_changes(&comparison);
    ensure!(changes.len() == 1, "{comparison:#?}");
    assert!(
        matches!(changes[0], Difference::ValueChanged { left, right, .. }
        if left == &ValueState::One { value: ScalarValue::string("c1") }
            && right == &ValueState::One { value: ScalarValue::string("b1") })
    );
    Ok(())
}

#[test]
fn reordered_references_resolve_through_pairs_not_positions() -> Result<()> {
    let left = document(
        json!([paragraph(json!([citation("c1")]))]),
        json!([
            reference("c1", "Auxin signalling"),
            reference("c2", "Root growth")
        ]),
    )?;
    let right = document(
        json!([paragraph(json!([citation("b1")]))]),
        json!([
            reference("b0", "Root growth"),
            reference("b1", "Auxin signalling")
        ]),
    )?;
    let comparison = symmetric_comparison(&left, &right)?;
    assert!(!comparison.alignment().has_one_sided());
    assert!(target_changes(&comparison).is_empty(), "{comparison:#?}");
    assert!(
        comparison
            .alignment()
            .pairs()
            .any(|(left, right, ..)| left.node_type == NodeType::Reference
                && left.path != right.path)
    );
    Ok(())
}

#[test]
fn the_same_link_label_can_resolve_to_different_nodes() -> Result<()> {
    let article = |swapped| {
        document(
            json!([
                paragraph(json!([link("#destination")])),
                paragraph(
                    json!([{"type": "Text", "value": "Auxin signalling", "id": if swapped {"other"} else {"destination"}}])
                ),
                paragraph(
                    json!([{"type": "Text", "value": "Root growth", "id": if swapped {"destination"} else {"other"}}])
                )
            ]),
            json!([]),
        )
    };
    let comparison = symmetric_comparison(&article(false)?, &article(true)?)?;
    assert_eq!(target_changes(&comparison).len(), 1, "{comparison:#?}");
    Ok(())
}

#[test]
fn unresolved_and_external_targets_remain_literal() -> Result<()> {
    for (left, right) in [
        (citation("missing1"), citation("missing2")),
        (
            citation("https://example.org/a"),
            citation("https://example.org/b"),
        ),
        (link("#missing1"), link("#missing2")),
        (
            link("https://example.org/a#section"),
            link("https://example.org/b#section"),
        ),
        (link("one.html#section"), link("two.html#section")),
    ] {
        let (left, right) = (
            serde_json::from_value(left)?,
            serde_json::from_value(right)?,
        );
        let comparison = symmetric_comparison(&left, &right)?;
        assert_eq!(target_changes(&comparison).len(), 1, "{comparison:#?}");
        assert!(compare(&left, &left)?.is_equal());
    }
    Ok(())
}

#[test]
fn missing_and_ambiguous_references_do_not_establish_equivalence() -> Result<()> {
    let article =
        |target, references| document(json!([paragraph(json!([citation(target)]))]), references);
    let left = article("c1", json!([reference("c1", "Auxin signalling")]))?;
    for right in [
        article("b0", json!([]))?,
        article(
            "b0",
            json!([
                reference("b0", "Auxin signalling"),
                reference("b0", "Root growth")
            ]),
        )?,
        article("b0", json!([reference("b0", "Unrelated research")]))?,
        // Even the same raw target differs if it resolves on only one side.
        article("c1", json!([]))?,
    ] {
        let comparison = symmetric_comparison(&left, &right)?;
        assert_eq!(target_changes(&comparison).len(), 1, "{comparison:#?}");
    }
    Ok(())
}

#[test]
fn rendering_a_citation_on_one_side_keeps_it_and_its_paragraph_paired() -> Result<()> {
    for (left_id, right_id) in [("reference-one", "reference-one"), ("c14", "b13")] {
        let article = |id, rendered: bool| {
            let mut citation = citation(id);
            if rendered {
                citation["content"] = json!([text("[1]")]);
            }
            document(
                json!([paragraph(json!([citation]))]),
                json!([reference(id, "Auxin signalling")]),
            )
        };
        let comparison =
            symmetric_comparison(&article(left_id, false)?, &article(right_id, true)?)?;
        let paired = |node_type| {
            comparison
                .alignment()
                .pairs()
                .any(|(left, ..)| left.node_type == node_type)
        };
        assert!(paired(NodeType::Paragraph), "{comparison:#?}");
        assert!(paired(NodeType::Citation), "{comparison:#?}");
        assert!(target_changes(&comparison).is_empty(), "{comparison:#?}");
    }
    Ok(())
}

#[test]
fn positional_reference_ids_do_not_pair_different_works() -> Result<()> {
    // Both sides number references by position, so dropping the second one shifts
    // `b1` onto a different work
    let left = document(
        json!([paragraph(json!([citation("b2")]))]),
        json!([
            reference("b0", "Auxin signalling in roots"),
            reference("b1", "Root growth under stress"),
            reference("b2", "Cell wall remodelling")
        ]),
    )?;
    let right = document(
        json!([paragraph(json!([citation("b1")]))]),
        json!([
            reference("b0", "Auxin signalling in roots"),
            reference("b1", "Cell wall remodelling")
        ]),
    )?;
    let comparison = symmetric_comparison(&left, &right)?;
    let left_only: Vec<_> = comparison
        .alignment()
        .correspondences()
        .iter()
        .filter_map(|correspondence| match correspondence {
            Correspondence::LeftOnly { left, .. } if left.node_type == NodeType::Reference => {
                Some(left.path.to_string())
            }
            _ => None,
        })
        .collect();
    assert_eq!(left_only, ["references/1"], "{comparison:#?}");
    assert!(target_changes(&comparison).is_empty(), "{comparison:#?}");
    Ok(())
}

#[test]
fn ids_within_resolved_citation_copies_are_not_ambiguous() -> Result<()> {
    let article = |author_id: &str| {
        let reference = json!({
            "type": "Reference", "id": "c1", "title": [text("Auxin signalling")],
            "authors": [{"type": "Person", "id": author_id, "givenNames": ["Ada"]}]
        });
        let mut citation = citation("c1");
        citation["cites"] = reference.clone();
        document(
            json!([paragraph(json!([citation, link(&format!("#{author_id}"))]))]),
            json!([reference]),
        )
    };
    let comparison = symmetric_comparison(&article("a1")?, &article("a2")?)?;
    assert!(comparison.is_equal(), "{comparison:#?}");
    Ok(())
}

#[test]
fn a_shared_target_does_not_override_text_similarity() -> Result<()> {
    let figure_link =
        |content: &str| json!({"type": "Link", "target": "#fig1", "content": [text(content)]});
    let left = document(
        json!([paragraph(json!([
            figure_link("Figure 1"),
            figure_link("the growth panel")
        ]))]),
        json!([]),
    )?;
    let right = document(
        json!([paragraph(json!([figure_link(
            "the growth panel (updated)"
        )]))]),
        json!([]),
    )?;
    let comparison = symmetric_comparison(&left, &right)?;
    let links: Vec<_> = comparison
        .alignment()
        .pairs()
        .filter(|(left, ..)| left.node_type == NodeType::Link)
        .map(|(left, right, ..)| (left.path.to_string(), right.path.to_string()))
        .collect();
    assert_eq!(
        links,
        [(
            "content/0/content/1".to_string(),
            "content/0/content/0".to_string()
        )],
        "{comparison:#?}"
    );
    Ok(())
}
