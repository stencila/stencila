// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::duration::Duration;
use super::string::String;
use super::time_unit::TimeUnit;

/// A validator specifying the constraints on a duration.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display("DurationValidator")]
pub struct DurationValidator {
    /// The type of this item.
    pub r#type: MustBe!("DurationValidator"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The time units that the duration can have.
    #[serde(alias = "time-units", alias = "time_units", alias = "timeUnit", alias = "time-unit", alias = "time_unit")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub time_units: Option<Vec<TimeUnit>>,

    /// The inclusive lower limit for a duration.
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub minimum: Option<Duration>,

    /// The inclusive upper limit for a duration.
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    pub maximum: Option<Duration>,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    pub uid: NodeUid
}

impl DurationValidator {
    const NICK: [u8; 3] = *b"duv";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::DurationValidator
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
