# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class MessageLevel(StrEnum):
    """
    The severity level of a message.
    """

    Trace = "Trace"
    Debug = "Debug"
    Info = "Info"
    Warning = "Warning"
    Error = "Error"
    Exception = "Exception"
