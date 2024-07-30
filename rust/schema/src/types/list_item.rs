// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::boolean::Boolean;
use super::cord::Cord;
use super::image_object::ImageObject;
use super::integer::Integer;
use super::node::Node;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;

/// A single item in a list.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "ListItem")]
#[html(elem = "li")]
#[jats(elem = "list-item")]
pub struct ListItem {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("ListItem"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The content of the list item.
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[patch(format = "all")]
    #[cfg_attr(feature = "proptest-min", proptest(strategy = r#"vec_paragraphs(1)"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"vec_blocks_list_item(1)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"vec_blocks_list_item(2)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"vec_blocks_list_item(4)"#))]
    #[dom(elem = "li")]
    pub content: Vec<Block>,

    /// The item represented by this list item.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub item: Option<Box<Node>>,

    /// A flag to indicate if this list item is checked.
    #[serde(alias = "is-checked", alias = "is_checked")]
    #[patch(format = "md", format = "myst")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub is_checked: Option<Boolean>,

    /// The position of the item in a series or sequence of items.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub position: Option<Integer>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<ListItemOptions>,

    /// A unique identifier for a node within a document
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub struct ListItemOptions {
    /// Alternate names (aliases) for the item.
    #[serde(alias = "alternate-names", alias = "alternate_names", alias = "alternateName", alias = "alternate-name", alias = "alternate_name")]
    #[serde(default, deserialize_with = "option_csv_or_array")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub description: Option<Cord>,

    /// Any kind of identifier for any kind of Thing.
    #[serde(alias = "identifier")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub identifiers: Option<Vec<PropertyValueOrString>>,

    /// Images of the item.
    #[serde(alias = "image")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub images: Option<Vec<ImageObject>>,

    /// The name of the item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub name: Option<String>,

    /// The URL of the item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub url: Option<String>,
}

impl ListItem {
    const NICK: [u8; 3] = [108, 115, 105];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::ListItem
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(content: Vec<Block>) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }
}
