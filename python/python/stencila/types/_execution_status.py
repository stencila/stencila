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
    Warnings = "Warnings"
    Errors = "Errors"
    Exceptions = "Exceptions"
    Interrupted = "Interrupted"
