use crate::prelude::*;

/// Under which circumstances the document node should be automatically executed.
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Defaults, Read, Write, ToHtml)]
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
