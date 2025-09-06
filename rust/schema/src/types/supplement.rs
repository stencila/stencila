// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::boolean::Boolean;
use super::compilation_message::CompilationMessage;
use super::creative_work_type::CreativeWorkType;
use super::creative_work_variant::CreativeWorkVariant;
use super::string::String;

/// A supplementary `CreativeWork` that supports this work but is not considered part of its main content.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
#[derive(derive_more::Display)]
#[display("Supplement")]
#[jats(elem = "supplementary-material")]
pub struct Supplement {
    /// The type of this item.
    pub r#type: MustBe!("Supplement"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    #[jats(attr = "id")]
    pub id: Option<String>,

    /// The `CreativeWork` type of the supplement.
    #[serde(alias = "work-type", alias = "work_type")]
    pub work_type: Option<CreativeWorkType>,

    /// A short identifier or title for the supplement (e.g., "S1").
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    #[jats(elem = "label")]
    pub label: Option<String>,

    /// Whether the supplement label should be automatically generated and updated.
    #[serde(alias = "label-automatically", alias = "label_automatically")]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub label_automatically: Option<Boolean>,

    /// A brief caption or description for the supplement.
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[walk]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    #[dom(elem = "div")]
    #[jats(elem = "caption")]
    pub caption: Option<Vec<Block>>,

    /// A reference to the supplement.
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    #[jats(attr = "href")]
    pub target: Option<String>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<SupplementOptions>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
pub struct SupplementOptions {
    /// Any messages generated while embedding the supplement.
    #[serde(alias = "compilation-messages", alias = "compilation_messages", alias = "compilationMessage", alias = "compilation-message", alias = "compilation_message")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(compilation)]
    #[dom(elem = "span")]
    pub compilation_messages: Option<Vec<CompilationMessage>>,

    /// The `CreativeWork` that constitutes the supplement.
    #[strip(content)]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    #[dom(elem = "div")]
    pub work: Option<CreativeWorkVariant>,
}

impl Supplement {
    const NICK: [u8; 3] = *b"spl";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Supplement
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
