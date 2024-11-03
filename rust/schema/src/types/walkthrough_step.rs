// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::boolean::Boolean;
use super::string::String;

/// A step in a walkthrough.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "WalkthroughStep")]
pub struct WalkthroughStep {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("WalkthroughStep"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// Whether this step is active (i.e. is encoded in source format and can be edited)
    #[serde(alias = "is-active", alias = "is_active")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub is_active: Option<Boolean>,

    /// The content of the step.
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[patch(format = "all")]
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[dom(elem = "div")]
    pub content: Vec<Block>,

    /// A unique identifier for a node within a document
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[serde(skip)]
    pub uid: NodeUid
}

impl WalkthroughStep {
    const NICK: [u8; 3] = [119, 107, 115];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::WalkthroughStep
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
