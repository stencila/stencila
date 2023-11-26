# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .modify_operation import ModifyOperation
from .suggestion_block import SuggestionBlock


@dataclass(init=False)
class ModifyBlock(SuggestionBlock):
    """
    A suggestion to modify some block content.
    """

    type: Literal["ModifyBlock"] = field(default="ModifyBlock", init=False)

    def __init__(self, content: List[Block], operations: List[ModifyOperation], id: Optional[str] = None):
        super().__init__(id = id, content = content, operations = operations)
        
