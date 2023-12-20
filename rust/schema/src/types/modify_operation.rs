// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;
use super::string_patch_or_primitive::StringPatchOrPrimitive;

/// An operation that is part of a suggestion to modify the property of a node.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "ModifyOperation")]
pub struct ModifyOperation {
    /// The type of this item.
    pub r#type: MustBe!("ModifyOperation"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The target property of each node to be modified.
    pub target: String,

    /// The new value, or string patch, to apply to the target property.
    pub value: Box<StringPatchOrPrimitive>,

    /// A universally unique identifier for this node
    
    #[serde(skip)]
    pub uuid: NodeUuid
}

impl ModifyOperation {
    pub fn new(target: String, value: Box<StringPatchOrPrimitive>) -> Self {
        Self {
            target,
            value,
            ..Default::default()
        }
    }
}

impl Entity for ModifyOperation {
    const NICK: &'static str = "mod";

    fn node_type(&self) -> NodeType {
        NodeType::ModifyOperation
    }

    fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uuid)
    }
}
