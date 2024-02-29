# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class ExecutionRequired(StrEnum):
    """
    Whether, and why, the execution of a node is required or not.
    """

    No = "No"
    NeverExecuted = "NeverExecuted"
    SemanticsChanged = "SemanticsChanged"
    DependenciesChanged = "DependenciesChanged"
    DependenciesFailed = "DependenciesFailed"
    ExecutionFailed = "ExecutionFailed"
    KernelRestarted = "KernelRestarted"
    UserRequested = "UserRequested"
