// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::inline::Inline;
use super::string::String;

/// A hyperlink to other pages, sections within the same document, resources, or any URL.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "Link")]
#[html(elem = "a")]
#[jats(elem = "ext-link")]
pub struct Link {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("Link"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The textual content of the link.
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[patch(format = "md", format = "smd", format = "myst")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"vec![t("text")]"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"vec_inlines_non_recursive(1)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"vec_inlines_non_recursive(2)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"vec_inlines_non_recursive(4)"#))]
    pub content: Vec<Inline>,

    /// The target of the link.
    #[patch(format = "md", format = "smd", format = "myst")]
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[html(attr = "href")]
    #[jats(attr = "xlink:href")]
    pub target: String,

    /// A title for the link.
    #[patch(format = "md", format = "smd", format = "myst")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "title")]
    #[jats(attr = "xlink:title")]
    pub title: Option<String>,

    /// The relation between the target and the current thing.
    #[patch(format = "md", format = "smd", format = "myst")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "rel")]
    pub rel: Option<String>,

    /// A unique identifier for a node within a document
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[serde(skip)]
    pub uid: NodeUid
}

impl Link {
    const NICK: [u8; 3] = [108, 105, 110];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Link
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(content: Vec<Inline>, target: String) -> Self {
        Self {
            content,
            target,
            ..Default::default()
        }
    }
}
