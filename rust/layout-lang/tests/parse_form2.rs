use pretty_assertions::assert_eq;
use stencila_layout_lang::{Columns, Layout, LayoutError, parse};

fn auto(widths: Vec<u32>, gaps: Vec<Option<u32>>) -> Layout {
    Layout::Auto {
        columns: Columns { widths, gaps },
    }
}

#[test]
fn parse_form2_two_explicit_widths() {
    assert_eq!(parse("30 70"), Ok(auto(vec![30, 70], vec![None])));
}

#[test]
fn parse_form2_two_widths_with_explicit_gap() {
    assert_eq!(parse("40 g20 40"), Ok(auto(vec![40, 40], vec![Some(20)])));
}

#[test]
fn parse_form2_three_widths_with_two_explicit_gaps() {
    assert_eq!(
        parse("20 g10 30 g10 40"),
        Ok(auto(vec![20, 30, 40], vec![Some(10), Some(10)]))
    );
}

#[test]
fn parse_form2_allows_small_integer_widths() {
    assert_eq!(parse("1 2"), Ok(auto(vec![1, 2], vec![None])));
}

#[test]
fn parse_form2_allows_equal_explicit_widths() {
    assert_eq!(parse("1 1"), Ok(auto(vec![1, 1], vec![None])));
}

#[test]
fn parse_form2_allows_extra_whitespace_between_tokens() {
    assert_eq!(parse("30   70"), Ok(auto(vec![30, 70], vec![None])));
}

// --- Validation errors ---

#[test]
fn parse_form2_rejects_zero_width_columns() {
    assert!(matches!(parse("0 70"), Err(LayoutError::ZeroColumn { .. })));
}

#[test]
fn parse_form2_rejects_zero_width_gaps() {
    assert!(matches!(
        parse("30 g0 70"),
        Err(LayoutError::ZeroGap { .. })
    ));
}

#[test]
fn parse_form2_rejects_leading_gap_tokens() {
    assert!(matches!(
        parse("g20 40"),
        Err(LayoutError::InvalidGapPosition { .. })
    ));
}

#[test]
fn parse_form2_rejects_trailing_gap_tokens() {
    assert!(matches!(
        parse("40 g20"),
        Err(LayoutError::InvalidGapPosition { .. })
    ));
}

#[test]
fn parse_form2_rejects_adjacent_gap_tokens() {
    assert!(matches!(
        parse("40 g20 g10 40"),
        Err(LayoutError::InvalidGapPosition { .. })
    ));
}
