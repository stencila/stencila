# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .cite_or_str import CiteOrStr
from .entity import Entity


@dataclass(kw_only=True, frozen=True)
class QuoteBlock(Entity):
    """
    A section quoted from somewhere else.
    """

    type: Literal["QuoteBlock"] = field(default="QuoteBlock", init=False)

    cite: Optional[CiteOrStr] = None
    """The source of the quote."""

    content: List[Block]
    """The content of the quote."""
