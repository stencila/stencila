//! Differences that no suggestion can carry become comments, anchored to what they
//! are about
//!
//! A comment is attached to the occurrence it describes by identifier, and the
//! identifier is assigned before the content is rewritten. That order is the thing
//! most likely to go wrong: after the rewrite the same path reaches the suggestion
//! wrapper, so a comment assigned afterwards would describe the change rather than
//! what changed.

use eyre::{Result, bail};

use stencila_node_merge::{
    CommentMode, MergeOptions, MetadataChanges, UnrepresentableReason, merge_with_options,
};
use stencila_schema::{
    Block, Heading, Node,
    shortcuts::{art, h1, h2, li, p, t, ul},
};

/// The comments of an article
fn comments(node: &Node) -> Result<Vec<(String, Option<String>)>> {
    let Node::Article(article) = node else {
        bail!("not an article")
    };

    Ok(article
        .options
        .comments
        .iter()
        .flatten()
        .map(|comment| {
            (
                stencila_codec_text_trait::to_text(&comment.content)
                    .trim()
                    .to_string(),
                comment.options.start_location.clone(),
            )
        })
        .collect())
}

/// Options that carry a metadata change as well as describing it
fn and_suggest() -> MergeOptions {
    MergeOptions {
        metadata_changes: MetadataChanges::CommentAndSuggest,
        ..Default::default()
    }
}

#[test]
fn a_changed_heading_level_is_commented() -> Result<()> {
    let left = art([h1([t("Title")])]);
    let right = art([h2([t("Title")])]);

    let merged = merge_with_options(&left, &right, &MergeOptions::default())?;

    // The comment says what changed, in the document's own terms rather than Rust's
    let comments = comments(merged.node())?;
    assert_eq!(comments.len(), 1, "{comments:?}");
    assert!(
        comments[0].0.contains("`level` changed from `1` to `2`"),
        "{comments:?}"
    );

    // By default nothing is marked: the heading reads the same on both sides, so a
    // replacement would show its text twice to report a change that is not in the text
    assert_eq!(merged.report().suggestions(), 0, "{:?}", merged.report());

    Ok(())
}

#[test]
fn a_changed_heading_level_can_also_be_carried_by_a_suggestion() -> Result<()> {
    let left = art([h1([t("Title")])]);
    let right = art([h2([t("Title")])]);

    let merged = merge_with_options(&left, &right, &and_suggest())?;

    assert_eq!(comments(merged.node())?.len(), 1);
    // Which is what makes accepting the merge reproduce the right document
    assert_eq!(merged.report().replaces, 1, "{:?}", merged.report());

    Ok(())
}

#[test]
fn a_comment_is_anchored_to_what_changed_not_to_the_suggestion() -> Result<()> {
    let left = art([h1([t("Title")])]);
    let right = art([h2([t("Title")])]);

    let merged = merge_with_options(&left, &right, &and_suggest())?;

    let Some((.., location)) = comments(merged.node())?.into_iter().next() else {
        bail!("expected one comment")
    };
    let Some(location) = location else {
        bail!("the comment should be anchored")
    };
    let Some(id) = location.strip_prefix('#') else {
        bail!("expected an identifier reference")
    };

    // The heading is inside the replacement's `original`, and it is the heading that
    // carries the identifier, not the `SuggestionBlock` that wraps it
    let Node::Article(article) = merged.node() else {
        bail!("not an article")
    };
    let Some(Block::SuggestionBlock(suggestion)) = article.content.first() else {
        bail!("expected a suggestion: {:#?}", article.content)
    };
    assert_ne!(
        suggestion.id.as_deref(),
        Some(id),
        "anchored to the wrapper"
    );

    let Some(Block::Heading(heading)) = suggestion.original.as_ref().and_then(|old| old.first())
    else {
        bail!("expected the heading in `original`: {suggestion:#?}")
    };
    assert_eq!(heading.id.as_deref(), Some(id));

    Ok(())
}

#[test]
fn comments_can_be_turned_off() -> Result<()> {
    let left = art([h1([t("Title")])]);
    let right = art([h2([t("Title")])]);

    let merged = merge_with_options(
        &left,
        &right,
        &MergeOptions {
            comments: CommentMode::Omit,
            ..and_suggest()
        },
    )?;

    assert!(comments(merged.node())?.is_empty());
    // The suggestion that carries the change is unaffected
    assert_eq!(merged.report().replaces, 1);

    Ok(())
}

#[test]
fn comment_only_drops_the_suggestion_but_keeps_the_comment() -> Result<()> {
    let left = art([h1([t("Title")])]);
    let right = art([h2([t("Title")])]);

    let merged = merge_with_options(
        &left,
        &right,
        &MergeOptions {
            metadata_changes: MetadataChanges::CommentOnly,
            ..Default::default()
        },
    )?;

    assert_eq!(comments(merged.node())?.len(), 1);
    assert_eq!(merged.report().suggestions(), 0, "{:?}", merged.report());
    assert!(!merged.report().is_complete());

    Ok(())
}

#[test]
fn a_container_that_cannot_hold_a_suggestion_is_reported() -> Result<()> {
    // `List.items` holds `ListItem`s, and a suggestion is not one, so a right-only
    // list item cannot be wrapped
    let left = art([ul([li([t("One")])])]);
    let right = art([ul([li([t("One")]), li([t("Two")])])]);

    let merged = merge_with_options(&left, &right, &MergeOptions::default())?;

    let reasons: Vec<_> = merged
        .report()
        .unrepresentable
        .iter()
        .map(|entry| entry.reason.clone())
        .collect();

    assert!(
        reasons.iter().any(|reason| matches!(
            reason,
            UnrepresentableReason::NotContentContainer { slot } if slot == "ListItem"
        )),
        "{reasons:?}"
    );

    Ok(())
}

#[test]
fn the_message_names_the_property_that_changed() -> Result<()> {
    let left = art([p([t("Text")])]);
    let right = art([h1([t("Text")])]);

    let merged = merge_with_options(&left, &right, &MergeOptions::default())?;

    let messages: Vec<String> = comments(merged.node())?
        .into_iter()
        .map(|(message, ..)| message)
        .collect();

    assert!(
        messages
            .iter()
            .any(|message| message.contains("Node type changed from `Paragraph` to `Heading`")),
        "{messages:?}"
    );

    Ok(())
}

#[test]
fn comments_reuse_an_existing_target_id() -> Result<()> {
    let left = art([Block::Heading(Heading {
        id: Some("stable-heading".to_string()),
        ..Heading::new(1, vec![t("Title")])
    })]);
    let right = art([Block::Heading(Heading {
        id: Some("revised-heading".to_string()),
        ..Heading::new(2, vec![t("Title")])
    })]);

    let merged = merge_with_options(&left, &right, &MergeOptions::default())?;
    let Node::Article(article) = merged.node() else {
        bail!("not an article")
    };
    let Some(Block::Heading(heading)) = article.content.first() else {
        bail!("expected a heading")
    };
    assert_eq!(heading.id.as_deref(), Some("stable-heading"));

    let Some(comments) = &article.options.comments else {
        bail!("expected comments")
    };
    assert!(comments.len() >= 2, "{comments:#?}");
    for comment in comments {
        assert_eq!(
            comment.options.start_location.as_deref(),
            Some("#stable-heading")
        );
        assert_ne!(comment.id.as_deref(), Some("stable-heading"));
    }

    Ok(())
}
