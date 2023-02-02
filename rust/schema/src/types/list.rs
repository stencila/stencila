//! Generated file, do not edit

use crate::prelude::*;

use super::list_item::ListItem;
use super::list_order::ListOrder;
use super::string::String;

/// A list of items.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct List {
    /// The type of this item
    r#type: MustBe!("List"),

    /// The identifier for this item
    id: String,

    /// The items in the list.
    items: Vec<ListItem>,

    /// The ordering of the list.
    order: ListOrder,
}
