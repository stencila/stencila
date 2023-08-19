// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::list_item::ListItem;
use super::list_order::ListOrder;
use super::string::String;

/// A list of items.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, Strip, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct List {
    /// The type of this item
    pub r#type: MustBe!("List"),

    /// The identifier for this item
    pub id: Option<String>,

    /// The items in the list.
    pub items: Vec<ListItem>,

    /// The ordering of the list.
    pub order: ListOrder,
}
impl List {
    pub fn new(items: Vec<ListItem>, order: ListOrder) -> Self {
        Self {
            items,
            order,
            ..Default::default()
        }
    }
}
