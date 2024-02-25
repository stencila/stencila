# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class ExecutionStatus(StrEnum):
    """
    Status of the most recent, including any current, execution of a document node.
    """

    Scheduled = "Scheduled"
    Skipped = "Skipped"
    Empty = "Empty"
    Running = "Running"
    Succeeded = "Succeeded"
    Failed = "Failed"
    Cancelled = "Cancelled"
