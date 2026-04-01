use pretty_assertions::assert_eq;
use stencila_layout_lang::{Columns, Layout, parse};

#[test]
fn parse_row_keyword() {
    assert_eq!(parse("row"), Ok(Layout::Row));
}

#[test]
fn parse_row_with_surrounding_whitespace() {
    assert_eq!(parse("  row  "), Ok(Layout::Row));
}

#[test]
fn parse_empty_string_returns_empty_input_error() {
    assert_eq!(
        parse(""),
        Err(stencila_layout_lang::LayoutError::EmptyInput),
        "parse(\"\") should return Err(LayoutError::EmptyInput)"
    );
}

#[test]
fn parse_whitespace_only_returns_empty_input_error() {
    assert_eq!(
        parse("   "),
        Err(stencila_layout_lang::LayoutError::EmptyInput),
        "parse(\"   \") should return Err(LayoutError::EmptyInput)"
    );
}

#[test]
fn parse_integer_with_trailing_junk_returns_parse_error() {
    let result = parse("2 junk");
    assert!(
        result.is_err(),
        "parse(\"2 junk\") should fail, got: {result:?}"
    );
    let err = result.expect_err("already checked");
    assert!(
        matches!(err, stencila_layout_lang::LayoutError::Parse(..)),
        "parse(\"2 junk\") should return LayoutError::Parse, got: {err:?}"
    );
}

#[test]
fn parse_row_with_trailing_content_returns_parse_error() {
    let result = parse("row extra");
    assert!(
        result.is_err(),
        "parse(\"row extra\") should fail, got: {result:?}"
    );
    let err = result.expect_err("already checked");
    assert!(
        matches!(err, stencila_layout_lang::LayoutError::Parse(..)),
        "parse(\"row extra\") should return LayoutError::Parse, got: {err:?}"
    );
}

#[test]
fn parse_form1_integer_two_columns() {
    assert_eq!(
        parse("2"),
        Ok(Layout::Auto {
            columns: Columns::equal(2)
        })
    );
}

#[test]
fn parse_form1_integer_one_column() {
    assert_eq!(
        parse("1"),
        Ok(Layout::Auto {
            columns: Columns::equal(1)
        })
    );
}

#[test]
fn parse_form1_integer_three_columns() {
    assert_eq!(
        parse("3"),
        Ok(Layout::Auto {
            columns: Columns::equal(3)
        })
    );
}

#[test]
fn parse_integer_with_surrounding_whitespace() {
    assert_eq!(
        parse("  2  "),
        Ok(Layout::Auto {
            columns: Columns::equal(2)
        })
    );
}
