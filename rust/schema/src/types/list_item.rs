//! Generated file, do not edit

use crate::prelude::*;

use super::block::Block;
use super::blocks_or_inlines::BlocksOrInlines;
use super::boolean::Boolean;
use super::image_object_or_string::ImageObjectOrString;
use super::integer::Integer;
use super::node::Node;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;

/// A single item in a list.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct ListItem {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    pub r#type: MustBe!("ListItem"),

    /// The identifier for this item
    #[key]
    pub id: Option<String>,

    /// The content of the list item.
    pub content: Option<BlocksOrInlines>,

    /// The item represented by this list item.
    pub item: Option<Box<Node>>,

    /// A flag to indicate if this list item is checked.
    pub is_checked: Option<Boolean>,

    /// The position of the item in a series or sequence of items.
    pub position: Option<Integer>,

    /// Non-core optional fields
    #[serde(flatten)]
    pub options: Box<ListItemOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct ListItemOptions {
    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    pub description: Option<Vec<Block>>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueOrString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectOrString>>,

    /// The name of the item.
    pub name: Option<String>,

    /// The URL of the item.
    pub url: Option<String>,
}
