// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// Under which circumstances the document node should be automatically executed.
#[derive(Debug, Display, Clone, PartialEq, Serialize, Deserialize, HtmlCodec, TextCodec, StripNode, SmartDefault, Read, Write)]
#[serde(crate = "common::serde")]
pub enum ExecutionAuto {
    Never,
    #[default]
    Needed,
    Always,
}
