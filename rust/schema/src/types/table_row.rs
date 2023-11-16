// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;
use super::table_cell::TableCell;
use super::table_row_type::TableRowType;

/// A row within a Table.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "TableRow")]
#[html(elem = "tr")]
pub struct TableRow {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("TableRow"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// An array of cells in the row.
    #[serde(alias = "cell")]
    #[serde(deserialize_with = "one_or_many")]
    #[cfg_attr(feature = "proptest-min", proptest(strategy = r#"vec(TableCell::arbitrary(), size_range(1..=1))"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"vec(TableCell::arbitrary(), size_range(2..=2))"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"vec(TableCell::arbitrary(), size_range(4..=4))"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"vec(TableCell::arbitrary(), size_range(1..=8))"#))]
    pub cells: Vec<TableCell>,

    /// The type of row.
    #[serde(alias = "row-type", alias = "row_type")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub row_type: Option<TableRowType>,
}

impl TableRow {
    pub fn new(cells: Vec<TableCell>) -> Self {
        Self {
            cells,
            ..Default::default()
        }
    }
}
