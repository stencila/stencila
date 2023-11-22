# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .suggestion_block import SuggestionBlock


@dataclass(init=False)
class InsertBlock(SuggestionBlock):
    """
    A suggestion to insert some block content.
    """

    type: Literal["InsertBlock"] = field(default="InsertBlock", init=False)

    def __init__(self, content: List[Block], id: Optional[str] = None):
        super().__init__(id = id, content = content)
        
