use eyre::Result;
use serde_json::json;

use stencila_node_compare::{
    AlignmentFormatVersion, CompareOptions, Comparison, Correspondence, Difference,
    DifferenceFilter, compare, compare_with_options,
};
use stencila_schema::{
    Node,
    shortcuts::{art, em, h1, p, sec, t},
};

fn pair() -> (Node, Node) {
    (
        art([
            p([t("First paragraph."), t(" More text.")]),
            p([t("Second paragraph.")]),
        ]),
        art([p([t("First paragraph. More text. Second paragraph.")])]),
    )
}

#[test]
fn a_merge_and_its_inverse_are_single_complete_differences() -> Result<()> {
    let (left, right) = pair();
    let merged = compare(&left, &right)?;
    merged.validate(&left, &right)?;
    assert_eq!(
        merged.alignment().format_version(),
        AlignmentFormatVersion::V2
    );
    assert!(!merged.alignment().has_one_sided(), "{merged:#?}");
    assert_eq!(merged.differences().len(), 1, "{merged:#?}");
    assert!(
        matches!(&merged.differences()[0], Difference::ParagraphBoundariesChanged { left, right }
        if left.count() == 2 && right.count() == 1)
    );
    assert!(!merged.is_equal());
    assert_eq!(
        merged.alignment().correspondences().len(),
        2,
        "root plus group; descendants covered collectively"
    );
    let split = compare(&right, &left)?;
    split.validate(&right, &left)?;
    assert_eq!(merged.clone().invert(), split);
    let restored: Comparison = serde_json::from_str(&serde_json::to_string(&merged)?)?;
    assert_eq!(restored, merged);
    restored.validate(&left, &right)?;
    Ok(())
}

#[test]
fn rich_content_is_preserved_and_real_changes_remain_visible() -> Result<()> {
    let left = art([
        p([t("With "), em([t("emphasis")]), t(".")]),
        p([t("Next paragraph.")]),
    ]);
    let right = art([p([t("With "), em([t("emphasis")]), t(". Next paragraph.")])]);
    let exact = compare(&left, &right)?;
    assert_eq!(exact.differences().len(), 1);
    assert!(!exact.alignment().has_one_sided(), "{exact:#?}");
    exact.validate(&left, &right)?;

    for changed in [
        art([p([t("With emphasis. Next paragraph.")])]),
        art([p([
            t("With "),
            em([t("emphasis")]),
            t(". Different paragraph."),
        ])]),
    ] {
        let comparison = compare(&left, &changed)?;
        assert!(
            comparison
                .alignment()
                .correspondences()
                .iter()
                .all(|item| !matches!(item, Correspondence::ParagraphGroup { .. }))
        );
        assert!(!comparison.is_equal());
    }
    Ok(())
}

#[test]
fn changed_citations_and_paragraph_metadata_are_not_hidden() -> Result<()> {
    let para = |content| json!({"type": "Paragraph", "content": content});
    let text = |value| json!({"type": "Text", "value": value});
    let link = |target| json!({"type": "Link", "content": [text("source")], "target": target});
    let left: Node = serde_json::from_value(json!({"type":"Article", "content": [
        para(vec![text("See "), link("https://example.org/a")]), para(vec![text("Next.")])
    ]}))?;
    let right: Node = serde_json::from_value(json!({"type":"Article", "content": [
        para(vec![text("See "), link("https://example.org/b"), text(" Next.")])
    ]}))?;
    assert!(
        !compare(&left, &right)?
            .differences()
            .iter()
            .any(|d| matches!(d, Difference::ParagraphBoundariesChanged { .. }))
    );

    let (left, right) = pair();
    let mut value = serde_json::to_value(left)?;
    value["content"][0]["id"] = json!("addressable-paragraph");
    let left = serde_json::from_value(value)?;
    assert!(
        !compare(&left, &right)?
            .differences()
            .iter()
            .any(|d| matches!(d, Difference::ParagraphBoundariesChanged { .. }))
    );
    Ok(())
}

#[test]
fn full_content_is_checked_beyond_the_matching_prefix() -> Result<()> {
    let prefix = "Long shared prefix. ".repeat(100);
    let left = art([p([t(&prefix)]), p([t("A distinct ending.")])]);
    let right = art([p([t(format!("{prefix} A different ending."))])]);
    let comparison = compare(&left, &right)?;
    assert!(
        !comparison
            .differences()
            .iter()
            .any(|d| matches!(d, Difference::ParagraphBoundariesChanged { .. }))
    );
    assert!(!comparison.is_equal());
    Ok(())
}

#[test]
fn identical_target_strings_must_resolve_to_corresponding_nodes() -> Result<()> {
    let text = |value| json!({"type":"Text", "value": value});
    let heading =
        |id, value| json!({"type":"Heading", "id":id, "level":1, "content":[text(value)]});
    let link = json!({"type":"Link", "target":"#a", "content":[text("See above")]});
    let left: Node = serde_json::from_value(json!({"type":"Article", "content":[
        heading("a", "Apples and orchards"),
        {"type":"Paragraph", "content":[text("The apple findings are described in detail here. "), link.clone()]},
        {"type":"Paragraph", "content":[text("More details.")]},
        heading("tail", "Zebras and savannahs")
    ]}))?;
    let right: Node = serde_json::from_value(json!({"type":"Article", "content":[
        heading("b", "Apples and orchards"),
        {"type":"Paragraph", "content":[text("The apple findings are described in detail here. "), link, text(" More details.")]},
        heading("tail", "Zebras and savannahs")
    ]}))?;
    let comparison = compare(&left, &right)?;
    assert!(
        !comparison
            .differences()
            .iter()
            .any(|difference| matches!(difference, Difference::ParagraphBoundariesChanged { .. }))
    );
    assert!(comparison.differences().iter().any(|difference| matches!(difference,
        Difference::ValueChanged { location, .. } if location.property == Some(stencila_node_type::NodeProperty::Target))));
    Ok(())
}

#[test]
fn groups_stay_within_consecutive_sibling_paragraphs() -> Result<()> {
    let right = art([p([t("First. Second.")])]);
    for left in [
        art([p([t("First.")]), h1([t("Heading")]), p([t("Second.")])]),
        art([sec([p([t("First.")])]), sec([p([t("Second.")])])]),
        art([p([t("Second.")]), p([t("First.")])]),
        art([p([t("")]), p([t("First. Second.")])]),
    ] {
        let comparison = compare(&left, &right)?;
        comparison.validate(&left, &right)?;
        assert!(
            !comparison
                .differences()
                .iter()
                .any(|d| matches!(d, Difference::ParagraphBoundariesChanged { .. })),
            "{comparison:#?}"
        );
    }
    Ok(())
}

#[test]
fn multiple_groups_leave_anchors_and_other_edits_intact() -> Result<()> {
    let left = art([
        h1([t("Start")]),
        p([t("One.")]),
        p([t("Two.")]),
        h1([t("Middle")]),
        p([t("Three.")]),
        p([t("Four.")]),
        p([t("Five.")]),
    ]);
    let right = art([
        h1([t("Start")]),
        p([t("One. Two.")]),
        h1([t("Middle")]),
        p([t("Three. Four. Five.")]),
    ]);
    let comparison = compare(&left, &right)?;
    comparison.validate(&left, &right)?;
    assert_eq!(comparison.differences().len(), 2, "{comparison:#?}");
    assert!(!comparison.alignment().has_one_sided());
    assert_eq!(comparison.clone().invert(), compare(&right, &left)?);
    Ok(())
}

#[test]
fn filtering_changes_reporting_without_changing_group_coverage() -> Result<()> {
    let (left, right) = pair();
    let full = compare(&left, &right)?;
    let filtered = compare_with_options(
        &left,
        &right,
        &CompareOptions {
            filter: DifferenceFilter {
                exclude: vec!["Paragraph.content".parse()?],
                include: vec![],
            },
            ..Default::default()
        },
    )?;
    assert_eq!(full.alignment(), filtered.alignment());
    assert!(filtered.is_equal());
    assert!(!filtered.is_equal_unfiltered());
    assert_eq!(filtered.suppressed_differences(), 1);
    assert_eq!(filtered.one_sided_tally().reported, [0, 0]);
    filtered.validate(&left, &right)?;
    let mut invalid = serde_json::to_value(&filtered)?;
    invalid["suppressedDifferences"] = json!(0);
    assert!(serde_json::from_value::<Comparison>(invalid).is_err());
    Ok(())
}

#[test]
fn malformed_groups_and_duplicate_descendant_coverage_are_rejected() -> Result<()> {
    let (left, right) = pair();
    let comparison = compare(&left, &right)?;
    let original = serde_json::to_value(comparison)?;
    let group_index = original["alignment"]["correspondences"]
        .as_array()
        .and_then(|items| {
            items
                .iter()
                .position(|item| item["type"] == "paragraphGroup")
        })
        .ok_or_else(|| eyre::eyre!("missing paragraph group"))?;
    for replacement in [
        json!(["content", 0]),
        json!(["content", 2]),
        json!(["abstract", 1]),
    ] {
        let mut value = original.clone();
        value["alignment"]["correspondences"][group_index]["left"]["rest"][0]["path"] = replacement;
        assert!(serde_json::from_value::<Comparison>(value).is_err());
    }
    let mut value = original.clone();
    value["alignment"]["formatVersion"] = json!("1");
    assert!(serde_json::from_value::<Comparison>(value).is_err());
    let mut value = original.clone();
    value["differences"] = json!([]);
    assert!(serde_json::from_value::<Comparison>(value).is_err());
    let mut value = original;
    let records = value["alignment"]["correspondences"]
        .as_array_mut()
        .ok_or_else(|| eyre::eyre!("missing correspondences"))?;
    records.push(json!({"type":"leftOnly", "left":{"path":["content",0,"content",0],"nodeType":"Text"},"reason":"noCompatibleCandidate","nearestOneSidedAncestor":null}));
    assert!(serde_json::from_value::<Comparison>(value).is_err());
    Ok(())
}

#[test]
fn groups_are_recognized_symmetrically_beside_reordered_siblings() -> Result<()> {
    let left = art([
        p([t("Alpha one.")]),
        p([t("Beta two.")]),
        h1([t("First")]),
        h1([t("Second")]),
        h1([t("Third")]),
    ]);
    let right = art([
        p([t("Alpha one. Beta two.")]),
        h1([t("First")]),
        h1([t("Third")]),
        h1([t("Second")]),
    ]);
    let forward = compare(&left, &right)?;
    let backward = compare(&right, &left)?;
    for comparison in [&forward, &backward] {
        assert!(
            comparison
                .differences()
                .iter()
                .any(|d| matches!(d, Difference::ParagraphBoundariesChanged { .. })),
            "{comparison:#?}"
        );
        assert!(!comparison.alignment().has_one_sided(), "{comparison:#?}");
    }
    assert_eq!(forward.invert(), backward);
    Ok(())
}

#[test]
fn references_resolve_to_identified_inlines_within_groups() -> Result<()> {
    let text = |value| json!({"type":"Text", "value": value});
    let para = |content| json!({"type":"Paragraph", "content": content});
    let marked = json!({"type":"Link", "id":"x", "target":"https://example.org", "content":[text("example")]});
    let heading = json!({"type":"Heading", "level":1, "content":[
        {"type":"Link", "target":"#x", "content":[text("back")]}
    ]});
    let intro = "A much longer introduction about apples and orchards to ensure this paragraph matches first.";
    let spaced = format!("{intro} ");
    let left: Node = serde_json::from_value(json!({"type":"Article", "content":[
        para(vec![text(&spaced), json!({"type":"Emphasis", "content":[text("See "), marked.clone()]})]),
        heading.clone()
    ]}))?;
    let right: Node = serde_json::from_value(json!({"type":"Article", "content":[
        para(vec![text(intro)]),
        para(vec![json!({"type":"Emphasis", "content":[text("S"), text("ee "), marked]})]),
        heading
    ]}))?;
    for (left, right) in [(&left, &right), (&right, &left)] {
        let comparison = compare(left, right)?;
        comparison.validate(left, right)?;
        assert_eq!(comparison.differences().len(), 1, "{comparison:#?}");
        assert!(
            matches!(
                comparison.differences()[0],
                Difference::ParagraphBoundariesChanged { .. }
            ),
            "{comparison:#?}"
        );
    }
    Ok(())
}
