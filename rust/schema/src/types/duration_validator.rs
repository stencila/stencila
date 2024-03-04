// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::duration::Duration;
use super::string::String;
use super::time_unit::TimeUnit;

/// A validator specifying the constraints on a duration.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "DurationValidator")]
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
    pub time_units: Option<Vec<TimeUnit>>,

    /// The inclusive lower limit for a duration.
    pub minimum: Option<Duration>,

    /// The inclusive upper limit for a duration.
    pub maximum: Option<Duration>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl DurationValidator {
    const NICK: [u8; 3] = [100, 117, 118];
    
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
