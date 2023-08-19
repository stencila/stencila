# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .cite_or_str import CiteOrStr
from .inline import Inline


class Quote(BaseModel):
    """
    Inline, quoted content.
    """

    id: Optional[str]
    """The identifier for this item"""

    content: List[Inline]
    """The content that is marked."""

    cite: Optional[CiteOrStr]
    """The source of the quote."""
