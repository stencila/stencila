# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class AdmonitionType(StrEnum):
    """
    The type of an `Admonition`.
    """

    Note = "Note"
    Info = "Info"
    Tip = "Tip"
    Important = "Important"
    Success = "Success"
    Failure = "Failure"
    Warning = "Warning"
    Danger = "Danger"
    Error = "Error"
