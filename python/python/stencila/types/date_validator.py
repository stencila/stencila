# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .date import Date
from .entity import Entity


@dataclass(init=False)
class DateValidator(Entity):
    """
    A validator specifying the constraints on a date.
    """

    type: Literal["DateValidator"] = field(default="DateValidator", init=False)

    minimum: Optional[Date] = None
    """The inclusive lower limit for a date."""

    maximum: Optional[Date] = None
    """The inclusive upper limit for a date."""

    def __init__(self, id: Optional[str] = None, minimum: Optional[Date] = None, maximum: Optional[Date] = None):
        super().__init__(id = id)
        self.minimum = minimum
        self.maximum = maximum
