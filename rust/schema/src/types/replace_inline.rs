// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::inline::Inline;
use super::string::String;

/// A suggestion to replace some inline content with new inline content.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "ReplaceInline")]
#[markdown(template = "{~~{{content}}~>{{replacement}}~~}")]
pub struct ReplaceInline {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("ReplaceInline"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The content that is suggested to be inserted or deleted.
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"vec![t("text")]"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"vec_inlines_non_recursive(1)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"vec_inlines_non_recursive(2)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"vec_inlines_non_recursive(4)"#))]
    pub content: Vec<Inline>,

    /// The new replacement inline content.
    #[serde(deserialize_with = "one_or_many")]
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub replacement: Vec<Inline>,

    /// A universally unique identifier for this node
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[serde(skip)]
    pub uuid: NodeUuid
}

impl ReplaceInline {
    pub fn new(content: Vec<Inline>, replacement: Vec<Inline>) -> Self {
        Self {
            content,
            replacement,
            ..Default::default()
        }
    }
}

impl Entity for ReplaceInline {
    const NICK: &'static str = "rep";

    fn node_type(&self) -> NodeType {
        NodeType::ReplaceInline
    }

    fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uuid)
    }
}
