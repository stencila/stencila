# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._block import Block
from ._modify_operation import ModifyOperation
from ._suggestion_block import SuggestionBlock
from ._suggestion_status import SuggestionStatus


@dataclass(init=False)
class ModifyBlock(SuggestionBlock):
    """
    A suggestion to modify some block content.
    """

    type: Literal["ModifyBlock"] = field(default="ModifyBlock", init=False)

    operations: List[ModifyOperation]
    """The operations to be applied to the nodes."""

    def __init__(self, content: List[Block], operations: List[ModifyOperation], id: Optional[str] = None, suggestion_status: Optional[SuggestionStatus] = None):
        super().__init__(id = id, suggestion_status = suggestion_status, content = content)
        self.operations = operations
