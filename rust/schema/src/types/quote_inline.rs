// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::cite_or_text::CiteOrText;
use super::inline::Inline;
use super::string::String;

/// Inline, quoted content.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "QuoteInline")]
#[html(elem = "q")]
#[jats(elem = "inline-quote")]
#[markdown(template = "<q>{content}</q>")]
pub struct QuoteInline {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("QuoteInline"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The content that is marked.
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"vec![t("text")]"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"vec_inlines_non_recursive(1)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"vec_inlines_non_recursive(2)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"vec_inlines_non_recursive(4)"#))]
    pub content: Vec<Inline>,

    /// The source of the quote.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub cite: Option<CiteOrText>,

    /// A unique identifier for this node
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[serde(skip)]
    pub node_id: NodeId
}

impl QuoteInline {
    pub fn new(content: Vec<Inline>) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }
}

impl Entity for QuoteInline {
    fn node_type() -> NodeType {
        NodeType::QuoteInline
    }

    fn node_id(&self) -> &NodeId {
        &self.node_id
    }
}
