# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .inline import Inline
from .suggestion import Suggestion


@dataclass(init=False)
class SuggestionInline(Suggestion):
    """
    Abstract base type for nodes that indicate a suggested change to inline content.
    """

    type: Literal["SuggestionInline"] = field(default="SuggestionInline", init=False)

    content: List[Inline]
    """The content that is suggested to be inserted or deleted."""

    def __init__(self, content: List[Inline], id: Optional[str] = None):
        super().__init__(id = id)
        self.content = content
