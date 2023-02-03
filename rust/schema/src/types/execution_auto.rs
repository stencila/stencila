//! Generated file, do not edit

use crate::prelude::*;

/// Under which circumstances the document node should be automatically executed.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(untagged, crate = "common::serde")]
#[def = "Needed"]
pub enum ExecutionAuto {
    Never,
    Needed,
    Always,
}
