//! Generated file, do not edit

use crate::prelude::*;

use super::cite::Cite;
use super::string::String;

/// A group of Cite nodes.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct CiteGroup {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    pub r#type: MustBe!("CiteGroup"),

    /// The identifier for this item
    #[key]
    pub id: Option<String>,

    /// One or more `Cite`s to be referenced in the same surrounding text.
    pub items: Vec<Cite>,
}
