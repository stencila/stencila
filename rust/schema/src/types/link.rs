// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::inline::Inline;
use super::string::String;

/// A hyperlink to other pages, sections within the same document, resources, or any URL.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, TextCodec, Read, Write)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[html(elem = "a")]
pub struct Link {
    /// The type of this item
    pub r#type: MustBe!("Link"),

    /// The identifier for this item
    #[strip(id)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The textual content of the link.
    pub content: Vec<Inline>,

    /// The target of the link.
    #[html(attr = "href")]
    pub target: String,

    /// Non-core optional fields
    #[serde(flatten)]
    pub options: Box<LinkOptions>,
}

#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, TextCodec, Read, Write)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[html(elem = "a")]
pub struct LinkOptions {
    /// A title for the link.
    pub title: Option<String>,

    /// The relation between the target and the current thing.
    pub rel: Option<String>,
}

impl Link {
    pub fn new(content: Vec<Inline>, target: String) -> Self {
        Self {
            content,
            target,
            ..Default::default()
        }
    }
}
