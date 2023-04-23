use crate::prelude::*;

/// Under which circumstances the document node should be automatically executed.
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Strip, Defaults, Read, Write, ToHtml)]
#[serde(crate = "common::serde")]
#[def = "Needed"]
pub enum ExecutionAuto {
    Never,
    Needed,
    Always,
}
