// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;

/// A calendar date encoded as a ISO 8601 string.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "Date")]
#[jats(elem = "date", special)]
pub struct Date {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("Date"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The date as an ISO 8601 string.
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"String::from("2022-02-22")"#))]
    #[cfg_attr(feature = "proptest-low", proptest(regex = r#"[0-9]{4}-[01][0-9]-[0-3][1-9]"#))]
    #[cfg_attr(feature = "proptest-high", proptest(regex = r#"[a-zA-Z0-9\-]{1,10}"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"String::arbitrary()"#))]
    pub value: String,
}

impl Date {
    pub fn new(value: String) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }
}
