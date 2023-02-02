//! Generated file, do not edit

use crate::prelude::*;

use super::cite::Cite;
use super::string::String;

/// A group of Cite nodes.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct CiteGroup {
    /// The type of this item
    r#type: MustBe!("CiteGroup"),

    /// The identifier for this item
    id: String,

    /// One or more `Cite`s to be referenced in the same surrounding text.
    items: Vec<Cite>,
}
