//! Generated file, do not edit

use crate::prelude::*;

use super::inline::Inline;
use super::string::String;

/// A hyperlink to other pages, sections within the same document, resources, or any URL.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Link {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    pub r#type: MustBe!("Link"),

    /// The identifier for this item
    #[key]
    pub id: Option<String>,

    /// The textual content of the link.
    pub content: Vec<Inline>,

    /// The target of the link.
    pub target: String,

    /// Non-core optional fields
    #[serde(flatten)]
    pub options: Box<LinkOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct LinkOptions {
    /// A title for the link.
    pub title: Option<String>,

    /// The relation between the target and the current thing.
    pub rel: Option<String>,
}
