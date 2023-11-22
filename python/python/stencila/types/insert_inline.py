# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .inline import Inline
from .suggestion_inline import SuggestionInline


@dataclass(init=False)
class InsertInline(SuggestionInline):
    """
    A suggestion to insert some inline content.
    """

    type: Literal["InsertInline"] = field(default="InsertInline", init=False)

    def __init__(self, content: List[Inline], id: Optional[str] = None):
        super().__init__(id = id, content = content)
        
