// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::author::Author;
use super::compilation_digest::CompilationDigest;
use super::compilation_message::CompilationMessage;
use super::cord::Cord;
use super::provenance_count::ProvenanceCount;
use super::string::String;

/// Document content in a specific format
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "RawBlock")]
#[patch(authors_on = "self")]
pub struct RawBlock {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("RawBlock"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The format of the raw content.
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"String::from("markdown")"#))]
    #[cfg_attr(feature = "proptest-low", proptest(value = r#"String::from("markdown")"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"r"[a-zA-Z0-9]{1,10}""#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"String::arbitrary()"#))]
    pub format: String,

    /// The raw content.
    #[walk]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"Cord::from("content")"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"r"[a-zA-Z0-9]{1,10}".prop_map(Cord::from)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"r"[^\p{C}]{1,100}".prop_map(Cord::from)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"String::arbitrary().prop_map(Cord::from)"#))]
    pub content: Cord,

    /// A digest of the `format` and `content` properties.
    #[serde(alias = "compilation-digest", alias = "compilation_digest")]
    #[strip(compilation)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub compilation_digest: Option<CompilationDigest>,

    /// Messages generated while parsing and transpiling the `content` into the `css` property.
    #[serde(alias = "compilation-messages", alias = "compilation_messages", alias = "compilationMessage", alias = "compilation-message", alias = "compilation_message")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(compilation)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub compilation_messages: Option<Vec<CompilationMessage>>,

    /// A Cascading Style Sheet (CSS) generated from the `content`.
    #[strip(output)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub css: Option<String>,

    /// The authors of the content.
    #[serde(alias = "author")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    #[strip(authors)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub authors: Option<Vec<Author>>,

    /// A summary of the provenance of the content.
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(provenance)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub provenance: Option<Vec<ProvenanceCount>>,

    /// A unique identifier for a node within a document
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[serde(skip)]
    pub uid: NodeUid
}

impl RawBlock {
    const NICK: [u8; 3] = [114, 97, 119];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::RawBlock
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(format: String, content: Cord) -> Self {
        Self {
            format,
            content,
            ..Default::default()
        }
    }
}
