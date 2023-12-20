// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::inline::Inline;
use super::string::String;

/// A paragraph.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "Paragraph")]
#[html(elem = "p")]
#[jats(elem = "p")]
#[markdown(template = "{{content}}\n\n")]
pub struct Paragraph {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("Paragraph"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The contents of the paragraph.
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[cfg_attr(feature = "proptest-min", proptest(strategy = r#"vec_inlines(1)"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"vec_inlines(2)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"vec_inlines(4)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"vec(Inline::arbitrary(), size_range(0..=8))"#))]
    pub content: Vec<Inline>,

    /// A universally unique identifier for this node
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[serde(skip)]
    pub uuid: NodeUuid
}

impl Paragraph {
    pub fn new(content: Vec<Inline>) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }
}

impl Entity for Paragraph {
    const NICK: &'static str = "par";

    fn node_type(&self) -> NodeType {
        NodeType::Paragraph
    }

    fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uuid)
    }
}
