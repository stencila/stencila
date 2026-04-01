use pretty_assertions::assert_eq;
use stencila_layout_lang::Columns;

#[test]
fn equal_produces_equal_widths_and_none_gaps() {
    let cols = Columns::equal(3);
    assert_eq!(cols.widths, vec![1, 1, 1]);
    assert_eq!(cols.gaps, vec![None, None]);
}

#[test]
fn equal_single_column() {
    let cols = Columns::equal(1);
    assert_eq!(cols.widths, vec![1]);
    assert!(cols.gaps.is_empty());
}

#[test]
fn equal_many_columns() {
    let cols = Columns::equal(5);
    assert_eq!(cols.widths, vec![1, 1, 1, 1, 1]);
    assert_eq!(cols.gaps, vec![None, None, None, None]);
}

#[test]
fn column_count_returns_number_of_columns() {
    let cols = Columns::equal(4);
    assert_eq!(cols.column_count(), 4);
}

#[test]
fn column_count_single() {
    let cols = Columns::equal(1);
    assert_eq!(cols.column_count(), 1);
}
