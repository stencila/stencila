# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .time_unit import TimeUnit
from .timestamp import Timestamp


class TimestampValidator(BaseModel):
    """
    A validator specifying the constraints on a timestamp.
    """

    id: Optional[str]
    """The identifier for this item"""

    time_units: Optional[List[TimeUnit]]
    """The time units that the timestamp can have."""

    minimum: Optional[Timestamp]
    """The inclusive lower limit for a timestamp."""

    maximum: Optional[Timestamp]
    """The inclusive upper limit for a timestamp."""
