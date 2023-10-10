// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;

/// A combination of date and time of day in the form `[-]CCYY-MM-DDThh:mm:ss[Z|(+|-)hh:mm]`.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub struct DateTime {
    /// The type of this item
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("DateTime"),

    /// The identifier for this item
    #[strip(id)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The date as an ISO 8601 string.
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"String::from("2022-02-22T22:22:22")"#))]
    #[cfg_attr(feature = "proptest-low", proptest(regex = r#"\d{4}-[01]\d-[0-3]\dT[0-2]\d:[0-5]\d:[0-5]\d\.\d+([+-][0-2]\d:[0-5]\d|Z)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(regex = r#"[a-zA-Z0-9\-:]{1,20}"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"String::arbitrary()"#))]
    pub value: String,
}

impl DateTime {
    pub fn new(value: String) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }
}
