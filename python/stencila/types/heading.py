# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .inline import Inline


class Heading(BaseModel):
    """
    A heading.
    """

    id: Optional[str]
    """The identifier for this item"""

    depth: int = 1
    """The depth of the heading."""

    content: List[Inline]
    """Content of the heading."""
