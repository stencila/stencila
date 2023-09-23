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
#[html(elem = "div", custom)]
#[markdown(format = "::: {{{code}}}\n\n{content}:::\n\n")]
pub struct Division {
    /// The type of this item
    pub r#type: MustBe!("Division"),

    /// The identifier for this item
    #[strip(id)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The code of the equation in the `styleLanguage`.
    pub code: Cord,

    /// The language used for the style specification e.g. css, tailwind, classes.
    pub style_language: Option<String>,

    /// A digest of the `code` and `styleLanguage`.
    pub compile_digest: Option<ExecutionDigest>,

    /// Errors that occurred when transpiling the `code`.
    pub errors: Option<Vec<String>>,

    /// A Cascading Style Sheet (CSS) transpiled from the `code` property.
    pub css: Option<String>,

    /// A list of class names associated with the node
    pub classes: Option<Vec<String>>,

    /// The content within the division
    #[strip(types)]
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
