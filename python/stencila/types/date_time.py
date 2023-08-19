# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class DateTime(BaseModel):
    """
    A combination of date and time of day in the form `[-]CCYY-MM-DDThh:mm:ss[Z|(+|-)hh:mm]`.
    """

    id: Optional[str]
    """The identifier for this item"""

    value: str
    """The date as an ISO 8601 string."""
