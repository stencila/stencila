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

use eyre::Result;

use stencila_node_compare::compare;
use stencila_node_merge::{CommentMode, MergeOptions, MetadataChanges, merge_with_options};
use stencila_node_suggestions::{ResolveSuggestions, SuggestionAction};
use stencila_schema::{
    Block, Node, Section,
    shortcuts::{art, h1, h2, p, sec, t},
};

/// Merge with comments off, so that rejecting restores the left document exactly
fn options() -> MergeOptions {
    MergeOptions {
        comments: CommentMode::Omit,
        ..Default::default()
    }
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

    let (rejected, ..) = resolved(left, right, &SuggestionAction::RejectAll, options)?;
    assert!(
        same(&rejected, left)?,
        "rejecting did not yield the left document:\n{}",
        residue(&rejected, left)?
    );

    Ok(())
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
        id: Some("old-section".to_string()),
        ..Section::new(vec![p([t("Body")])])
    })]);
    let right = art([Block::Section(Section {
        id: Some("new-section".to_string()),
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
