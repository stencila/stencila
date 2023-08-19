# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .time import Time


class TimeValidator(BaseModel):
    """
    A validator specifying the constraints on a time.
    """

    id: Optional[str]
    """The identifier for this item"""

    minimum: Optional[Time]
    """The inclusive lower limit for a time."""

    maximum: Optional[Time]
    """The inclusive upper limit for a time."""
