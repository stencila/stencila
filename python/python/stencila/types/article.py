# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .creative_work import CreativeWork
from .int_or_str import IntOrStr


@dataclass(kw_only=True, frozen=True)
class Article(CreativeWork):
    """
    An article, including news and scholarly articles.
    """

    type: Literal["Article"] = field(default="Article", init=False)

    content: List[Block]
    """The content of the article."""

    page_start: Optional[IntOrStr] = None
    """The page on which the article starts; for example "135" or "xiii"."""

    page_end: Optional[IntOrStr] = None
    """The page on which the article ends; for example "138" or "xvi"."""

    pagination: Optional[str] = None
    """Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55"."""
