# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .time_unit import TimeUnit


class Timestamp(BaseModel):
    """
    A value that represents a point in time
    """

    id: Optional[str]
    """The identifier for this item"""

    value: int
    """The time, in `timeUnit`s, before or after the Unix Epoch (1970-01-01T00:00:00Z)."""

    time_unit: TimeUnit
    """The time unit that the `value` represents."""
