//! A merge respects the filter carried by its comparison

use eyre::Result;

use stencila_node_compare::{
    CompareOptions, DifferenceFilter, Selector, compare, compare_with_options,
};
use stencila_node_merge::merge_comparison;
use stencila_schema::shortcuts::{art, p, t};

#[test]
fn an_excluded_one_sided_subtree_is_not_merged() -> Result<()> {
    let left = art([p([t("One")])]);
    let right = art([p([t("One")]), p([t("Two")])]);
    let comparison = compare_with_options(
        &left,
        &right,
        &CompareOptions {
            filter: DifferenceFilter {
                exclude: vec!["Paragraph".parse::<Selector>()?],
                ..Default::default()
            },
            ..Default::default()
        },
    )?;

    assert!(
        comparison.is_equal(),
        "the visible comparison should be equal"
    );
    let merged = merge_comparison(&left, &right, &comparison)?;
    assert_eq!(merged.report().suggestions(), 0, "{:?}", merged.report());
    assert!(compare(merged.node(), &left)?.is_equal_unfiltered());

    Ok(())
}
