// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::author::Author;
use super::duration::Duration;
use super::inline::Inline;
use super::provenance_count::ProvenanceCount;
use super::string::String;
use super::suggestion_status::SuggestionStatus;
use super::timestamp::Timestamp;

/// Abstract base type for nodes that indicate a suggested change to inline content.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "SuggestionInline")]
#[patch(authors_on = "self")]
pub struct SuggestionInline {
    /// The type of this item.
    pub r#type: MustBe!("SuggestionInline"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The status of the suggestion including whether it is proposed, accepted, or rejected.
    #[serde(alias = "suggestion-status", alias = "suggestion_status")]
    #[strip(metadata)]
    #[patch(format = "md", format = "myst")]
    pub suggestion_status: SuggestionStatus,

    /// The authors of the suggestion
    #[serde(alias = "author")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    #[strip(authors)]
    #[dom(elem = "span")]
    pub authors: Option<Vec<Author>>,

    /// A summary of the provenance of the content within the suggestion.
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(provenance)]
    #[dom(elem = "span")]
    pub provenance: Option<Vec<ProvenanceCount>>,

    /// Time taken to generate the suggestion.
    #[serde(alias = "execution-duration", alias = "execution_duration")]
    #[strip(execution)]
    #[dom(with = "Duration::to_dom_attr")]
    pub execution_duration: Option<Duration>,

    /// The timestamp when the generation ended.
    #[serde(alias = "execution-ended", alias = "execution_ended")]
    #[strip(execution, timestamps)]
    #[dom(with = "Timestamp::to_dom_attr")]
    pub execution_ended: Option<Timestamp>,

    /// Feedback on the suggestion
    #[patch(format = "md", format = "myst")]
    pub feedback: Option<String>,

    /// The content that is suggested to be inserted, modified, replaced, or deleted.
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[patch(format = "all")]
    #[dom(elem = "span")]
    pub content: Vec<Inline>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl SuggestionInline {
    const NICK: [u8; 3] = [115, 103, 105];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::SuggestionInline
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(suggestion_status: SuggestionStatus, content: Vec<Inline>) -> Self {
        Self {
            suggestion_status,
            content,
            ..Default::default()
        }
    }
}
