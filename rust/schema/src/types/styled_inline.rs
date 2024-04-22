// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::author::Author;
use super::compilation_digest::CompilationDigest;
use super::compilation_message::CompilationMessage;
use super::cord::Cord;
use super::inline::Inline;
use super::string::String;

/// Styled inline content.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "StyledInline")]
#[patch(authors_on = "options")]
#[html(elem = "span")]
#[jats(elem = "styled-content")]
pub struct StyledInline {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("StyledInline"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The code of the equation in the `styleLanguage`.
    #[patch(format = "md")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"Cord::new("code")"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"r"[a-zA-Z0-9 \t]{1,10}".prop_map(Cord::new)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"r"[^\p{C}]{1,100}".prop_map(Cord::new)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"String::arbitrary().prop_map(Cord::new)"#))]
    #[jats(attr = "style")]
    pub code: Cord,

    /// The language used for the style specification e.g. css, tw
    #[serde(alias = "style-language", alias = "style_language")]
    #[patch(format = "md")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"None"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"option::of(r"(css)|(tw)")"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"option::of(r"[a-zA-Z0-9]{1,10}")"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"option::of(String::arbitrary())"#))]
    #[jats(attr = "style-detail")]
    pub style_language: Option<String>,

    /// The content within the span.
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[patch(format = "md")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"vec![t("text")]"#))]
    #[cfg_attr(feature = "proptest-low", proptest(value = r#"vec![t("text")]"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"vec_inlines_non_recursive(2)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"vec_inlines_non_recursive(4)"#))]
    #[dom(elem = "span")]
    pub content: Vec<Inline>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<StyledInlineOptions>,

    /// A unique identifier for a node within a document
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub struct StyledInlineOptions {
    /// The authors of the styling code.
    #[serde(alias = "author")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    #[strip(authors)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub authors: Option<Vec<Author>>,

    /// A digest of the `code` and `styleLanguage`.
    #[serde(alias = "compilation-digest", alias = "compilation_digest")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub compilation_digest: Option<CompilationDigest>,

    /// Messages generated while parsing and transpiling the style.
    #[serde(alias = "compilation-messages", alias = "compilation_messages", alias = "compilationMessage", alias = "compilation-message", alias = "compilation_message")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub compilation_messages: Option<Vec<CompilationMessage>>,

    /// A Cascading Style Sheet (CSS) transpiled from the `code` property.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub css: Option<String>,

    /// A space separated list of class names associated with the node.
    #[serde(alias = "class-list", alias = "class_list")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub class_list: Option<String>,
}

impl StyledInline {
    const NICK: [u8; 3] = [115, 116, 105];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::StyledInline
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(code: Cord, content: Vec<Inline>) -> Self {
        Self {
            code,
            content,
            ..Default::default()
        }
    }
}
