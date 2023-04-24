use crate::prelude::*;

/// Under which circumstances the document node should be automatically executed.
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Defaults, Strip, Read, Write, ToHtml)]
#[serde(crate = "common::serde")]
#[def = "Needed"]
pub enum ExecutionAuto {
    Never,
    Needed,
    Always,
}
