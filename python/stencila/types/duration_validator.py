# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .duration import Duration
from .time_unit import TimeUnit


class DurationValidator(BaseModel):
    """
    A validator specifying the constraints on a duration.
    """

    id: Optional[str]
    """The identifier for this item"""

    time_units: Optional[List[TimeUnit]]
    """The time units that the duration can have."""

    minimum: Optional[Duration]
    """The inclusive lower limit for a duration."""

    maximum: Optional[Duration]
    """The inclusive upper limit for a duration."""
