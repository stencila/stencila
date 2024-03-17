# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._block import Block
from ._suggestion_block import SuggestionBlock
from ._suggestion_status import SuggestionStatus


@dataclass(init=False)
class DeleteBlock(SuggestionBlock):
    """
    A suggestion to delete some block content.
    """

    type: Literal["DeleteBlock"] = field(default="DeleteBlock", init=False)

    def __init__(self, content: List[Block], id: Optional[str] = None, suggestion_status: Optional[SuggestionStatus] = None):
        super().__init__(id = id, suggestion_status = suggestion_status, content = content)
        
