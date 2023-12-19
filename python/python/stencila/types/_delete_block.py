# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._block import Block
from ._suggestion_block import SuggestionBlock


@dataclass(init=False)
class DeleteBlock(SuggestionBlock):
    """
    A suggestion to delete some block content.
    """

    type: Literal["DeleteBlock"] = field(default="DeleteBlock", init=False)

    def __init__(self, content: List[Block], id: Optional[str] = None):
        super().__init__(id = id, content = content)
        
