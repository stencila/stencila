use pretty_assertions::assert_eq;
use stencila_layout_lang::{Columns, Layout, LayoutError, Placement, parse};

fn placement(label: char, col: u32, row: u32, col_span: u32, row_span: u32) -> Placement {
    Placement {
        label,
        col,
        row,
        col_span,
        row_span,
    }
}

fn map(columns: Columns, placements: Vec<Placement>) -> Layout {
    Layout::Map {
        columns,
        placements,
    }
}

#[test]
fn parse_form3_vertical_span_standalone() {
    assert_eq!(
        parse("a,b|a,c"),
        Ok(map(
            Columns::equal(2),
            vec![
                placement('a', 0, 0, 1, 2),
                placement('b', 1, 0, 1, 1),
                placement('c', 1, 1, 1, 1),
            ]
        ))
    );
}

#[test]
fn parse_form3_horizontal_span_standalone() {
    assert_eq!(
        parse("a,a|b,c"),
        Ok(map(
            Columns::equal(2),
            vec![
                placement('a', 0, 0, 2, 1),
                placement('b', 0, 1, 1, 1),
                placement('c', 1, 1, 1, 1),
            ]
        ))
    );
}

#[test]
fn parse_form3_full_rectangle_span_standalone() {
    assert_eq!(
        parse("a,a|a,a"),
        Ok(map(Columns::equal(2), vec![placement('a', 0, 0, 2, 2)]))
    );
}

#[test]
fn parse_form3_empty_cell_standalone() {
    assert_eq!(
        parse("a,.|b,c"),
        Ok(map(
            Columns::equal(2),
            vec![
                placement('a', 0, 0, 1, 1),
                placement('b', 0, 1, 1, 1),
                placement('c', 1, 1, 1, 1),
            ]
        ))
    );
}

#[test]
fn parse_form4_with_explicit_widths_and_vertical_span() {
    assert_eq!(
        parse("30,70:a,b|a,c"),
        Ok(map(
            Columns {
                widths: vec![30, 70],
                gaps: vec![None],
            },
            vec![
                placement('a', 0, 0, 1, 2),
                placement('b', 1, 0, 1, 1),
                placement('c', 1, 1, 1, 1),
            ]
        ))
    );
}

#[test]
fn parse_form4_allows_whitespace_around_colon() {
    assert_eq!(
        parse("30,70 : a,b"),
        Ok(map(
            Columns {
                widths: vec![30, 70],
                gaps: vec![None],
            },
            vec![placement('a', 0, 0, 1, 1), placement('b', 1, 0, 1, 1)]
        ))
    );
}

#[test]
fn parse_form4_supports_explicit_gap_tokens() {
    assert_eq!(
        parse("40,g20,40:a,b|a,c"),
        Ok(map(
            Columns {
                widths: vec![40, 40],
                gaps: vec![Some(20)],
            },
            vec![
                placement('a', 0, 0, 1, 2),
                placement('b', 1, 0, 1, 1),
                placement('c', 1, 1, 1, 1),
            ]
        ))
    );
}

#[test]
fn parse_form4_supports_empty_cells() {
    assert_eq!(
        parse("30,70:a,.|b,c"),
        Ok(map(
            Columns {
                widths: vec![30, 70],
                gaps: vec![None],
            },
            vec![
                placement('a', 0, 0, 1, 1),
                placement('b', 0, 1, 1, 1),
                placement('c', 1, 1, 1, 1),
            ]
        ))
    );
}

#[test]
fn parse_form4_rejects_inconsistent_row_widths() {
    assert_eq!(
        parse("a,b|a"),
        Err(LayoutError::InconsistentRowWidth {
            expected: 2,
            actual: 1,
        })
    );
}

#[test]
fn parse_form4_rejects_non_rectangular_spans() {
    assert_eq!(
        parse("a,b|a,a"),
        Err(LayoutError::NonRectangularSpan { label: 'a' })
    );
}

#[test]
fn parse_form4_rejects_column_count_mismatches() {
    assert_eq!(
        parse("30,70,20:a,b|a,c"),
        Err(LayoutError::ColumnCountMismatch {
            expected: 3,
            actual: 2,
        })
    );
}

#[test]
fn parse_form4_rejects_integer_column_count_combined_forms() {
    assert!(matches!(
        parse("2:a,b"),
        Err(LayoutError::InvalidCombinedForm { form }) if form == "integer column count"
    ));

    assert!(matches!(
        parse("2 : a,b|a,c"),
        Err(LayoutError::InvalidCombinedForm { form }) if form == "integer column count"
    ));
}

#[test]
fn parse_form4_rejects_row_preset_combined_forms() {
    assert!(matches!(
        parse("row:a,b"),
        Err(LayoutError::InvalidCombinedForm { form }) if form == "row preset"
    ));
}

#[test]
fn parse_form4_rejects_malformed_map_syntax() {
    for input in ["a,,b", "|a,b", "a,b|", "a,b extra", "a,g20,b", "a,g20|b,c"] {
        let result = parse(input);
        assert!(
            matches!(result, Err(LayoutError::Parse(..))),
            "parse({input:?}) should return LayoutError::Parse, got: {result:?}"
        );
    }
}
