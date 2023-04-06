use crate::prelude::*;

/// Under which circumstances the document node should be automatically executed.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged, crate = "common::serde")]
#[def = "Unknown"]
pub enum ExecutionRequired {
    No,
    NeverExecuted,
    SemanticsChanged,
    DependenciesChanged,
    DependenciesFailed,
    Failed,
    KernelRestarted,
    Unknown,
}
