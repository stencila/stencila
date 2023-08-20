# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .time import Time


@dataclass(kw_only=True, frozen=True)
class TimeValidator(Entity):
    """
    A validator specifying the constraints on a time.
    """

    type: Literal["TimeValidator"] = field(default="TimeValidator", init=False)

    minimum: Optional[Time] = None
    """The inclusive lower limit for a time."""

    maximum: Optional[Time] = None
    """The inclusive upper limit for a time."""
