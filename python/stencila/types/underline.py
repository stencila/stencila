# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .inline import Inline


class Underline(BaseModel):
    """
    Inline text that is underlined.
    """

    id: Optional[str]
    """The identifier for this item"""

    content: List[Inline]
    """The content that is marked."""
