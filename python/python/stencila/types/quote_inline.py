# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .cite_or_text import CiteOrText
from .inline import Inline
from .mark import Mark


@dataclass(init=False)
class QuoteInline(Mark):
    """
    Inline, quoted content.
    """

    type: Literal["QuoteInline"] = field(default="QuoteInline", init=False)

    cite: Optional[CiteOrText] = None
    """The source of the quote."""

    def __init__(self, content: List[Inline], id: Optional[str] = None, cite: Optional[CiteOrText] = None):
        super().__init__(id = id, content = content)
        self.cite = cite
