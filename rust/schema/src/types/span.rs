// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::cord::Cord;
use super::execution_digest::ExecutionDigest;
use super::inline::Inline;
use super::string::String;

/// Styled inline content
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[html(elem = "span", custom)]
#[markdown(format = "[{content}]{{{code}}}")]
pub struct Span {
    /// The type of this item
    pub r#type: MustBe!("Span"),

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

    /// The content within the span
    pub content: Vec<Inline>,
}

impl Span {
    pub fn new(code: Cord, content: Vec<Inline>) -> Self {
        Self {
            code,
            content,
            ..Default::default()
        }
    }
}
