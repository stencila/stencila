// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::integer::Integer;
use super::string::String;
use super::time_unit::TimeUnit;

/// A value that represents the difference between two timestamps.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display("Duration")]
#[jats(elem = "duration", special)]
pub struct Duration {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("Duration"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    #[jats(attr = "id")]
    pub id: Option<String>,

    /// The time difference in `timeUnit`s.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub value: Integer,

    /// The time unit that the `value` represents.
    #[serde(alias = "time-unit", alias = "time_unit")]
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub time_unit: TimeUnit,

    /// A unique identifier for a node within a document
    #[serde(skip)]
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub uid: NodeUid
}

impl Duration {
    const NICK: [u8; 3] = *b"dur";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Duration
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(value: Integer, time_unit: TimeUnit) -> Self {
        Self {
            value,
            time_unit,
            ..Default::default()
        }
    }
}
