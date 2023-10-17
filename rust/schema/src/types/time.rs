// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;

/// A point in time recurring on multiple days
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub struct Time {
    /// The type of this item
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("Time"),

    /// The identifier for this item
    #[strip(id)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The time of day as a string in format `hh:mm:ss[Z|(+|-)hh:mm]`.
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"String::from("2022-02-22T22:22:22")"#))]
    #[cfg_attr(feature = "proptest-low", proptest(regex = r#"[0-2][0-9]:[0-5][0-9]:[0-5][0-9]\.[0-9]+([+-][0-2][0-9]:[0-5][0-9]|Z)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(regex = r#"[a-zA-Z0-9\-:]{1,20}"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"String::arbitrary()"#))]
    pub value: String,
}

impl Time {
    pub fn new(value: String) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }
}
