// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::citation_intent::CitationIntent;
use super::citation_mode::CitationMode;
use super::inline::Inline;
use super::integer_or_string::IntegerOrString;
use super::string::String;

/// A reference to a `CreativeWork` that is cited in another `CreativeWork`.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "Cite")]
pub struct Cite {
    /// The type of this item.
    pub r#type: MustBe!("Cite"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The target of the citation (URL or reference ID).
    pub target: String,

    /// Determines how the citation is shown within the surrounding text.
    #[serde(alias = "citation-mode", alias = "citation_mode")]
    pub citation_mode: CitationMode,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<CiteOptions>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct CiteOptions {
    /// The type/s of the citation, both factually and rhetorically.
    #[serde(alias = "citation-intent", alias = "citation_intent")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    pub citation_intent: Option<Vec<CitationIntent>>,

    /// Optional structured content/text of this citation.
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[walk]
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

impl Cite {
    const NICK: &'static str = "cit";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Cite
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uid)
    }
    
    pub fn new(target: String, citation_mode: CitationMode) -> Self {
        Self {
            target,
            citation_mode,
            ..Default::default()
        }
    }
}
