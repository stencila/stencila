use stencila_layout_lang::LayoutError;

#[test]
fn display_parse_includes_detail() {
    let err = LayoutError::Parse("unexpected token at position 5".into());
    let msg = err.to_string();
    assert!(
        msg.contains("unexpected token at position 5"),
        "Parse display should include the detail string, got: {msg}"
    );
}

#[test]
fn display_empty_input() {
    let err = LayoutError::EmptyInput;
    let msg = err.to_string();
    let lower = msg.to_lowercase();
    assert!(
        lower.contains("empty"),
        "EmptyInput display should mention 'empty', got: {msg}"
    );
}

#[test]
fn display_zero_column_includes_position() {
    let err = LayoutError::ZeroColumn { position: 3 };
    let msg = err.to_string();
    let lower = msg.to_lowercase();
    assert!(
        lower.contains("position") && msg.contains('3'),
        "ZeroColumn display should mention 'position' and include value 3, got: {msg}"
    );
}

#[test]
fn display_zero_gap_includes_position() {
    let err = LayoutError::ZeroGap { position: 2 };
    let msg = err.to_string();
    let lower = msg.to_lowercase();
    assert!(
        lower.contains("position") && msg.contains('2'),
        "ZeroGap display should mention 'position' and include value 2, got: {msg}"
    );
}

#[test]
fn display_invalid_gap_position_includes_position() {
    let err = LayoutError::InvalidGapPosition { position: 7 };
    let msg = err.to_string();
    let lower = msg.to_lowercase();
    assert!(
        lower.contains("position") && msg.contains('7'),
        "InvalidGapPosition display should mention 'position' and include value 7, got: {msg}"
    );
}

#[test]
fn display_inconsistent_row_width_includes_expected_and_actual() {
    let err = LayoutError::InconsistentRowWidth {
        expected: 4,
        actual: 3,
    };
    let msg = err.to_string();
    let lower = msg.to_lowercase();
    assert!(
        lower.contains("expected") && msg.contains('4'),
        "InconsistentRowWidth display should mention 'expected' and include value 4, got: {msg}"
    );
    assert!(
        msg.contains('3'),
        "InconsistentRowWidth display should include actual value 3, got: {msg}"
    );
}

#[test]
fn display_non_rectangular_span_includes_label() {
    let err = LayoutError::NonRectangularSpan { label: 'x' };
    let msg = err.to_string();
    assert!(
        msg.contains('x'),
        "NonRectangularSpan display should include label character 'x', got: {msg}"
    );
}

#[test]
fn display_column_count_mismatch_includes_expected_and_actual() {
    let err = LayoutError::ColumnCountMismatch {
        expected: 5,
        actual: 2,
    };
    let msg = err.to_string();
    let lower = msg.to_lowercase();
    assert!(
        lower.contains("expected") && msg.contains('5'),
        "ColumnCountMismatch display should mention 'expected' and include value 5, got: {msg}"
    );
    assert!(
        msg.contains('2'),
        "ColumnCountMismatch display should include actual value 2, got: {msg}"
    );
}

#[test]
fn display_invalid_combined_form_includes_form() {
    let err = LayoutError::InvalidCombinedForm {
        form: "row + map".into(),
    };
    let msg = err.to_string();
    assert!(
        msg.contains("row + map"),
        "InvalidCombinedForm display should include form name, got: {msg}"
    );
}
