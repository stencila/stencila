# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .inline import Inline


class Strong(BaseModel):
    """
    Strongly emphasised content.
    """

    id: Optional[str]
    """The identifier for this item"""

    content: List[Inline]
    """The content that is marked."""
