# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._entity import Entity
from ._primitive import Primitive


@dataclass(init=False)
class ArrayHint(Entity):
    """
    A hint to the content of an `Array`.
    """

    type: Literal["ArrayHint"] = field(default="ArrayHint", init=False)

    length: int
    """The length (number of items) of the array."""

    types: Optional[List[str]] = None
    """The distinct types of the array items."""

    minimum: Optional[Primitive] = None
    """The minimum value in the array."""

    maximum: Optional[Primitive] = None
    """The maximum value in the array."""

    nulls: Optional[int] = None
    """The number of `Null` values in the array."""

    def __init__(self, length: int, id: Optional[str] = None, types: Optional[List[str]] = None, minimum: Optional[Primitive] = None, maximum: Optional[Primitive] = None, nulls: Optional[int] = None):
        super().__init__(id = id)
        self.length = length
        self.types = types
        self.minimum = minimum
        self.maximum = maximum
        self.nulls = nulls
