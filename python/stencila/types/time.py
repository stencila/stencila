# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class Time(BaseModel):
    """
    A point in time recurring on multiple days
    """

    id: Optional[str]
    """The identifier for this item"""

    value: str
    """The time of day as a string in format `hh:mm:ss[Z|(+|-)hh:mm]`."""
