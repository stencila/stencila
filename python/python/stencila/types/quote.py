# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .cite_or_text import CiteOrText
from .mark import Mark


@dataclass(kw_only=True, frozen=True)
class Quote(Mark):
    """
    Inline, quoted content.
    """

    type: Literal["Quote"] = field(default="Quote", init=False)

    cite: Optional[CiteOrText] = None
    """The source of the quote."""
