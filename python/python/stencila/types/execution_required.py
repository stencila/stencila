# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class ExecutionRequired(StrEnum):
    """
    Under which circumstances the document node should be automatically executed.
    """

    No = "No"
    NeverExecuted = "NeverExecuted"
    SemanticsChanged = "SemanticsChanged"
    DependenciesChanged = "DependenciesChanged"
    DependenciesFailed = "DependenciesFailed"
    Failed = "Failed"
    KernelRestarted = "KernelRestarted"
