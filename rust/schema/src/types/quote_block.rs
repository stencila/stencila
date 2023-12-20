// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::cite_or_text::CiteOrText;
use super::string::String;

/// A section quoted from somewhere else.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "QuoteBlock")]
#[html(elem = "blockquote")]
#[jats(elem = "disp-quote")]
pub struct QuoteBlock {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("QuoteBlock"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The source of the quote.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub cite: Option<CiteOrText>,

    /// The content of the quote.
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[cfg_attr(feature = "proptest-min", proptest(strategy = r#"vec_paragraphs(1)"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"vec_blocks_non_recursive(2)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"vec_blocks_non_recursive(4)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"vec_blocks_non_recursive(8)"#))]
    pub content: Vec<Block>,

    /// A universally unique identifier for this node
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[serde(skip)]
    pub uuid: NodeUuid
}

impl QuoteBlock {
    pub fn new(content: Vec<Block>) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }
}

impl Entity for QuoteBlock {
    const NICK: &'static str = "quo";

    fn node_type(&self) -> NodeType {
        NodeType::QuoteBlock
    }

    fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uuid)
    }
}
