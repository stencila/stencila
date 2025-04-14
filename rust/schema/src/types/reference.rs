// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::author::Author;
use super::creative_work_type::CreativeWorkType;
use super::date::Date;
use super::integer_or_string::IntegerOrString;
use super::string::String;

/// A reference to a creative work, including books, movies, photographs, software programs, etc.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display("Reference")]
#[patch(authors_on = "self")]
pub struct Reference {
    /// The type of this item.
    pub r#type: MustBe!("Reference"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The Digital Object Identifier for the work.
    pub doi: Option<String>,

    /// The authors of the work.
    #[serde(alias = "author")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    pub authors: Option<Vec<Author>>,

    /// Date of first publication.
    #[serde(default, deserialize_with = "option_string_or_object")]
    #[dom(with = "Date::to_dom_attr")]
    pub date: Option<Date>,

    /// The title of the work.
    #[serde(alias = "headline")]
    #[dom(attr = "_title")]
    pub title: Option<String>,

    /// An other `CreativeWork` that the reference is a part of.
    #[serde(alias = "is-part-of", alias = "is_part_of")]
    pub is_part_of: Option<Box<CreativeWorkType>>,

    /// The page on which the article starts; for example "135" or "xiii".
    #[serde(alias = "page-start", alias = "page_start")]
    pub page_start: Option<IntegerOrString>,

    /// The page on which the article ends; for example "138" or "xvi".
    #[serde(alias = "page-end", alias = "page_end")]
    pub page_end: Option<IntegerOrString>,

    /// Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55".
    pub pagination: Option<String>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

impl Reference {
    const NICK: [u8; 3] = [114, 101, 102];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Reference
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
