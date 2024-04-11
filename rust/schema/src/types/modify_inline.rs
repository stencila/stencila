// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::inline::Inline;
use super::modify_operation::ModifyOperation;
use super::string::String;
use super::suggestion_status::SuggestionStatus;

/// A suggestion to modify some inline content.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "ModifyInline")]
pub struct ModifyInline {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("ModifyInline"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The status of the suggestion including whether it is proposed, accepted, or rejected.
    #[serde(alias = "suggestion-status", alias = "suggestion_status")]
    #[strip(metadata)]
    #[merge(format = "md")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub suggestion_status: Option<SuggestionStatus>,

    /// The content that is suggested to be inserted, modified, replaced, or deleted.
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[merge(format = "all")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"vec![t("text")]"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"vec_inlines_non_recursive(1)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"vec_inlines_non_recursive(2)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"vec_inlines_non_recursive(4)"#))]
    #[dom(elem = "span")]
    pub content: Vec<Inline>,

    /// The operations to be applied to the nodes.
    #[serde(alias = "operation")]
    #[serde(deserialize_with = "one_or_many")]
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[dom(elem = "span")]
    pub operations: Vec<ModifyOperation>,

    /// A unique identifier for a node within a document
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[serde(skip)]
    pub uid: NodeUid
}

impl ModifyInline {
    const NICK: [u8; 3] = [109, 100, 105];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::ModifyInline
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(content: Vec<Inline>, operations: Vec<ModifyOperation>) -> Self {
        Self {
            content,
            operations,
            ..Default::default()
        }
    }
}
