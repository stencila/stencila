//! Tests of the difference filter
//!
//! The filter is the one semantic option the crate accepts, so these tests pin both what
//! it selects and, just as importantly, what it must never touch: the alignment.

use eyre::{Result, bail};
use pretty_assertions::assert_eq;

use stencila_node_compare::{
    CompareOptions, Comparison, DifferenceFilter, Selector, align, compare, compare_with_options,
};
use stencila_node_type::{NodeProperty, NodeType};
use stencila_schema::{
    Block, Link, Node, Paragraph, Section,
    shortcuts::{art, p, t},
};

/// Compare two nodes under a filter given as exclusions and inclusions
fn filtered(left: &Node, right: &Node, exclude: &[&str], include: &[&str]) -> Result<Comparison> {
    let selectors = |texts: &[&str]| -> Result<Vec<Selector>> {
        texts
            .iter()
            .map(|text| text.parse::<Selector>().map_err(Into::into))
            .collect()
    };

    let options = CompareOptions {
        filter: DifferenceFilter {
            exclude: selectors(exclude)?,
            include: selectors(include)?,
        },
        ..Default::default()
    };

    Ok(compare_with_options(left, right, &options)?)
}

/// A paragraph carrying an explicit `id`
fn para(id: &str, text: &str) -> Block {
    Block::Paragraph(Paragraph {
        id: Some(id.to_string()),
        ..Paragraph::new(vec![t(text)])
    })
}

/// A paragraph holding one link, both carrying explicit `id`s
fn linked(paragraph_id: &str, link_id: &str, target: &str, text: &str) -> Block {
    Block::Paragraph(Paragraph {
        id: Some(paragraph_id.to_string()),
        ..Paragraph::new(vec![stencila_schema::Inline::Link(Link {
            id: Some(link_id.to_string()),
            target: target.to_string(),
            ..Link::new(vec![t(text)], target.to_string())
        })])
    })
}

/// The selector grammar covers each of its four forms, and round-trips
#[test]
fn selectors_parse_and_render() -> Result<()> {
    for (text, expected) in [
        ("*", Selector::All),
        ("Link", Selector::Type(NodeType::Link)),
        ("id", Selector::Property(NodeProperty::Id)),
        (
            "Link.id",
            Selector::TypeProperty(NodeType::Link, NodeProperty::Id),
        ),
    ] {
        let selector = text.parse::<Selector>()?;
        assert_eq!(selector, expected, "parsing `{text}`");
        assert_eq!(selector.to_string(), text, "rendering `{text}`");
    }

    // `all` is a synonym for `*`, which canonicalizes back to `*`
    assert_eq!("all".parse::<Selector>()?, Selector::All);
    assert_eq!(Selector::All.to_string(), "*");

    Ok(())
}

/// A selector that could never match anything is a mistake, not an empty filter
#[test]
fn impossible_selectors_are_rejected() {
    for text in [
        "",
        "Lnk",             // not a node type
        "jatsReftype",     // not a property; the casing is wrong
        "Link.href",       // not a property at all
        "Link.rowSpan",    // a property, but not one `Link` declares
        "link",            // node types are PascalCase, so this reads as a property
    ] {
        assert!(
            text.parse::<Selector>().is_err(),
            "`{text}` should not parse"
        );
    }
}

/// Excluding a property hides only differences about that property
#[test]
fn excluding_a_property_hides_only_that_property() -> Result<()> {
    // Same content on both sides, so both paragraphs pair and the only differences are
    // the identifiers
    let left = art([para("one", "Hello"), para("two", "World")]);
    let right = art([para("three", "Hello"), para("four", "World")]);

    let all = compare(&left, &right)?;
    let ids: Vec<_> = all
        .differences()
        .iter()
        .filter(|difference| difference.property() == Some(NodeProperty::Id))
        .collect();
    assert_eq!(ids.len(), 2, "both paragraphs differ in `id`");

    let filtered = filtered(&left, &right, &["id"], &[])?;
    assert_eq!(
        filtered.differences().len(),
        all.differences().len() - 2,
        "exactly the `id` differences are suppressed"
    );
    assert_eq!(filtered.suppressed_differences(), 2);
    assert!(
        filtered
            .differences()
            .iter()
            .all(|difference| difference.property() != Some(NodeProperty::Id)),
        "no `id` difference survives"
    );
    assert!(
        filtered.is_equal(),
        "identifiers aside, these two documents say the same thing"
    );
    assert!(
        !filtered.is_equal_unfiltered(),
        "but they are not equal once the filter is set aside"
    );

    Ok(())
}

/// A more specific selector beats a less specific one, whatever order they are given in
#[test]
fn the_most_specific_selector_wins_regardless_of_order() -> Result<()> {
    let left = art([linked("p1", "l1", "http://example.com", "here")]);
    let right = art([linked("p2", "l2", "http://example.com", "here")]);

    // Both the paragraph and the link differ in `id`
    let excluded = filtered(&left, &right, &["id"], &[])?;
    assert_eq!(excluded.suppressed_differences(), 2);

    // Re-including one type's `id` beats the bare property exclusion
    let reincluded = filtered(&left, &right, &["id"], &["Link.id"])?;
    assert_eq!(reincluded.suppressed_differences(), 1);
    assert_eq!(
        reincluded
            .differences()
            .iter()
            .filter(|difference| difference.property() == Some(NodeProperty::Id))
            .count(),
        1,
        "the link's `id` is reported again"
    );

    // Order within the selector lists is not significant either
    let one_way = filtered(&left, &right, &["id", "Paragraph.authors"], &["Link.id", "*"])?;
    let other_way = filtered(&left, &right, &["Paragraph.authors", "id"], &["*", "Link.id"])?;
    assert_eq!(one_way.differences(), other_way.differences());
    assert_eq!(
        one_way.differences(),
        reincluded.differences(),
        "and a redundant `*` inclusion loses to every more specific selector"
    );

    Ok(())
}

/// `*` makes the filter an allowlist
#[test]
fn everything_can_be_excluded_and_selectively_restored() -> Result<()> {
    // Identical but for the identifiers, so there is exactly one difference to restore
    let left = art([para("one", "Hello")]);
    let right = art([para("two", "Hello")]);

    let nothing = filtered(&left, &right, &["*"], &[])?;
    assert!(nothing.differences().is_empty(), "`*` excludes everything");
    assert!(
        nothing.is_equal(),
        "with every difference suppressed, the nodes are equal modulo the filter"
    );

    let only_ids = filtered(&left, &right, &["*"], &["id"])?;
    assert_eq!(only_ids.differences().len(), 1);
    assert_eq!(
        only_ids.differences()[0].property(),
        Some(NodeProperty::Id),
        "only the `id` difference is restored"
    );

    Ok(())
}

/// A filter never changes which occurrences pair with which
///
/// The whole safety argument for putting a semantic option on `CompareOptions` rests on
/// this: filtering is applied after alignment, so it cannot feed back into matching.
#[test]
fn filtering_never_changes_the_alignment() -> Result<()> {
    let left = art([
        linked("p1", "l1", "http://example.com", "here"),
        para("two", "World"),
    ]);
    let right = art([para("three", "World")]);

    let unfiltered = compare(&left, &right)?;
    let standalone = align(&left, &right)?;

    for (exclude, include) in [
        (vec!["id"], vec![]),
        (vec!["*"], vec![]),
        (vec!["Link"], vec!["Link.id"]),
        (vec!["id", "content"], vec!["Paragraph.id"]),
    ] {
        let comparison = filtered(&left, &right, &exclude, &include)?;
        assert_eq!(
            comparison.alignment(),
            unfiltered.alignment(),
            "the alignment is unchanged by {exclude:?}/{include:?}"
        );
        assert_eq!(
            comparison.alignment(),
            &standalone,
            "and still equals the standalone alignment"
        );
    }

    Ok(())
}

/// Excluding a node type hides its one-sided subtree whole, not just its root
#[test]
fn one_sided_subtrees_are_hidden_whole() -> Result<()> {
    // The link exists only on the left, so it and its inline text are both one-sided
    let left = art([linked("p1", "l1", "http://example.com", "here")]);
    let right = art([Block::Paragraph(Paragraph {
        id: Some("p1".to_string()),
        ..Paragraph::new(vec![])
    })]);

    let unfiltered = compare(&left, &right)?;
    let tally = unfiltered.one_sided_tally();
    assert!(
        tally.left_only() >= 2,
        "the link and its text are both one-sided, got {tally:?}"
    );
    assert!(!unfiltered.is_equal());

    let filtered = filtered(&left, &right, &["Link"], &[])?;
    let tally = filtered.one_sided_tally();
    assert_eq!(
        tally.left_only(),
        0,
        "the link's whole subtree is hidden, not just its root"
    );
    assert!(
        tally.suppressed_total() >= 2,
        "and the descendants are counted as suppressed"
    );
    assert!(
        filtered.is_equal(),
        "nothing the filter reports distinguishes the two"
    );

    // The alignment still records every one-sided occurrence
    assert!(filtered.alignment().has_one_sided());

    Ok(())
}

/// A filtered comparison says so, and cannot be mistaken for an exhaustive one
#[test]
fn a_filtered_comparison_is_self_describing() -> Result<()> {
    let left = art([para("one", "Hello")]);
    let right = art([para("two", "Hello")]);

    let unfiltered = compare(&left, &right)?;
    assert!(!unfiltered.is_filtered());
    assert!(!unfiltered.is_equal_unfiltered());

    let filtered = filtered(&left, &right, &["id"], &[])?;
    assert!(filtered.is_filtered());
    assert!(filtered.is_equal(), "equal, modulo the filter");
    assert!(
        !filtered.is_equal_unfiltered(),
        "but known to differ, because something was suppressed"
    );

    // A filter that suppressed nothing still answers the stricter question
    let untouched = filtered_equal()?;
    assert!(untouched.is_filtered());
    assert!(untouched.is_equal_unfiltered());

    Ok(())
}

/// Two identical nodes compared under a filter that suppresses nothing
fn filtered_equal() -> Result<Comparison> {
    let node = art([p([t("Hello")])]);
    filtered(&node, &node, &["jatsRefType"], &[])
}

/// The filter survives serialization, and a filtered artifact round-trips
#[test]
fn the_filter_round_trips_through_json() -> Result<()> {
    let left = art([para("one", "Hello")]);
    let right = art([para("two", "Goodbye")]);

    let comparison = filtered(&left, &right, &["id"], &["Paragraph.id"])?;
    let json = serde_json::to_string(&comparison)?;
    let restored: Comparison = serde_json::from_str(&json)?;

    assert_eq!(restored, comparison);
    assert_eq!(restored.filter(), comparison.filter());
    assert_eq!(
        restored.suppressed_differences(),
        comparison.suppressed_differences()
    );
    assert_eq!(restored.is_equal(), comparison.is_equal());

    // An unfiltered comparison serializes without any filter fields at all, so existing
    // artifacts and their readers are untouched
    let plain = compare(&left, &right)?;
    let json = serde_json::to_value(&plain)?;
    let Some(object) = json.as_object() else {
        bail!("Expected a JSON object");
    };
    assert!(!object.contains_key("filter"));
    assert!(!object.contains_key("suppressedDifferences"));

    Ok(())
}

/// A section is a different node type, so `Section` selectors do not touch paragraphs
#[test]
fn type_selectors_are_specific_to_their_type() -> Result<()> {
    let left = art([
        para("one", "Hello"),
        Block::Section(Section {
            id: Some("s1".to_string()),
            ..Section::new(vec![p([t("Inside")])])
        }),
    ]);
    let right = art([
        para("two", "Hello"),
        Block::Section(Section {
            id: Some("s2".to_string()),
            ..Section::new(vec![p([t("Inside")])])
        }),
    ]);

    let all = filtered(&left, &right, &[], &[])?;
    let paragraph_only = filtered(&left, &right, &["Section.id"], &[])?;
    let section_only = filtered(&left, &right, &["Paragraph.id"], &[])?;

    assert_eq!(paragraph_only.suppressed_differences(), 1);
    assert_eq!(section_only.suppressed_differences(), 1);
    assert_eq!(
        paragraph_only.differences().len() + 1,
        all.differences().len()
    );

    Ok(())
}
