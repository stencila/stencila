// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;

/// Inline code.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, TextCodec, Read, Write)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[html(elem = "code", custom)]
pub struct CodeFragment {
    /// The type of this item
    pub r#type: MustBe!("CodeFragment"),

    /// The identifier for this item
    #[strip(id)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The code.
    #[html(content)]
    pub code: String,

    /// The programming language of the code.
    pub programming_language: Option<String>,

    /// Non-core optional fields
    #[serde(flatten)]
    pub options: Box<CodeFragmentOptions>,
}

#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, TextCodec, Read, Write)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[html(elem = "code", custom)]
pub struct CodeFragmentOptions {
    /// Media type, typically expressed using a MIME format, of the code.
    pub media_type: Option<String>,
}

impl CodeFragment {
    pub fn new(code: String) -> Self {
        Self {
            code,
            ..Default::default()
        }
    }
}
