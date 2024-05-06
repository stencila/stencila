// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::provenance_category::ProvenanceCategory;
use super::string::String;
use super::unsigned_integer::UnsignedInteger;

/// The count of the number of characters in a `ProvenanceCategory` within an entity.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "ProvenanceCount")]
pub struct ProvenanceCount {
    /// The type of this item.
    pub r#type: MustBe!("ProvenanceCount"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The provenance category that the character count applies to.
    #[serde(alias = "provenance-category", alias = "provenance_category")]
    pub provenance_category: ProvenanceCategory,

    /// The number of characters in the provenance category.
    #[serde(alias = "character-count", alias = "character_count")]
    pub character_count: UnsignedInteger,

    /// The percentage of characters in the provenance category.
    #[serde(alias = "character-percent", alias = "character_percent")]
    pub character_percent: Option<UnsignedInteger>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl ProvenanceCount {
    const NICK: [u8; 3] = [112, 114, 99];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::ProvenanceCount
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(provenance_category: ProvenanceCategory, character_count: UnsignedInteger) -> Self {
        Self {
            provenance_category,
            character_count,
            ..Default::default()
        }
    }
}
