// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::inline::Inline;
use super::string::String;

/// Superscripted content.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "Superscript")]
#[html(elem = "sup")]
#[jats(elem = "sup")]
#[markdown(template = "^{{content}}^", escape = "^")]
pub struct Superscript {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("Superscript"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The content that is marked.
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[patch(format = "all")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"vec![t("text")]"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"vec_inlines_non_recursive(1)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"vec_inlines_non_recursive(2)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"vec_inlines_non_recursive(4)"#))]
    #[dom(elem = "none")]
    pub content: Vec<Inline>,

    /// A unique identifier for a node within a document
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[serde(skip)]
    pub uid: NodeUid
}

impl Superscript {
    const NICK: [u8; 3] = [115, 117, 112];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Superscript
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(content: Vec<Inline>) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }
}
