// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::modify_operation::ModifyOperation;
use super::string::String;

/// A suggestion to modify some block content.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "ModifyBlock")]
#[markdown(special)]
pub struct ModifyBlock {
    /// The type of this item.
    pub r#type: MustBe!("ModifyBlock"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The content that is suggested to be inserted or deleted.
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    pub content: Vec<Block>,

    /// The operations to be applied to the nodes.
    #[serde(alias = "operation")]
    #[serde(deserialize_with = "one_or_many")]
    pub operations: Vec<ModifyOperation>,
}

impl ModifyBlock {
    pub fn new(content: Vec<Block>, operations: Vec<ModifyOperation>) -> Self {
        Self {
            content,
            operations,
            ..Default::default()
        }
    }
}
