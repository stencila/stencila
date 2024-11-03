// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::boolean::Boolean;
use super::string::String;
use super::walkthrough_step::WalkthroughStep;

/// An interactive walkthrough made up of several, successively revealed steps.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "Walkthrough")]
#[patch(apply_with = "Walkthrough::apply_with")]
pub struct Walkthrough {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("Walkthrough"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// Whether the walkthrough is expanded so that each step can be edited
    #[serde(alias = "is-expanded", alias = "is_expanded")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub is_expanded: Option<Boolean>,

    /// The steps making up the walkthrough.
    #[serde(alias = "step")]
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[patch(format = "all")]
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[dom(elem = "div")]
    pub steps: Vec<WalkthroughStep>,

    /// A unique identifier for a node within a document
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[serde(skip)]
    pub uid: NodeUid
}

impl Walkthrough {
    const NICK: [u8; 3] = [119, 107, 116];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Walkthrough
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(steps: Vec<WalkthroughStep>) -> Self {
        Self {
            steps,
            ..Default::default()
        }
    }
}
