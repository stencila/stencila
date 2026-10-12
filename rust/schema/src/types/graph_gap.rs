// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::graph_gap_state::GraphGapState;
use super::graph_gap_target::GraphGapTarget;
use super::string::String;
use super::thing_variant_or_string::ThingVariantOrString;

/// A sourced limitation of an account, occurrence, relationship or evidentiary basis.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, InspectNode, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
#[derive(derive_more::Display)]
#[display("GraphGap")]
pub struct GraphGap {
    /// The type of this item.
    pub r#type: MustBe!("GraphGap"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[patch(format = "md", format = "smd", format = "myst", format = "qmd", format = "tiptap")]
    #[html(attr = "id")]
    #[jats(attr = "id")]
    pub id: Option<String>,

    /// The distinct availability or coverage state.
    pub state: GraphGapState,

    /// The producer whose account this gap qualifies.
    pub source: ThingVariantOrString,

    /// The structured scope of the limitation.
    pub target: GraphGapTarget,

    /// Human-readable scope of the limitation.
    pub description: String,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<GraphGapOptions>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, InspectNode, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
pub struct GraphGapOptions {
    /// The affected resource's durable identity, never a graph-local node id.
    pub resource: Option<String>,

    /// A stable producer-specific reason code.
    pub code: Option<String>,
}

impl GraphGap {
    const NICK: [u8; 3] = *b"ggp";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::GraphGap
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(state: GraphGapState, source: ThingVariantOrString, target: GraphGapTarget, description: String) -> Self {
        Self {
            state,
            source,
            target,
            description,
            ..Default::default()
        }
    }
}
