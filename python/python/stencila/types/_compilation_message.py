# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._code_location import CodeLocation
from ._entity import Entity
from ._message_level import MessageLevel


@dataclass(init=False)
class CompilationMessage(Entity):
    """
    An error, warning or log message generated during compilation.
    """

    type: Literal["CompilationMessage"] = field(default="CompilationMessage", init=False)

    level: MessageLevel
    """The severity level of the message."""

    message: str
    """The text of the message."""

    error_type: Optional[str] = None
    """The type of error e.g. "SyntaxError", "ZeroDivisionError"."""

    code_location: Optional[CodeLocation] = None
    """The location that the error occurred."""

    def __init__(self, level: MessageLevel, message: str, id: Optional[str] = None, error_type: Optional[str] = None, code_location: Optional[CodeLocation] = None):
        super().__init__(id = id)
        self.level = level
        self.message = message
        self.error_type = error_type
        self.code_location = code_location
