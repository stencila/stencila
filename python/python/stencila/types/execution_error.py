# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .code_location import CodeLocation
from .entity import Entity


@dataclass(init=False)
class ExecutionError(Entity):
    """
    An error that occurred when executing an executable node.
    """

    type: Literal["ExecutionError"] = field(default="ExecutionError", init=False)

    error_message: str
    """The error message or brief description of the error."""

    error_type: Optional[str] = None
    """The type of error e.g. "SyntaxError", "ZeroDivisionError"."""

    code_location: Optional[CodeLocation] = None
    """The location that the error occurred."""

    stack_trace: Optional[str] = None
    """Stack trace leading up to the error."""

    def __init__(self, error_message: str, id: Optional[str] = None, error_type: Optional[str] = None, code_location: Optional[CodeLocation] = None, stack_trace: Optional[str] = None):
        super().__init__(id = id)
        self.error_message = error_message
        self.error_type = error_type
        self.code_location = code_location
        self.stack_trace = stack_trace
