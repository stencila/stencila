# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .time_unit import TimeUnit


@dataclass(kw_only=True, frozen=True)
class Timestamp(Entity):
    """
    A value that represents a point in time.
    """

    type: Literal["Timestamp"] = field(default="Timestamp", init=False)

    value: int
    """The time, in `timeUnit`s, before or after the Unix Epoch (1970-01-01T00:00:00Z)."""

    time_unit: TimeUnit
    """The time unit that the `value` represents."""
