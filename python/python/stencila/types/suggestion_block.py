# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .suggestion import Suggestion


@dataclass(init=False)
class SuggestionBlock(Suggestion):
    """
    Abstract base type for nodes that indicate a suggested change to block content.
    """

    type: Literal["SuggestionBlock"] = field(default="SuggestionBlock", init=False)

    content: List[Block]
    """The content that is suggested to be inserted or deleted."""

    def __init__(self, content: List[Block], id: Optional[str] = None):
        super().__init__(id = id)
        self.content = content
