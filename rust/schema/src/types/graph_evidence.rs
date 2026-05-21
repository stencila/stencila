// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::graph_evidence_confidence::GraphEvidenceConfidence;
use super::graph_evidence_kind::GraphEvidenceKind;
use super::object::Object;
use super::string::String;
use super::thing_variant_or_string::ThingVariantOrString;
use super::timestamp::Timestamp;

/// Evidence for a graph edge.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
#[derive(derive_more::Display)]
#[display("GraphEvidence")]
pub struct GraphEvidence {
    /// The type of this item.
    pub r#type: MustBe!("GraphEvidence"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    #[jats(attr = "id")]
    pub id: Option<String>,

    /// The kind of evidence.
    pub kind: GraphEvidenceKind,

    /// The confidence in the evidence.
    pub confidence: Option<GraphEvidenceConfidence>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<GraphEvidenceOptions>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
pub struct GraphEvidenceOptions {
    /// The source of the evidence.
    pub source: Option<ThingVariantOrString>,

    /// When this evidence was recorded.
    #[serde(alias = "recorded-at", alias = "recorded_at")]
    #[strip(timestamps)]
    pub recorded_at: Option<Timestamp>,

    /// Additional structured details about the evidence for machine consumers.
    pub details: Option<Object>,

    /// A human-readable description of the evidence.
    pub description: Option<String>,
}

impl GraphEvidence {
    const NICK: [u8; 3] = *b"gev";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::GraphEvidence
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(kind: GraphEvidenceKind) -> Self {
        Self {
            kind,
            ..Default::default()
        }
    }
}
