// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::author::Author;
use super::inline::Inline;
use super::integer::Integer;
use super::label_type::LabelType;
use super::provenance_count::ProvenanceCount;
use super::string::String;

/// A heading.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display("Heading")]
#[patch(authors_on = "self")]
#[html(special)]
#[jats(elem = "title", special)]
pub struct Heading {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("Heading"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The type of the label for the appendix (if present, should be `AppendixLabel`).
    #[serde(alias = "label-type", alias = "label_type")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"None"#))]
    #[cfg_attr(feature = "proptest-low", proptest(value = r#"None"#))]
    #[cfg_attr(feature = "proptest-high", proptest(value = r#"None"#))]
    #[cfg_attr(feature = "proptest-max", proptest(value = r#"None"#))]
    pub label_type: Option<LabelType>,

    /// A short label for the heading.
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"None"#))]
    #[cfg_attr(feature = "proptest-low", proptest(value = r#"None"#))]
    #[cfg_attr(feature = "proptest-high", proptest(value = r#"None"#))]
    #[cfg_attr(feature = "proptest-max", proptest(value = r#"None"#))]
    pub label: Option<String>,

    /// The level of the heading.
    #[default = 0]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd", format = "latex", format = "lexical", format = "koenig")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"1"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"1..=6i64"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"0..=6i64"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"i64::arbitrary()"#))]
    pub level: Integer,

    /// Content of the heading.
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[patch(format = "all")]
    #[cfg_attr(feature = "proptest-min", proptest(strategy = r#"vec_inlines(1)"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"vec_inlines(2)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"vec_inlines(4)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"vec(Inline::arbitrary(), size_range(0..=8))"#))]
    pub content: Vec<Inline>,

    /// The authors of the heading.
    #[serde(alias = "author")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    #[strip(authors)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub authors: Option<Vec<Author>>,

    /// A summary of the provenance of the content within the heading.
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(provenance)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub provenance: Option<Vec<ProvenanceCount>>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub uid: NodeUid
}

impl Heading {
    const NICK: [u8; 3] = *b"hea";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Heading
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(level: Integer, content: Vec<Inline>) -> Self {
        Self {
            level,
            content,
            ..Default::default()
        }
    }
}
