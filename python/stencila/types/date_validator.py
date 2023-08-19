# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .date import Date


class DateValidator(BaseModel):
    """
    A validator specifying the constraints on a date.
    """

    id: Optional[str]
    """The identifier for this item"""

    minimum: Optional[Date]
    """The inclusive lower limit for a date."""

    maximum: Optional[Date]
    """The inclusive upper limit for a date."""
