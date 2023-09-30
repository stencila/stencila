# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .date import Date
from .entity import Entity


@dataclass(kw_only=True, frozen=True)
class DateValidator(Entity):
    """
    A validator specifying the constraints on a date.
    """

    type: Literal["DateValidator"] = field(default="DateValidator", init=False)

    minimum: Optional[Date] = None
    """The inclusive lower limit for a date."""

    maximum: Optional[Date] = None
    """The inclusive upper limit for a date."""
