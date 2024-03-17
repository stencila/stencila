# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._block import Block
from ._suggestion_block import SuggestionBlock
from ._suggestion_status import SuggestionStatus


@dataclass(init=False)
class ReplaceBlock(SuggestionBlock):
    """
    A suggestion to replace some block content with new block content.
    """

    type: Literal["ReplaceBlock"] = field(default="ReplaceBlock", init=False)

    replacement: List[Block]
    """The new replacement block content."""

    def __init__(self, content: List[Block], replacement: List[Block], id: Optional[str] = None, suggestion_status: Optional[SuggestionStatus] = None):
        super().__init__(id = id, suggestion_status = suggestion_status, content = content)
        self.replacement = replacement
