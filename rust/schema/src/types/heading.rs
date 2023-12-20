// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::inline::Inline;
use super::integer::Integer;
use super::string::String;

/// A heading.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, HtmlCodec, JatsCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "Heading")]
#[html(special)]
#[jats(elem = "title", special)]
pub struct Heading {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("Heading"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The level of the heading.
    #[default = 0]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"1"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"1..=6i64"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"0..=6i64"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"i64::arbitrary()"#))]
    pub level: Integer,

    /// Content of the heading.
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[cfg_attr(feature = "proptest-min", proptest(strategy = r#"vec_inlines(1)"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"vec_inlines(2)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"vec_inlines(4)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"vec(Inline::arbitrary(), size_range(0..=8))"#))]
    pub content: Vec<Inline>,

    /// A universally unique identifier for this node
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[serde(skip)]
    pub uuid: NodeUuid
}

impl Heading {
    pub fn new(level: Integer, content: Vec<Inline>) -> Self {
        Self {
            level,
            content,
            ..Default::default()
        }
    }
}

impl Entity for Heading {
    const NICK: &'static str = "hea";

    fn node_type(&self) -> NodeType {
        NodeType::Heading
    }

    fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uuid)
    }
}
