# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .duration import Duration
from .entity import Entity
from .time_unit import TimeUnit


@dataclass(kw_only=True, frozen=True)
class DurationValidator(Entity):
    """
    A validator specifying the constraints on a duration.
    """

    type: Literal["DurationValidator"] = field(default="DurationValidator", init=False)

    time_units: Optional[List[TimeUnit]] = None
    """The time units that the duration can have."""

    minimum: Optional[Duration] = None
    """The inclusive lower limit for a duration."""

    maximum: Optional[Duration] = None
    """The inclusive upper limit for a duration."""
