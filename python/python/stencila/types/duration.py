# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .time_unit import TimeUnit


@dataclass(init=False)
class Duration(Entity):
    """
    A value that represents the difference between two timestamps.
    """

    type: Literal["Duration"] = field(default="Duration", init=False)

    value: int
    """The time difference in `timeUnit`s."""

    time_unit: TimeUnit
    """The time unit that the `value` represents."""

    def __init__(self, value: int, time_unit: TimeUnit, id: Optional[str] = None):
        super().__init__(id = id)
        self.value = value
        self.time_unit = time_unit
