# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .unsigned_integer import UnsignedInteger


@dataclass(init=False)
class CodeLocation(Entity):
    """
    The location within some source code.
    """

    type: Literal["CodeLocation"] = field(default="CodeLocation", init=False)

    source: Optional[str] = None
    """The source of the code, a file path, label or URL."""

    start_line: Optional[UnsignedInteger] = None
    """The 1-based index if the first line on which the error occurred."""

    start_column: Optional[UnsignedInteger] = None
    """The 1-based index if the first column on which the error occurred."""

    end_line: Optional[UnsignedInteger] = None
    """The 1-based index if the last line on which the error occurred."""

    end_column: Optional[UnsignedInteger] = None
    """The 1-based index if the last column on which the error occurred."""

    def __init__(self, id: Optional[str] = None, source: Optional[str] = None, start_line: Optional[UnsignedInteger] = None, start_column: Optional[UnsignedInteger] = None, end_line: Optional[UnsignedInteger] = None, end_column: Optional[UnsignedInteger] = None):
        super().__init__(id = id)
        self.source = source
        self.start_line = start_line
        self.start_column = start_column
        self.end_line = end_line
        self.end_column = end_column
