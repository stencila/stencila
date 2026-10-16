//! Accepting every suggestion yields the right document; rejecting every suggestion
//! yields the left one
//!
//! This is the property that makes a merged document a faithful representation of a
//! comparison rather than an approximation of one, so it is pinned here both where it
//! holds and where it does not. `MergeReport::is_complete` is the machine-checkable
//! statement of "it holds for this merge", and every test that expects the round-trip
//! asserts it rather than assuming it.
//!
//! Equality is decided by comparing, not by `PartialEq`: independently built trees get
//! unrelated node uids, and `is_equal_unfiltered` is exactly "no differences, none
//! suppressed, and nothing one-sided".
//!
//! It is equality *modulo adjacent text nodes being one node*. Marking only the runs of
//! a text that differ splits the `Text` that held it into several, and accepting the
//! suggestions between them leaves those pieces adjacent rather than rejoined. The text
//! of the document is exactly the right document's, and every format encodes it
//! identically; only the node boundaries differ. `coalesce_text` below rejoins them so
//! that the comparison is about content rather than about where a mark happened to fall.

use std::collections::BTreeSet;

use eyre::Result;
use serde_json::{Value, json};

use stencila_node_compare::compare;
use stencila_node_merge::{CommentMode, MergeOptions, MetadataChanges, merge_with_options};
use stencila_node_suggestions::{ResolveSuggestions, SuggestionAction};
use stencila_schema::{
    Block, Node, Section, SectionType,
    shortcuts::{art, h1, h2, p, sec, t},
};

/// Merge with comments off, so that rejecting restores the left document exactly
fn options() -> MergeOptions {
    MergeOptions {
        comments: CommentMode::Omit,
        ..Default::default()
    }
}

#[test]
fn cosmetic_changes_still_round_trip() -> Result<()> {
    use stencila_node_compare::{CompareOptions, Normalization, compare_with_options};
    use stencila_node_merge::merge_comparison_with_options;

    let left = art([p([t("Auxin’s role")])]);
    let right = art([p([t("Auxin's  role")])]);
    let comparison = compare_with_options(
        &left,
        &right,
        &CompareOptions {
            normalize: [Normalization::Whitespace, Normalization::Typography].into(),
            ..Default::default()
        },
    )?;
    assert!(
        comparison
            .differences()
            .iter()
            .all(|difference| difference.is_cosmetic())
    );
    let merged = merge_comparison_with_options(&left, &right, &comparison, &options())?;
    assert!(merged.report().is_complete());
    let mut accepted = merged.node().clone();
    accepted.resolve_suggestions(&SuggestionAction::AcceptAll);
    assert!(same(&accepted, &right)?);
    let mut rejected = merged.into_node();
    rejected.resolve_suggestions(&SuggestionAction::RejectAll);
    assert!(same(&rejected, &left)?);
    Ok(())
}

/// Whether two nodes are the same document, up to adjacent text nodes being one node
fn same(one: &Node, other: &Node) -> Result<bool> {
    Ok(compare(&coalesce_text(one)?, &coalesce_text(other)?)?.is_equal_unfiltered())
}

/// Rejoin adjacent `Text` nodes throughout a document
///
/// Done over the serialized form so that it applies to every container the schema has,
/// rather than to the handful this file happens to exercise.
fn coalesce_text(node: &Node) -> Result<Node> {
    let mut value = serde_json::to_value(node)?;
    rejoin(&mut value);
    Ok(serde_json::from_value(value)?)
}

/// Rejoin adjacent text within every array of a JSON tree
fn rejoin(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Array(items) => {
            for item in items.iter_mut() {
                rejoin(item);
            }

            let mut joined: Vec<serde_json::Value> = Vec::with_capacity(items.len());
            for item in items.drain(..) {
                let addition = text_value(&item);
                let previous = joined.last().and_then(text_value);

                if let (Some(previous), Some(addition)) = (previous, addition) {
                    if let Some(string) = joined
                        .last_mut()
                        .and_then(|last| last.pointer_mut("/value/string"))
                    {
                        *string = serde_json::Value::String(previous + &addition);
                    }
                    continue;
                }

                joined.push(item);
            }
            *items = joined;
        }
        serde_json::Value::Object(entries) => {
            for (.., entry) in entries.iter_mut() {
                rejoin(entry);
            }
        }
        _ => {}
    }
}

/// The string of a JSON value that is a `Text` node
///
/// `Text.value` is a `Cord`, which serializes as an object carrying the string and its
/// authorship rather than as a bare string.
fn text_value(value: &serde_json::Value) -> Option<String> {
    (value.get("type")?.as_str()? == "Text")
        .then(|| {
            value
                .pointer("/value/string")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
        .flatten()
}

/// Merge, then resolve every suggestion one way
fn resolved(
    left: &Node,
    right: &Node,
    action: &SuggestionAction,
    options: &MergeOptions,
) -> Result<(Node, bool)> {
    let merged = merge_with_options(left, right, options)?;
    let complete = merged.report().is_complete();
    let mut node = merged.into_node();
    node.resolve_suggestions(action);
    Ok((node, complete))
}

/// Assert the round-trip in both directions
fn round_trips(left: &Node, right: &Node) -> Result<()> {
    round_trips_with_options(left, right, &options())
}

/// Assert the round-trip in both directions with explicit merge options
fn round_trips_with_options(left: &Node, right: &Node, options: &MergeOptions) -> Result<()> {
    let (accepted, complete) = resolved(left, right, &SuggestionAction::AcceptAll, options)?;
    assert!(complete, "the merge left something unrepresentable");
    assert!(
        same(&accepted, right)?,
        "accepting did not yield the right document:\n{}",
        residue(&accepted, right)?
    );

    // Comparison resolves targets through the alignment, so equality alone cannot
    // tell a translated target from one that resolves to nothing
    let dangling = dangling_targets(&accepted)?;
    assert!(
        dangling.is_subset(&dangling_targets(right)?),
        "accepting left targets that resolve to nothing: {dangling:?}"
    );

    let (rejected, ..) = resolved(left, right, &SuggestionAction::RejectAll, options)?;
    assert!(
        same(&rejected, left)?,
        "rejecting did not yield the left document:\n{}",
        residue(&rejected, left)?
    );
    let dangling = dangling_targets(&rejected)?;
    assert!(
        dangling.is_subset(&dangling_targets(left)?),
        "rejecting left targets that resolve to nothing: {dangling:?}"
    );

    Ok(())
}

/// The citation and fragment link targets that no ID in a document resolves
///
/// IDs within a citation's cached copy of a reference are not addressable, as in
/// comparison.
fn dangling_targets(node: &Node) -> Result<BTreeSet<String>> {
    fn walk(value: &Value, ids: &mut BTreeSet<String>, targets: &mut BTreeSet<String>) {
        match value {
            Value::Array(items) => {
                for item in items {
                    walk(item, ids, targets);
                }
            }
            Value::Object(entries) => {
                if let Some(id) = entries.get("id").and_then(Value::as_str) {
                    ids.insert(id.to_string());
                }
                let target = entries.get("target").and_then(Value::as_str);
                match (entries.get("type").and_then(Value::as_str), target) {
                    (Some("Citation"), Some(target)) => {
                        targets.insert(target.trim().to_string());
                    }
                    (Some("Link"), Some(target)) => {
                        if let Some(fragment) = target.strip_prefix('#') {
                            targets.insert(fragment.to_string());
                        }
                    }
                    _ => {}
                }
                for (key, entry) in entries {
                    if key != "cites" {
                        walk(entry, ids, targets);
                    }
                }
            }
            _ => {}
        }
    }

    let (mut ids, mut targets) = (BTreeSet::new(), BTreeSet::new());
    walk(&serde_json::to_value(node)?, &mut ids, &mut targets);
    Ok(targets.difference(&ids).cloned().collect())
}

/// What still differs between two documents, for a failure message
///
/// The differences themselves, not the documents: a dump of two articles says nothing
/// about which part of them failed to line up.
fn residue(one: &Node, other: &Node) -> Result<String> {
    let comparison = compare(&coalesce_text(one)?, &coalesce_text(other)?)?;

    let mut residue = String::new();
    for difference in comparison.differences() {
        residue.push_str(&format!("{difference:?}\n\n"));
    }
    for correspondence in comparison.alignment().correspondences() {
        if !matches!(
            correspondence,
            stencila_node_compare::Correspondence::Paired { .. }
        ) {
            residue.push_str(&format!("{correspondence:?}\n\n"));
        }
    }

    Ok(residue)
}

#[test]
fn an_inserted_block_round_trips() -> Result<()> {
    round_trips(&art([p([t("One")])]), &art([p([t("One")]), p([t("Two")])]))
}

#[test]
fn a_deleted_block_round_trips() -> Result<()> {
    round_trips(&art([p([t("One")]), p([t("Two")])]), &art([p([t("One")])]))
}

#[test]
fn a_changed_text_round_trips() -> Result<()> {
    round_trips(&art([p([t("Methods")])]), &art([p([t("Method")])]))
}

#[test]
fn a_rewritten_document_round_trips() -> Result<()> {
    round_trips(
        &art([
            h1([t("Introduction")]),
            p([t("The first paragraph")]),
            p([t("The second paragraph")]),
            p([t("The third paragraph")]),
        ]),
        &art([
            h1([t("Introduction")]),
            p([t("The first paragraph")]),
            p([t("An entirely new paragraph")]),
            p([t("The third paragraph")]),
            p([t("And one appended at the end")]),
        ]),
    )
}

#[test]
fn a_nested_change_round_trips() -> Result<()> {
    round_trips(
        &art([sec([p([t("Alpha")]), p([t("Beta")])])]),
        &art([sec([p([t("Alpha")]), p([t("Gamma")]), p([t("Beta")])])]),
    )
}

#[test]
fn metadata_and_nested_content_changes_round_trip_together() -> Result<()> {
    let left = art([h1([t("Old title")])]);
    let right = art([h2([t("New title")])]);
    let options = MergeOptions {
        comments: CommentMode::Omit,
        metadata_changes: MetadataChanges::CommentAndSuggest,
        ..Default::default()
    };

    round_trips_with_options(&left, &right, &options)
}

#[test]
fn metadata_on_a_block_container_can_be_suggested() -> Result<()> {
    let left = art([Block::Section(Section {
        section_type: Some(SectionType::Introduction),
        ..Section::new(vec![p([t("Body")])])
    })]);
    let right = art([Block::Section(Section {
        section_type: Some(SectionType::Discussion),
        ..Section::new(vec![p([t("Body")])])
    })]);
    let options = MergeOptions {
        comments: CommentMode::Omit,
        metadata_changes: MetadataChanges::CommentAndSuggest,
        ..Default::default()
    };

    round_trips_with_options(&left, &right, &options)
}

#[test]
fn merging_a_document_with_itself_changes_nothing() -> Result<()> {
    let document = art([h1([t("Title")]), p([t("Body")])]);

    let merged = merge_with_options(&document, &document, &options())?;
    assert_eq!(merged.report().suggestions(), 0, "{:?}", merged.report());
    assert!(same(merged.node(), &document)?);

    Ok(())
}

#[test]
fn a_reorder_is_reported_as_unrepresentable() -> Result<()> {
    // Both occurrences are paired, so there is no one-sided content to insert or
    // delete and no suggestion can carry the change of position
    let left = art([p([t("Alpha")]), p([t("Beta")])]);
    let right = art([p([t("Beta")]), p([t("Alpha")])]);

    let merged = merge_with_options(&left, &right, &MergeOptions::default())?;
    assert!(
        !merged.report().is_complete(),
        "a reorder should be reported as unrepresentable: {:?}",
        merged.report()
    );

    Ok(())
}

#[test]
fn a_document_with_many_kinds_of_change_round_trips() -> Result<()> {
    round_trips(
        &art([
            h1([t("A study")]),
            p([t("Background sentence one.")]),
            p([t("Background sentence two.")]),
            sec([
                h1([t("Methods")]),
                p([t("We did the first thing.")]),
                p([t("We did the second thing.")]),
            ]),
            sec([h1([t("Results")]), p([t("Everything worked.")])]),
        ]),
        &art([
            h1([t("A study")]),
            // one edited in place
            p([t("Background sentence one, revised.")]),
            // one deleted, one inserted after it
            p([t("An inserted background sentence.")]),
            sec([
                h1([t("Methods")]),
                p([t("We did the first thing.")]),
                // one deleted from a nested container
                p([t("We did a third thing.")]),
            ]),
            // a whole section inserted
            sec([h1([t("Discussion")]), p([t("It means something.")])]),
            sec([h1([t("Results")]), p([t("Everything worked.")])]),
        ]),
    )
}

/// Build a node from its JSON form
fn node(value: Value) -> Result<Node> {
    Ok(serde_json::from_value(value)?)
}

/// A string property of every node of a type, in document order
fn strings(node: &Node, node_type: &str, property: &str) -> Result<Vec<String>> {
    fn walk(value: &Value, node_type: &str, property: &str, found: &mut Vec<String>) {
        match value {
            Value::Array(items) => {
                for item in items {
                    walk(item, node_type, property, found);
                }
            }
            Value::Object(entries) => {
                if entries.get("type").and_then(Value::as_str) == Some(node_type)
                    && let Some(value) = entries.get(property).and_then(Value::as_str)
                {
                    found.push(value.to_string());
                }
                for entry in entries.values() {
                    walk(entry, node_type, property, found);
                }
            }
            _ => {}
        }
    }

    let mut found = Vec::new();
    walk(
        &serde_json::to_value(node)?,
        node_type,
        property,
        &mut found,
    );
    Ok(found)
}

/// Accept every suggestion
fn accepted(left: &Node, right: &Node, options: &MergeOptions) -> Result<Node> {
    Ok(resolved(left, right, &SuggestionAction::AcceptAll, options)?.0)
}

fn section(id: Option<&str>, text: &str) -> Value {
    let mut section = json!({"type": "Section", "content": [
        {"type": "Paragraph", "content": [{"type": "Text", "value": text}]}
    ]});
    if let Some(id) = id {
        section["id"] = json!(id);
    }
    section
}

fn link_paragraph(text: &str, target: &str) -> Value {
    json!({"type": "Paragraph", "content": [
        {"type": "Text", "value": text},
        {"type": "Link", "target": target, "content": [{"type": "Text", "value": "here"}]}
    ]})
}

/// A link added beside a renamed section points at the name the left document keeps
#[test]
fn inserted_links_follow_the_retained_ids() -> Result<()> {
    let left = node(json!({"type": "Article", "content": [section(Some("old"), "Results")]}))?;
    let right = node(json!({"type": "Article", "content": [
        section(Some("new"), "Results"),
        link_paragraph("See the results ", "#new")
    ]}))?;
    round_trips(&left, &right)?;

    let accepted = accepted(&left, &right, &options())?;
    assert_eq!(strings(&accepted, "Section", "id")?, ["old"]);
    assert_eq!(strings(&accepted, "Link", "target")?, ["#old"]);
    Ok(())
}

/// A citation changed to cite a different work names that work as the left does
#[test]
fn changed_citations_follow_the_retained_reference_ids() -> Result<()> {
    let article = |first: &str, second: &str, target: &str| {
        node(json!({
            "type": "Article",
            "content": [{"type": "Paragraph", "content": [
                {"type": "Text", "value": "As shown "},
                {"type": "Citation", "target": target}
            ]}],
            "references": [
                {"type": "Reference", "id": first, "title": [{"type": "Text", "value": "Auxin signalling"}]},
                {"type": "Reference", "id": second, "title": [{"type": "Text", "value": "Root growth"}]}
            ]
        }))
    };
    let left = article("c1", "c2", "c1")?;
    let right = article("b0", "b1", "b1")?;
    let options = MergeOptions {
        comments: CommentMode::Omit,
        metadata_changes: MetadataChanges::CommentAndSuggest,
        ..Default::default()
    };
    round_trips_with_options(&left, &right, &options)?;

    assert_eq!(
        strings(&accepted(&left, &right, &options)?, "Citation", "target")?,
        ["c2"]
    );
    Ok(())
}

/// A reference ID shared with a node of another type still resolves for citations, as
/// it does in comparison
#[test]
fn citations_resolve_references_whose_ids_other_nodes_share() -> Result<()> {
    let article = |reference: &str, paragraph: Option<&str>, target: &str| {
        let mut cited = json!({"type": "Paragraph", "content": [
            {"type": "Text", "value": "As shown "},
            {"type": "Citation", "target": target}
        ]});
        if let Some(id) = paragraph {
            cited["id"] = json!(id);
        }
        node(json!({
            "type": "Article",
            "content": [cited],
            "references": [
                {"type": "Reference", "id": reference, "title": [{"type": "Text", "value": "Auxin signalling"}]}
            ]
        }))
    };
    let left = article("c1", None, "c0")?;
    let right = article("b1", Some("b1"), "b1")?;
    let options = MergeOptions {
        comments: CommentMode::Omit,
        metadata_changes: MetadataChanges::CommentAndSuggest,
        ..Default::default()
    };
    round_trips_with_options(&left, &right, &options)?;

    assert_eq!(
        strings(&accepted(&left, &right, &options)?, "Citation", "target")?,
        ["c1"]
    );
    Ok(())
}

/// New content keeps its own IDs, renamed where the left document already uses them
#[test]
fn inserted_ids_do_not_collide_with_retained_ones() -> Result<()> {
    // The right renames the section `results` to `findings`, and gives `results` to
    // new inline text, which cannot be the section's counterpart
    let left = node(json!({"type": "Article", "content": [section(Some("results"), "Results")]}))?;
    let right = node(json!({"type": "Article", "content": [
        section(Some("findings"), "Results"),
        {"type": "Paragraph", "content": [
            {"type": "Text", "id": "results", "value": "A new remark. "},
            {"type": "Link", "target": "#results", "content": [{"type": "Text", "value": "This"}]},
            {"type": "Link", "target": "#findings", "content": [{"type": "Text", "value": "That"}]}
        ]}
    ]}))?;
    round_trips(&left, &right)?;

    let accepted = accepted(&left, &right, &options())?;
    assert_eq!(strings(&accepted, "Section", "id")?, ["results"]);
    assert_eq!(strings(&accepted, "Text", "id")?, ["results-2"]);
    assert_eq!(
        strings(&accepted, "Link", "target")?,
        ["#results-2", "#results"]
    );
    Ok(())
}

/// A left node that new content refers to is given the ID the reference needs
#[test]
fn referenced_left_nodes_are_identified() -> Result<()> {
    let left = node(json!({"type": "Article", "content": [section(None, "Results")]}))?;
    let right = node(json!({"type": "Article", "content": [
        section(Some("results"), "Results"),
        link_paragraph("See the results ", "#results")
    ]}))?;
    round_trips(&left, &right)?;

    let accepted = accepted(&left, &right, &options())?;
    assert_eq!(strings(&accepted, "Section", "id")?, ["results"]);
    assert_eq!(strings(&accepted, "Link", "target")?, ["#results"]);
    Ok(())
}

/// The original of an unrepresentable move keeps its ID, so the copy needs another
#[test]
fn moved_nodes_do_not_duplicate_ids() -> Result<()> {
    let moved = json!({"type": "Paragraph", "id": "old", "content": [
        {"type": "Text", "value": "Auxin gradients orient the growth of the root tip"}
    ]});
    let left = node(json!({"type": "Article", "content": [
        {"type": "Paragraph", "content": [{"type": "Text", "value": "Introduction"}]},
        moved.clone()
    ]}))?;
    let right = node(json!({"type": "Article", "content": [
        {"type": "Paragraph", "content": [{"type": "Text", "value": "Introduction"}]},
        {"type": "Section", "content": [
            {"type": "Heading", "level": 1, "content": [{"type": "Text", "value": "Results"}]},
            moved,
            link_paragraph("See ", "#old")
        ]}
    ]}))?;

    let merged = merge_with_options(&left, &right, &options())?;
    let merged = serde_json::to_value(merged.node())?;
    // The original, kept in place, and the copy, inside the inserted section
    assert_eq!(merged.pointer("/content/2/id"), Some(&json!("old")));
    assert_eq!(
        merged.pointer("/content/1/content/0/content/1/id"),
        Some(&json!("old-2"))
    );
    // A copied reference names the counterpart, as comparison resolves it
    assert_eq!(
        merged.pointer("/content/1/content/0/content/2/content/1/target"),
        Some(&json!("#old"))
    );
    Ok(())
}

/// A kept left node given an ID for a reference does not share it with a copy of itself
#[test]
fn referenced_left_nodes_do_not_share_ids_with_their_copies() -> Result<()> {
    let moved = |id: Option<&str>| {
        let mut paragraph = json!({"type": "Paragraph", "content": [
            {"type": "Text", "value": "Auxin gradients orient the growth of the root tip"}
        ]});
        if let Some(id) = id {
            paragraph["id"] = json!(id);
        }
        paragraph
    };
    let left = node(json!({"type": "Article", "content": [
        {"type": "Paragraph", "content": [{"type": "Text", "value": "Introduction"}]},
        moved(None)
    ]}))?;
    let right = node(json!({"type": "Article", "content": [
        {"type": "Paragraph", "content": [{"type": "Text", "value": "Introduction"}]},
        {"type": "Section", "content": [
            {"type": "Heading", "level": 1, "content": [{"type": "Text", "value": "Results"}]},
            moved(Some("tip")),
            link_paragraph("See ", "#tip")
        ]}
    ]}))?;

    let merged = merge_with_options(&left, &right, &options())?;
    let merged = serde_json::to_value(merged.node())?;
    assert_eq!(merged.pointer("/content/2/id"), Some(&json!("tip")));
    assert_eq!(
        merged.pointer("/content/1/content/0/content/1/id"),
        Some(&json!("tip-2"))
    );
    assert_eq!(
        merged.pointer("/content/1/content/0/content/2/content/1/target"),
        Some(&json!("#tip"))
    );
    Ok(())
}

/// A text replacement does not keep the original's ID, so a moved copy of that text
/// takes it over, and a copied reference to it still resolves once accepted
#[test]
fn moved_text_with_replaced_text_keeps_its_id() -> Result<()> {
    let paragraph = |value: &str| {
        json!({"type": "Paragraph", "content": [
            {"type": "Text", "id": "old", "value": value}
        ]})
    };
    let left = node(json!({"type": "Article", "content": [
        {"type": "Paragraph", "content": [{"type": "Text", "value": "Introduction"}]},
        paragraph("Auxin gradients orient the growth of the root tip")
    ]}))?;
    let right = node(json!({"type": "Article", "content": [
        {"type": "Paragraph", "content": [{"type": "Text", "value": "Introduction"}]},
        {"type": "Section", "content": [
            {"type": "Heading", "level": 1, "content": [{"type": "Text", "value": "Results"}]},
            paragraph("Auxin gradients orient the growth of the root tips"),
            link_paragraph("See ", "#old")
        ]}
    ]}))?;

    let accepted = accepted(&left, &right, &options())?;
    assert_eq!(strings(&accepted, "Text", "id")?, ["old"]);
    assert_eq!(strings(&accepted, "Link", "target")?, ["#old"]);
    Ok(())
}
