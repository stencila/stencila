# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class Date(BaseModel):
    """
    A calendar date encoded as a ISO 8601 string.
    """

    id: Optional[str]
    """The identifier for this item"""

    value: str
    """The date as an ISO 8601 string."""
