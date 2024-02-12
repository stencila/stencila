# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class ExecutionMessageLevel(StrEnum):
    """
    The severity level of an execution message.
    """

    Trace = "Trace"
    Debug = "Debug"
    Info = "Info"
    Warn = "Warn"
    Error = "Error"
