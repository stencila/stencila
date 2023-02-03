//! Generated file, do not edit

use crate::prelude::*;

use super::list_item::ListItem;
use super::list_order::ListOrder;
use super::string::String;

/// A list of items.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct List {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    pub r#type: MustBe!("List"),

    /// The identifier for this item
    #[key]
    pub id: Option<String>,

    /// The items in the list.
    pub items: Vec<ListItem>,

    /// The ordering of the list.
    pub order: ListOrder,
}
