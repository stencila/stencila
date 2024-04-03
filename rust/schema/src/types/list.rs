// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::author::Author;
use super::list_item::ListItem;
use super::list_order::ListOrder;
use super::string::String;

/// A list of items.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, MergeNode, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "List")]
#[html(special)]
#[jats(elem = "list")]
pub struct List {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("List"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The items in the list.
    #[serde(alias = "item")]
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[merge(format = "md")]
    #[cfg_attr(feature = "proptest-min", proptest(strategy = r#"vec(ListItem::arbitrary(), size_range(1..=1))"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"vec(ListItem::arbitrary(), size_range(1..=2))"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"vec(ListItem::arbitrary(), size_range(1..=4))"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"vec(ListItem::arbitrary(), size_range(1..=8))"#))]
    #[jats(content)]
    pub items: Vec<ListItem>,

    /// The ordering of the list.
    #[merge(format = "md")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"ListOrder::Unordered"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"prop_oneof![Just(ListOrder::Unordered),Just(ListOrder::Ascending)]"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"ListOrder::arbitrary()"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"ListOrder::arbitrary()"#))]
    #[jats(attr = "list-type")]
    pub order: ListOrder,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<ListOptions>,

    /// A unique identifier for a node within a document
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, MergeNode, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub struct ListOptions {
    /// The authors of the list.
    #[serde(alias = "author")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub authors: Option<Vec<Author>>,
}

impl List {
    const NICK: [u8; 3] = [108, 115, 116];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::List
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(items: Vec<ListItem>, order: ListOrder) -> Self {
        Self {
            items,
            order,
            ..Default::default()
        }
    }
}
