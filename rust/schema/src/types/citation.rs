// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::citation_intent::CitationIntent;
use super::citation_mode::CitationMode;
use super::compilation_message::CompilationMessage;
use super::inline::Inline;
use super::integer_or_string::IntegerOrString;
use super::reference::Reference;
use super::string::String;

/// A reference to a `CreativeWork` that is cited in another `CreativeWork`.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display("Citation")]
#[jats(special)]
pub struct Citation {
    /// The type of this item.
    pub r#type: MustBe!("Citation"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The target of the citation (URL or reference ID).
    #[patch(format = "all")]
    pub target: String,

    /// Determines how the citation is shown within the surrounding text.
    #[serde(alias = "citation-mode", alias = "citation_mode")]
    #[patch(format = "all")]
    pub citation_mode: Option<CitationMode>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<CitationOptions>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct CitationOptions {
    /// Messages generated while resolving the target if the citation.
    #[serde(alias = "compilation-messages", alias = "compilation_messages", alias = "compilationMessage", alias = "compilation-message", alias = "compilation_message")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(compilation)]
    pub compilation_messages: Option<Vec<CompilationMessage>>,

    /// The `Reference` being cited, resolved from the `target`.
    #[dom(elem = "span")]
    pub cites: Option<Reference>,

    /// The type/s of the citation, both factually and rhetorically.
    #[serde(alias = "citation-intent", alias = "citation_intent")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    pub citation_intent: Option<Vec<CitationIntent>>,

    /// Optional structured content/text of this citation.
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[walk]
    #[patch(format = "all")]
    #[dom(elem = "span")]
    pub content: Option<Vec<Inline>>,

    /// The page on which the work starts; for example "135" or "xiii".
    #[serde(alias = "page-start", alias = "page_start")]
    pub page_start: Option<IntegerOrString>,

    /// The page on which the work ends; for example "138" or "xvi".
    #[serde(alias = "page-end", alias = "page_end")]
    pub page_end: Option<IntegerOrString>,

    /// Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55".
    pub pagination: Option<String>,

    /// Text to show before the citation.
    #[serde(alias = "citation-prefix", alias = "citation_prefix")]
    pub citation_prefix: Option<String>,

    /// Text to show after the citation.
    #[serde(alias = "citation-suffix", alias = "citation_suffix")]
    pub citation_suffix: Option<String>,
}

impl Citation {
    const NICK: [u8; 3] = [99, 105, 116];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Citation
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(target: String) -> Self {
        Self {
            target,
            ..Default::default()
        }
    }
}
