// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;

/// A combination of date and time of day in the form `[-]CCYY-MM-DDThh:mm:ss[Z|(+|-)hh:mm]`.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "DateTime")]
#[jats(elem = "date-time", special)]
pub struct DateTime {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("DateTime"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The date as an ISO 8601 string.
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"String::from("2022-02-22T22:22:22")"#))]
    #[cfg_attr(feature = "proptest-low", proptest(regex = r#"[0-9]{4}-[01][0-9]-[0-3][0-9]T[0-2][0-9]:[0-5][0-9]:[0-5][0-9]\.[0-9]+([+-][0-2][0-9]:[0-5][0-9]|Z)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(regex = r#"[a-zA-Z0-9\-:]{1,20}"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"String::arbitrary()"#))]
    pub value: String,

    /// A universally unique identifier for this node
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[serde(skip)]
    pub uuid: NodeUuid
}

impl DateTime {
    pub fn new(value: String) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }
}

impl Entity for DateTime {
    const NICK: &'static str = "dat";

    fn node_type(&self) -> NodeType {
        NodeType::DateTime
    }

    fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uuid)
    }
}
