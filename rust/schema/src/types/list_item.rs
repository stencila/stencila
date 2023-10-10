// Generated file; do not edit. See `schema-gen` crate.

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
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[html(elem = "li", custom)]
#[jats(elem = "list-item")]
#[markdown(special)]
pub struct ListItem {
    /// The type of this item
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("ListItem"),

    /// The identifier for this item
    #[strip(id)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The content of the list item.
    #[cfg_attr(feature = "proptest-min", proptest(strategy = r#"vec_paragraphs(1).prop_map(|blocks| Some(BlocksOrInlines::Blocks(blocks)))"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"vec_blocks_list_item(1).prop_map(|blocks| Some(BlocksOrInlines::Blocks(blocks)))"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"vec_blocks_list_item(2).prop_map(|blocks| Some(BlocksOrInlines::Blocks(blocks)))"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"vec_blocks_list_item(4).prop_map(|blocks| Some(BlocksOrInlines::Blocks(blocks)))"#))]
    pub content: Option<BlocksOrInlines>,

    /// The item represented by this list item.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub item: Option<Box<Node>>,

    /// A flag to indicate if this list item is checked.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub is_checked: Option<Boolean>,

    /// The position of the item in a series or sequence of items.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub position: Option<Integer>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    #[markdown(flatten)]
    pub options: Box<ListItemOptions>,
}

#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub struct ListItemOptions {
    /// Alternate names (aliases) for the item.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    #[strip(types)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub description: Option<Vec<Block>>,

    /// Any kind of identifier for any kind of Thing.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub identifiers: Option<Vec<PropertyValueOrString>>,

    /// Images of the item.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub images: Option<Vec<ImageObjectOrString>>,

    /// The name of the item.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub name: Option<String>,

    /// The URL of the item.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub url: Option<String>,
}

impl ListItem {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
