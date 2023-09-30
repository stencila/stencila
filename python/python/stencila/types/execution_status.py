# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class ExecutionStatus(StrEnum):
    """
    Status of the most recent, including any current, execution of a document node.
    """

    Scheduled = "Scheduled"
    ScheduledPreviouslyFailed = "ScheduledPreviouslyFailed"
    Running = "Running"
    RunningPreviouslyFailed = "RunningPreviouslyFailed"
    Succeeded = "Succeeded"
    Failed = "Failed"
    Cancelled = "Cancelled"
