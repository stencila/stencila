# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .time_unit import TimeUnit
from .timestamp import Timestamp


@dataclass(init=False)
class TimestampValidator(Entity):
    """
    A validator specifying the constraints on a timestamp.
    """

    type: Literal["TimestampValidator"] = field(default="TimestampValidator", init=False)

    time_units: Optional[List[TimeUnit]] = None
    """The time units that the timestamp can have."""

    minimum: Optional[Timestamp] = None
    """The inclusive lower limit for a timestamp."""

    maximum: Optional[Timestamp] = None
    """The inclusive upper limit for a timestamp."""

    def __init__(self, id: Optional[str] = None, time_units: Optional[List[TimeUnit]] = None, minimum: Optional[Timestamp] = None, maximum: Optional[Timestamp] = None):
        super().__init__(id = id)
        self.time_units = time_units
        self.minimum = minimum
        self.maximum = maximum
