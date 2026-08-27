//! Every one-sided subtree and content change becomes a suggestion of the right kind,
//! in the right container
//!
//! The fixtures are ordinary Stencila-native structures, so a failure here is a change
//! in this crate rather than in any codec.

use eyre::{Result, bail};

use stencila_node_merge::merge;
use stencila_schema::{
    Block, Inline, Node, SuggestionType,
    shortcuts::{art, p, sec, t},
};

/// The blocks of an article
fn blocks(node: &Node) -> Result<&[Block]> {
    match node {
        Node::Article(article) => Ok(&article.content),
        _ => bail!("not an article"),
    }
}

/// The text of a node, with the block separators that `to_text` adds collapsed away
fn text<T: stencila_codec_text_trait::TextCodec>(node: &T) -> String {
    stencila_codec_text_trait::to_text(node)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The suggestion type and text of a block, when it is a suggestion
fn block_suggestion(block: &Block) -> Option<(SuggestionType, String)> {
    match block {
        Block::SuggestionBlock(suggestion) => {
            Some((suggestion.suggestion_type?, text(&suggestion.content)))
        }
        _ => None,
    }
}

#[test]
fn right_only_block_becomes_an_insertion() -> Result<()> {
    let left = art([p([t("One")])]);
    let right = art([p([t("One")]), p([t("Two")])]);

    let merged = merge(&left, &right)?;
    let content = blocks(merged.node())?;

    assert_eq!(content.len(), 2, "{content:#?}");
    assert_eq!(
        block_suggestion(&content[1]),
        Some((SuggestionType::Insert, "Two".to_string())),
        "{content:#?}"
    );
    assert_eq!(merged.report().inserts, 1);
    assert!(merged.report().is_complete(), "{:?}", merged.report());

    Ok(())
}

#[test]
fn left_only_block_becomes_a_deletion() -> Result<()> {
    let left = art([p([t("One")]), p([t("Two")])]);
    let right = art([p([t("One")])]);

    let merged = merge(&left, &right)?;
    let content = blocks(merged.node())?;

    assert_eq!(content.len(), 2, "{content:#?}");
    assert_eq!(
        block_suggestion(&content[1]),
        Some((SuggestionType::Delete, "Two".to_string())),
        "{content:#?}"
    );
    assert_eq!(merged.report().deletes, 1);

    Ok(())
}

#[test]
fn an_insertion_lands_where_the_right_document_put_it() -> Result<()> {
    let left = art([p([t("One")]), p([t("Three")])]);
    let right = art([p([t("One")]), p([t("Two")]), p([t("Three")])]);

    let merged = merge(&left, &right)?;
    let content = blocks(merged.node())?;

    assert_eq!(content.len(), 3, "{content:#?}");
    assert_eq!(
        block_suggestion(&content[1]),
        Some((SuggestionType::Insert, "Two".to_string())),
        "{content:#?}"
    );

    Ok(())
}

#[test]
fn an_insertion_before_everything_lands_first() -> Result<()> {
    let left = art([p([t("Second")])]);
    let right = art([p([t("First")]), p([t("Second")])]);

    let merged = merge(&left, &right)?;
    let content = blocks(merged.node())?;

    assert_eq!(
        block_suggestion(&content[0]),
        Some((SuggestionType::Insert, "First".to_string())),
        "{content:#?}"
    );

    Ok(())
}

#[test]
fn a_changed_text_becomes_an_inline_replacement() -> Result<()> {
    let left = art([p([t("Methods")])]);
    let right = art([p([t("Method")])]);

    let merged = merge(&left, &right)?;
    let content = blocks(merged.node())?;

    // The replacement wraps just the text, not the whole paragraph
    let Block::Paragraph(paragraph) = &content[0] else {
        bail!("expected the paragraph to survive: {content:#?}")
    };

    let Some(Inline::SuggestionInline(suggestion)) = paragraph.content.first() else {
        bail!("expected an inline suggestion: {paragraph:#?}")
    };

    assert_eq!(suggestion.suggestion_type, Some(SuggestionType::Replace));
    assert_eq!(text(&suggestion.content), "Method");
    assert_eq!(
        suggestion.original.as_ref().map(text),
        Some("Methods".to_string())
    );
    assert_eq!(merged.report().replaces, 1);

    Ok(())
}

#[test]
fn a_right_only_subtree_is_inserted_once_at_its_root() -> Result<()> {
    let left = art([p([t("One")])]);
    let right = art([p([t("One")]), sec([p([t("Alpha")]), p([t("Beta")])])]);

    let merged = merge(&left, &right)?;
    let content = blocks(merged.node())?;

    // The section and both its paragraphs are one-sided, but only the section is a
    // maximal root, so there is exactly one suggestion covering the whole subtree
    assert_eq!(content.len(), 2, "{content:#?}");
    assert_eq!(merged.report().inserts, 1, "{:?}", merged.report());
    assert_eq!(
        block_suggestion(&content[1]),
        Some((SuggestionType::Insert, "Alpha Beta".to_string())),
        "{content:#?}"
    );

    Ok(())
}

#[test]
fn only_the_changed_words_of_a_long_text_are_marked() -> Result<()> {
    // The case that prompted this: a long text differing in two characters was marked
    // in its entirety, because the difference is reported against the `Text` node and
    // the whole paragraph's prose lives in one of those
    let long = "We introduce a new approach to modelling decision confidence, with the \
                aim of enabling computationally cheap predictions while taking into \
                account trial-by-trial variability in dynamic stimuli.";
    let left = art([p([t(long)])]);
    let right = art([p([t(long.replace("cheap", "inexpensive"))])]);

    let merged = merge(&left, &right)?;
    let content = blocks(merged.node())?;

    let Block::Paragraph(paragraph) = &content[0] else {
        bail!("expected the paragraph to survive: {content:#?}")
    };

    // The unchanged prose is left as plain text either side of one marked run
    let suggestions: Vec<_> = paragraph
        .content
        .iter()
        .filter(|inline| matches!(inline, Inline::SuggestionInline(..)))
        .collect();
    assert_eq!(suggestions.len(), 1, "{:#?}", paragraph.content);

    let Some(Inline::SuggestionInline(suggestion)) = suggestions.first().copied() else {
        bail!("expected an inline suggestion")
    };
    assert_eq!(text(&suggestion.content), "inexpensive");
    assert_eq!(
        suggestion.original.as_ref().map(text),
        Some("cheap".to_string())
    );

    // And the paragraph still reads as the left document did, apart from that run
    assert!(
        paragraph.content.len() >= 3,
        "the text was not split around the change: {:#?}",
        paragraph.content
    );

    Ok(())
}
