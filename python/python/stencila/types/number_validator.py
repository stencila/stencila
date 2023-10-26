# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity


@dataclass(init=False)
class NumberValidator(Entity):
    """
    A validator specifying the constraints on a numeric node.
    """

    type: Literal["NumberValidator"] = field(default="NumberValidator", init=False)

    minimum: Optional[float] = None
    """The inclusive lower limit for a numeric node."""

    exclusive_minimum: Optional[float] = None
    """The exclusive lower limit for a numeric node."""

    maximum: Optional[float] = None
    """The inclusive upper limit for a numeric node."""

    exclusive_maximum: Optional[float] = None
    """The exclusive upper limit for a numeric node."""

    multiple_of: Optional[float] = None
    """A number that a numeric node must be a multiple of."""

    def __init__(self, id: Optional[str] = None, minimum: Optional[float] = None, exclusive_minimum: Optional[float] = None, maximum: Optional[float] = None, exclusive_maximum: Optional[float] = None, multiple_of: Optional[float] = None):
        super().__init__(id = id)
        self.minimum = minimum
        self.exclusive_minimum = exclusive_minimum
        self.maximum = maximum
        self.exclusive_maximum = exclusive_maximum
        self.multiple_of = multiple_of
