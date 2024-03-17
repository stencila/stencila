# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._inline import Inline
from ._modify_operation import ModifyOperation
from ._suggestion_inline import SuggestionInline
from ._suggestion_status import SuggestionStatus


@dataclass(init=False)
class ModifyInline(SuggestionInline):
    """
    A suggestion to modify some inline content.
    """

    type: Literal["ModifyInline"] = field(default="ModifyInline", init=False)

    operations: List[ModifyOperation]
    """The operations to be applied to the nodes."""

    def __init__(self, content: List[Inline], operations: List[ModifyOperation], id: Optional[str] = None, suggestion_status: Optional[SuggestionStatus] = None):
        super().__init__(id = id, suggestion_status = suggestion_status, content = content)
        self.operations = operations
