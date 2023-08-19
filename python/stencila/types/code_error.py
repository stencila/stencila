# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class CodeError(BaseModel):
    """
    An error that occurred when parsing, compiling or executing a Code node.
    """

    id: Optional[str]
    """The identifier for this item"""

    error_message: str
    """The error message or brief description of the error."""

    error_type: Optional[str]
    """The type of error e.g. "SyntaxError", "ZeroDivisionError"."""

    stack_trace: Optional[str]
    """Stack trace leading up to the error."""
