// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::cite::Cite;
use super::string::String;

/// A group of Cite nodes.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, HtmlCodec, TextCodec, StripNode, Read, Write)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct CiteGroup {
    /// The type of this item
    pub r#type: MustBe!("CiteGroup"),

    /// The identifier for this item
    #[strip(id)]
    pub id: Option<String>,

    /// One or more `Cite`s to be referenced in the same surrounding text.
    pub items: Vec<Cite>,
}
impl CiteGroup {
    pub fn new(items: Vec<Cite>) -> Self {
        Self {
            items,
            ..Default::default()
        }
    }
}
