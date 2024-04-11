// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::section_type::SectionType;
use super::string::String;

/// A section of a document.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "Section")]
#[html(elem = "section", special)]
#[jats(elem = "sec")]
pub struct Section {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("Section"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The content within the section.
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[merge(format = "all")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"Vec::new()"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"vec_heading_paragraph()"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"vec_blocks_non_recursive(4)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"vec_blocks_non_recursive(8)"#))]
    #[dom(elem = "section")]
    pub content: Vec<Block>,

    /// The type of section.
    #[serde(alias = "section-type", alias = "section_type")]
    #[merge(format = "md")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"None"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"option::of(SectionType::arbitrary())"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"option::of(SectionType::arbitrary())"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"option::of(SectionType::arbitrary())"#))]
    #[jats(attr = "content-type")]
    pub section_type: Option<SectionType>,

    /// A unique identifier for a node within a document
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[serde(skip)]
    pub uid: NodeUid
}

impl Section {
    const NICK: [u8; 3] = [115, 101, 99];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Section
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(content: Vec<Block>) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }
}
