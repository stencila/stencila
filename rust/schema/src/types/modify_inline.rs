// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::inline::Inline;
use super::modify_operation::ModifyOperation;
use super::string::String;

/// A suggestion to modify some inline content.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "ModifyInline")]
#[markdown(special)]
pub struct ModifyInline {
    /// The type of this item.
    pub r#type: MustBe!("ModifyInline"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The content that is suggested to be inserted or deleted.
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    pub content: Vec<Inline>,

    /// The operations to be applied to the nodes.
    #[serde(alias = "operation")]
    #[serde(deserialize_with = "one_or_many")]
    pub operations: Vec<ModifyOperation>,
}

impl ModifyInline {
    pub fn new(content: Vec<Inline>, operations: Vec<ModifyOperation>) -> Self {
        Self {
            content,
            operations,
            ..Default::default()
        }
    }
}
