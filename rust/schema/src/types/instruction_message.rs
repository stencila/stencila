// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::author::Author;
use super::file::File;
use super::inline::Inline;
use super::message_role::MessageRole;
use super::provenance_count::ProvenanceCount;
use super::string::String;

/// A message within an `Instruction`.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
#[derive(derive_more::Display)]
#[display("InstructionMessage")]
#[patch(authors_on = "self")]
pub struct InstructionMessage {
    /// The type of this item.
    pub r#type: MustBe!("InstructionMessage"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    #[jats(attr = "id")]
    pub id: Option<String>,

    /// The role of the message in the conversation.
    pub role: Option<MessageRole>,

    /// The content of the message as inline nodes.
    #[serde(default)]
    #[walk]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    #[dom(elem = "div")]
    pub content: Vec<Inline>,

    /// Files attached to the message.
    #[serde(alias = "file")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[dom(elem = "div")]
    pub files: Option<Vec<File>>,

    /// The authors of the message.
    #[serde(alias = "author")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(authors)]
    #[dom(elem = "span")]
    pub authors: Option<Vec<Author>>,

    /// A summary of the provenance of the messages and content within the instruction.
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(provenance)]
    #[dom(elem = "span")]
    pub provenance: Option<Vec<ProvenanceCount>>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

impl InstructionMessage {
    const NICK: [u8; 3] = *b"ime";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::InstructionMessage
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(content: Vec<Inline>) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }
}
