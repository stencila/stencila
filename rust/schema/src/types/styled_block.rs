// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::compilation_digest::CompilationDigest;
use super::compilation_error::CompilationError;
use super::cord::Cord;
use super::string::String;

/// Styled block content.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "StyledBlock")]
#[html(elem = "div")]
pub struct StyledBlock {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("StyledBlock"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The code of the equation in the `styleLanguage`.
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"Cord::new("code")"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"r"[a-zA-Z0-9 \t]{1,10}".prop_map(Cord::new)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"r"[^\p{C}]{1,100}".prop_map(Cord::new)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"String::arbitrary().prop_map(Cord::new)"#))]
    #[jats(attr = "style")]
    pub code: Cord,

    /// The language used for the style specification e.g. css, tw
    #[serde(alias = "style-language", alias = "style_language")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"None"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"option::of(r"(css)|(tw)")"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"option::of(r"[a-zA-Z0-9]{1,10}")"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"option::of(String::arbitrary())"#))]
    #[jats(attr = "style-detail")]
    pub style_language: Option<String>,

    /// A digest of the `code` and `styleLanguage`.
    #[serde(alias = "compilation-digest", alias = "compilation_digest")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub compilation_digest: Option<CompilationDigest>,

    /// Errors generated when parsing and transpiling the style.
    #[serde(alias = "compilation-errors", alias = "compilation_errors", alias = "compilationError", alias = "compilation-error", alias = "compilation_error")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub compilation_errors: Option<Vec<CompilationError>>,

    /// A Cascading Style Sheet (CSS) transpiled from the `code` property.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub css: Option<String>,

    /// A list of class names associated with the node.
    #[serde(alias = "class")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub classes: Option<Vec<String>>,

    /// The content within the styled block
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub content: Vec<Block>,

    /// A universally unique identifier for this node
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[serde(skip)]
    pub uuid: NodeUuid
}

impl StyledBlock {
    pub fn new(code: Cord, content: Vec<Block>) -> Self {
        Self {
            code,
            content,
            ..Default::default()
        }
    }
}

impl Entity for StyledBlock {
    const NICK: &'static str = "sty";

    fn node_type(&self) -> NodeType {
        NodeType::StyledBlock
    }

    fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uuid)
    }
}
