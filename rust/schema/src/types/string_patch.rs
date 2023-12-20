// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;
use super::string_operation::StringOperation;

/// An set of operations to modify a string.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "StringPatch")]
pub struct StringPatch {
    /// The type of this item.
    pub r#type: MustBe!("StringPatch"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The operations to be applied to the string.
    #[serde(alias = "operation")]
    #[serde(deserialize_with = "one_or_many")]
    pub operations: Vec<StringOperation>,

    /// A universally unique identifier for this node
    
    #[serde(skip)]
    pub uuid: NodeUuid
}

impl StringPatch {
    pub fn new(operations: Vec<StringOperation>) -> Self {
        Self {
            operations,
            ..Default::default()
        }
    }
}

impl Entity for StringPatch {
    const NICK: &'static str = "str";

    fn node_type(&self) -> NodeType {
        NodeType::StringPatch
    }

    fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uuid)
    }
}
