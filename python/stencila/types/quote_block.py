# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .cite_or_str import CiteOrStr


class QuoteBlock(BaseModel):
    """
    A section quoted from somewhere else.
    """

    id: Optional[str]
    """The identifier for this item"""

    cite: Optional[CiteOrStr]
    """The source of the quote."""

    content: List[Block]
    """The content of the quote."""
