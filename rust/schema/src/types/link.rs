//! Generated file, do not edit

use crate::prelude::*;

use super::inline::Inline;
use super::string::String;

/// A hyperlink to other pages, sections within the same document, resources, or any URL.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Link {
    /// The type of this item
    r#type: MustBe!("Link"),

    /// The identifier for this item
    id: String,

    /// The textual content of the link.
    content: Vec<Inline>,

    /// The target of the link.
    target: String,

    /// Non-core optional fields
    #[serde(flatten)]
    options: Box<LinkOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct LinkOptions {
    /// A title for the link.
    title: Option<String>,

    /// The relation between the target and the current thing.
    rel: Option<String>,
}
