# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .author import Author
from .block import Block
from .cite_or_text import CiteOrText
from .entity import Entity


@dataclass(init=False)
class QuoteBlock(Entity):
    """
    A section quoted from somewhere else.
    """

    type: Literal["QuoteBlock"] = field(default="QuoteBlock", init=False)

    cite: Optional[CiteOrText] = None
    """The source of the quote."""

    content: List[Block]
    """The content of the quote."""

    authors: Optional[List[Author]] = None
    """The authors of the quote."""

    def __init__(self, content: List[Block], id: Optional[str] = None, cite: Optional[CiteOrText] = None, authors: Optional[List[Author]] = None):
        super().__init__(id = id)
        self.cite = cite
        self.content = content
        self.authors = authors
