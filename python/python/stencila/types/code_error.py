# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity


@dataclass(kw_only=True, frozen=True)
class CodeError(Entity):
    """
    An error that occurred when parsing, compiling or executing a `Code` node.
    """

    type: Literal["CodeError"] = field(default="CodeError", init=False)

    error_message: str
    """The error message or brief description of the error."""

    error_type: Optional[str] = None
    """The type of error e.g. "SyntaxError", "ZeroDivisionError"."""

    stack_trace: Optional[str] = None
    """Stack trace leading up to the error."""
