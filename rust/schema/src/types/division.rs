// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::cord::Cord;
use super::execution_digest::ExecutionDigest;
use super::string::String;

/// Styled block content
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[html(elem = "div", custom)]
#[markdown(special)]
pub struct Division {
    /// The type of this item
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("Division"),

    /// The identifier for this item
    #[strip(id)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The code of the equation in the `styleLanguage`.
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"Cord::new("code")"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"r"[a-zA-Z0-9\s\t]{1,10}".prop_map(Cord::new)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"r"[^`]{1,100}".prop_map(Cord::new)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"String::arbitrary().prop_map(Cord::new)"#))]
    pub code: Cord,

    /// The language used for the style specification e.g. css, tw
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"None"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"option::of(r"(css)|(tw)")"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"option::of(r"[a-zA-Z0-9]{1,10}")"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"option::of(String::arbitrary())"#))]
    pub style_language: Option<String>,

    /// A digest of the `code` and `styleLanguage`.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub compile_digest: Option<ExecutionDigest>,

    /// Errors that occurred when transpiling the `code`.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub errors: Option<Vec<String>>,

    /// A Cascading Style Sheet (CSS) transpiled from the `code` property.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub css: Option<String>,

    /// A list of class names associated with the node
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub classes: Option<Vec<String>>,

    /// The content within the division
    #[strip(types)]
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub content: Vec<Block>,
}

impl Division {
    pub fn new(code: Cord, content: Vec<Block>) -> Self {
        Self {
            code,
            content,
            ..Default::default()
        }
    }
}
