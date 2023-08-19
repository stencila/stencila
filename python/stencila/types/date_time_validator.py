# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .date_time import DateTime


class DateTimeValidator(BaseModel):
    """
    A validator specifying the constraints on a date-time.
    """

    id: Optional[str]
    """The identifier for this item"""

    minimum: Optional[DateTime]
    """The inclusive lower limit for a date-time."""

    maximum: Optional[DateTime]
    """The inclusive upper limit for a date-time."""
