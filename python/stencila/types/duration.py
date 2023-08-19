# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .time_unit import TimeUnit


class Duration(BaseModel):
    """
    A value that represents the difference between two timestamps
    """

    id: Optional[str]
    """The identifier for this item"""

    value: int
    """The time difference in `timeUnit`s."""

    time_unit: TimeUnit
    """The time unit that the `value` represents."""
