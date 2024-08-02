// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::integer::Integer;
use super::string::String;
use super::unsigned_integer::UnsignedInteger;

/// Model selection criteria and execution options for the generative model used for an instruction.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "InstructionModel")]
pub struct InstructionModel {
    /// The type of this item.
    pub r#type: MustBe!("InstructionModel"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// A pattern to filter model ids by.
    #[serde(alias = "id-pattern", alias = "id_pattern")]
    #[patch(format = "md", format = "myst")]
    pub id_pattern: Option<String>,

    /// The relative weighting given to model quality (0-100).
    #[serde(alias = "quality-weight", alias = "quality_weight")]
    #[patch(format = "md", format = "myst")]
    pub quality_weight: Option<UnsignedInteger>,

    /// The relative weighting given to model speed (0-100).
    #[serde(alias = "speed-weight", alias = "speed_weight")]
    #[patch(format = "md", format = "myst")]
    pub speed_weight: Option<UnsignedInteger>,

    /// The relative weighting given to model cost (0-100).
    #[serde(alias = "cost-weight", alias = "cost_weight")]
    #[patch(format = "md", format = "myst")]
    pub cost_weight: Option<UnsignedInteger>,

    /// The minimum score for models to be selected (0-100).
    #[serde(alias = "minimum-score", alias = "minimum_score")]
    #[patch(format = "md", format = "myst")]
    pub minimum_score: Option<UnsignedInteger>,

    /// The temperature option for model inference (0-100).
    #[patch(format = "md", format = "myst")]
    pub temperature: Option<UnsignedInteger>,

    /// The random seed used for the model (if possible)
    #[serde(alias = "random-seed", alias = "random_seed")]
    pub random_seed: Option<Integer>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl InstructionModel {
    const NICK: [u8; 3] = [105, 115, 109];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::InstructionModel
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
