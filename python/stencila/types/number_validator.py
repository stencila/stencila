# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class NumberValidator(BaseModel):
    """
    A validator specifying the constraints on a numeric node.
    """

    id: Optional[str]
    """The identifier for this item"""

    minimum: Optional[float]
    """The inclusive lower limit for a numeric node."""

    exclusive_minimum: Optional[float]
    """The exclusive lower limit for a numeric node."""

    maximum: Optional[float]
    """The inclusive upper limit for a numeric node."""

    exclusive_maximum: Optional[float]
    """The exclusive upper limit for a numeric node."""

    multiple_of: Optional[float]
    """A number that a numeric node must be a multiple of."""
