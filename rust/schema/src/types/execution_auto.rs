// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// Under which circumstances the document node should be automatically executed.
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, SmartDefault, Strip, Read, Write, ToHtml, ToText)]
#[serde(crate = "common::serde")]
pub enum ExecutionAuto {
    Never,
    #[default]
    Needed,
    Always,
}
