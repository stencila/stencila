use pretty_assertions::assert_eq;
use stencila_layout_lang::{Columns, Layout, Placement, parse};

fn placement(label: char, col: u32, row: u32, col_span: u32, row_span: u32) -> Placement {
    Placement {
        label,
        col,
        row,
        col_span,
        row_span,
    }
}

fn map(columns: usize, placements: Vec<Placement>) -> Layout {
    Layout::Map {
        columns: Columns::equal(columns),
        placements,
    }
}

#[test]
fn parse_form3_single_row_two_cells() {
    assert_eq!(
        parse("a b"),
        Ok(map(
            2,
            vec![placement('a', 0, 0, 1, 1), placement('b', 1, 0, 1, 1)]
        ))
    );
}

#[test]
fn parse_form3_single_row_three_cells() {
    assert_eq!(
        parse("a b c"),
        Ok(map(
            3,
            vec![
                placement('a', 0, 0, 1, 1),
                placement('b', 1, 0, 1, 1),
                placement('c', 2, 0, 1, 1),
            ]
        ))
    );
}

#[test]
fn parse_form3_two_rows_without_spans() {
    assert_eq!(
        parse("a b|c d"),
        Ok(map(
            2,
            vec![
                placement('a', 0, 0, 1, 1),
                placement('b', 1, 0, 1, 1),
                placement('c', 0, 1, 1, 1),
                placement('d', 1, 1, 1, 1),
            ]
        ))
    );
}

#[test]
fn parse_form3_allows_whitespace_around_pipe() {
    assert_eq!(
        parse("a b | c d"),
        Ok(map(
            2,
            vec![
                placement('a', 0, 0, 1, 1),
                placement('b', 1, 0, 1, 1),
                placement('c', 0, 1, 1, 1),
                placement('d', 1, 1, 1, 1),
            ]
        ))
    );
}

#[test]
fn parse_form3_derives_vertical_span_in_alphabetical_order() {
    assert_eq!(
        parse("a b|a c"),
        Ok(map(
            2,
            vec![
                placement('a', 0, 0, 1, 2),
                placement('b', 1, 0, 1, 1),
                placement('c', 1, 1, 1, 1),
            ]
        ))
    );
}

#[test]
fn parse_form3_derives_horizontal_span() {
    assert_eq!(
        parse("a a|b c"),
        Ok(map(
            2,
            vec![
                placement('a', 0, 0, 2, 1),
                placement('b', 0, 1, 1, 1),
                placement('c', 1, 1, 1, 1),
            ]
        ))
    );
}

#[test]
fn parse_form3_derives_full_rectangle_span() {
    assert_eq!(
        parse("a a|a a"),
        Ok(map(2, vec![placement('a', 0, 0, 2, 2)]))
    );
}

#[test]
fn parse_form3_vertical_span_later_letter_sorts_alphabetically() {
    // Regression: labels must be sorted alphabetically, not by first-appearance
    // in grid scan order. In "a d|b d|c d", 'd' appears before 'b' and 'c'
    // during left-to-right top-to-bottom scanning, but placements must be
    // ordered [a, b, c, d] so the nth subfigure maps to the nth letter.
    assert_eq!(
        parse("a d|b d|c d"),
        Ok(map(
            2,
            vec![
                placement('a', 0, 0, 1, 1),
                placement('b', 0, 1, 1, 1),
                placement('c', 0, 2, 1, 1),
                placement('d', 1, 0, 1, 3),
            ]
        ))
    );
}

#[test]
fn parse_form3_ignores_empty_cells_when_building_placements() {
    assert_eq!(
        parse("a .|b c"),
        Ok(map(
            2,
            vec![
                placement('a', 0, 0, 1, 1),
                placement('b', 0, 1, 1, 1),
                placement('c', 1, 1, 1, 1),
            ]
        ))
    );
}
