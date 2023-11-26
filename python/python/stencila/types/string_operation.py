# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .unsigned_integer import UnsignedInteger


@dataclass(init=False)
class StringOperation(Entity):
    """
    An operation that modifies a string.
    """

    type: Literal["StringOperation"] = field(default="StringOperation", init=False)

    start_position: UnsignedInteger
    """The start position in the string of the operation."""

    end_position: Optional[UnsignedInteger] = None
    """The end position in the string of the operation."""

    value: Optional[str] = None
    """The string value to insert or use as the replacement."""

    def __init__(self, start_position: UnsignedInteger, id: Optional[str] = None, end_position: Optional[UnsignedInteger] = None, value: Optional[str] = None):
        super().__init__(id = id)
        self.start_position = start_position
        self.end_position = end_position
        self.value = value
