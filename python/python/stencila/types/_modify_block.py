# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._block import Block
from ._modify_operation import ModifyOperation
from ._suggestion_block import SuggestionBlock


@dataclass(init=False)
class ModifyBlock(SuggestionBlock):
    """
    A suggestion to modify some block content.
    """

    type: Literal["ModifyBlock"] = field(default="ModifyBlock", init=False)

    operations: List[ModifyOperation]
    """The operations to be applied to the nodes."""

    def __init__(self, content: List[Block], operations: List[ModifyOperation], id: Optional[str] = None):
        super().__init__(id = id, content = content)
        self.operations = operations
