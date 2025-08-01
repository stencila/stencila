// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::boolean::Boolean;
use super::execution_bounds::ExecutionBounds;
use super::integer::Integer;
use super::string::String;
use super::unsigned_integer::UnsignedInteger;

/// Model selection and inference parameters for generative AI models.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display("ModelParameters")]
pub struct ModelParameters {
    /// The type of this item.
    pub r#type: MustBe!("ModelParameters"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The ids of the models to select.
    #[serde(alias = "models", alias = "model", alias = "model-ids", alias = "model_ids", alias = "modelId", alias = "model-id", alias = "model_id")]
    #[serde(default, deserialize_with = "option_csv_or_array")]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub model_ids: Option<Vec<String>>,

    /// The number of replicate inferences to run per model id.
    #[serde(alias = "reps")]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub replicates: Option<UnsignedInteger>,

    /// The relative weighting given to model quality (0-100).
    #[serde(alias = "quality", alias = "qual", alias = "quality-weight", alias = "quality_weight")]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub quality_weight: Option<UnsignedInteger>,

    /// The relative weighting given to model cost (0-100).
    #[serde(alias = "cost", alias = "cost-weight", alias = "cost_weight")]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub cost_weight: Option<UnsignedInteger>,

    /// The relative weighting given to model speed (0-100).
    #[serde(alias = "speed", alias = "speed-weight", alias = "speed_weight")]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub speed_weight: Option<UnsignedInteger>,

    /// The minimum score for models to be selected (0-100).
    #[serde(alias = "minimum-score", alias = "minimum_score", alias = "minScore", alias = "min-score", alias = "min_score")]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub minimum_score: Option<UnsignedInteger>,

    /// The temperature option for model inference (0-100).
    #[serde(alias = "temp")]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub temperature: Option<UnsignedInteger>,

    /// The random seed used for the model (if possible)
    #[serde(alias = "random-seed", alias = "random_seed", alias = "rand-seed", alias = "rand_seed", alias = "seed")]
    pub random_seed: Option<Integer>,

    /// Automatically execute generated content.
    #[serde(alias = "execute-content", alias = "execute_content")]
    pub execute_content: Option<Boolean>,

    /// The environment in which code should be executed.
    #[serde(alias = "execution-bounds", alias = "execution_bounds")]
    pub execution_bounds: Option<ExecutionBounds>,

    /// When executing content, the maximum number of retries.
    #[serde(alias = "retries", alias = "maximum-retries", alias = "maximum_retries")]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub maximum_retries: Option<UnsignedInteger>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

impl ModelParameters {
    const NICK: [u8; 3] = *b"mdp";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::ModelParameters
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
