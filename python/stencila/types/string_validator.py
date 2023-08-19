# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *


class StringValidator(BaseModel):
    """
    A schema specifying constraints on a string node.
    """

    id: Optional[str]
    """The identifier for this item"""

    min_length: Optional[int]
    """The minimum length for a string node."""

    max_length: Optional[int]
    """The maximum length for a string node."""

    pattern: Optional[str]
    """A regular expression that a string node must match."""
