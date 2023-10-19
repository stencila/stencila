# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .creative_work import CreativeWork


@dataclass(kw_only=True, frozen=True)
class Figure(CreativeWork):
    """
    Encapsulates one or more images, videos, tables, etc, and provides captions and labels for them.
    """

    type: Literal["Figure"] = field(default="Figure", init=False)

    content: List[Block]
    """The content of the figure."""

    label: Optional[str] = None
    """A short label for the figure."""

    caption: Optional[List[Block]] = None
    """A caption for the figure."""
