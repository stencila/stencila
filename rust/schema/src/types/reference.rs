// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::author::Author;
use super::creative_work_type::CreativeWorkType;
use super::date::Date;
use super::inline::Inline;
use super::integer_or_string::IntegerOrString;
use super::person::Person;
use super::person_or_organization::PersonOrOrganization;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;
use super::string_or_number::StringOrNumber;
use super::unsigned_integer::UnsignedInteger;

/// A reference to a creative work, including books, movies, photographs, software programs, etc.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, TextCodec)]
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

    /// The index (1-based) of appearance order of the reference in the work.
    #[serde(alias = "appearance-index", alias = "appearance_index")]
    pub appearance_index: Option<UnsignedInteger>,

    /// The type of `CreativeWork` being referenced (e.g. Article, Book, Dataset).
    #[serde(alias = "work-type", alias = "work_type")]
    pub work_type: Option<CreativeWorkType>,

    /// The Digital Object Identifier (https://doi.org/) of the work being referenced.
    pub doi: Option<String>,

    /// The authors of the work.
    #[serde(alias = "author")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    pub authors: Option<Vec<Author>>,

    /// People who edited the referenced work.
    #[serde(alias = "editor")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    #[strip(metadata)]
    pub editors: Option<Vec<Person>>,

    /// A publisher of the referenced work.
    #[serde(default, deserialize_with = "option_string_or_object")]
    #[strip(metadata)]
    pub publisher: Option<PersonOrOrganization>,

    /// Date of first publication.
    #[serde(default, deserialize_with = "option_string_or_object")]
    #[dom(with = "Date::to_dom_attr")]
    pub date: Option<Date>,

    /// The title of the referenced work.
    #[serde(alias = "headline")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[walk]
    #[dom(elem = "span")]
    pub title: Option<Vec<Inline>>,

    /// Another `Reference` that this reference is a part of.
    #[serde(alias = "is-part-of", alias = "is_part_of")]
    pub is_part_of: Option<Box<Reference>>,

    /// Identifies the volume of publication or multi-part work; for example, "iii" or "2".
    #[serde(alias = "volume-number", alias = "volume_number")]
    pub volume_number: Option<IntegerOrString>,

    /// Identifies the issue of a serial publication; for example, "3" or "12".
    #[serde(alias = "issue-number", alias = "issue_number")]
    pub issue_number: Option<IntegerOrString>,

    /// The page on which the article starts; for example "135" or "xiii".
    #[serde(alias = "page-start", alias = "page_start")]
    pub page_start: Option<IntegerOrString>,

    /// The page on which the article ends; for example "138" or "xvi".
    #[serde(alias = "page-end", alias = "page_end")]
    pub page_end: Option<IntegerOrString>,

    /// Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55".
    pub pagination: Option<String>,

    /// The version/edition of the referenced work.
    #[strip(metadata)]
    pub version: Option<StringOrNumber>,

    /// Any kind of identifier for the referenced work.
    #[serde(alias = "identifier")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    pub identifiers: Option<Vec<PropertyValueOrString>>,

    /// The URL of the referenced work.
    pub url: Option<String>,

    /// Plain text representation of the referenced work.
    pub text: Option<String>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

impl Reference {
    const NICK: [u8; 3] = *b"ref";
    
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
