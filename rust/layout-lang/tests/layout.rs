use pretty_assertions::assert_eq;
use stencila_layout_lang::{Columns, Layout, Placement};

#[test]
fn column_count_auto_returns_some() {
    let layout = Layout::Auto {
        columns: Columns::equal(2),
    };
    assert_eq!(layout.column_count(), Some(2));
}

#[test]
fn column_count_row_returns_none() {
    let layout = Layout::Row;
    assert_eq!(layout.column_count(), None);
}

#[test]
fn column_count_map_returns_count() {
    let layout = Layout::Map {
        columns: Columns::equal(3),
        placements: vec![],
    };
    assert_eq!(layout.column_count(), Some(3));
}

#[test]
fn resolve_row_produces_expected_map() {
    let resolved = Layout::Row.resolve_row(3);

    assert_eq!(
        resolved,
        Layout::Map {
            columns: Columns::equal(3),
            placements: vec![
                Placement {
                    label: 'a',
                    col: 0,
                    row: 0,
                    col_span: 1,
                    row_span: 1,
                },
                Placement {
                    label: 'b',
                    col: 1,
                    row: 0,
                    col_span: 1,
                    row_span: 1,
                },
                Placement {
                    label: 'c',
                    col: 2,
                    row: 0,
                    col_span: 1,
                    row_span: 1,
                },
            ],
        }
    );
}

#[test]
fn resolve_row_auto_passes_through() {
    let layout = Layout::Auto {
        columns: Columns::equal(3),
    };
    let resolved = layout.resolve_row(3);
    assert_eq!(
        resolved,
        Layout::Auto {
            columns: Columns::equal(3)
        }
    );
}

#[test]
fn resolve_row_map_passes_through() {
    let layout = Layout::Map {
        columns: Columns::equal(2),
        placements: vec![
            Placement {
                label: 'a',
                col: 0,
                row: 0,
                col_span: 1,
                row_span: 1,
            },
            Placement {
                label: 'b',
                col: 1,
                row: 0,
                col_span: 1,
                row_span: 1,
            },
        ],
    };
    let resolved = layout.clone().resolve_row(2);
    assert_eq!(resolved, layout);
}
