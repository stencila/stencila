# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._inline import Inline
from ._suggestion_inline import SuggestionInline
from ._suggestion_status import SuggestionStatus


@dataclass(init=False)
class ReplaceInline(SuggestionInline):
    """
    A suggestion to replace some inline content with new inline content.
    """

    type: Literal["ReplaceInline"] = field(default="ReplaceInline", init=False)

    replacement: List[Inline]
    """The new replacement inline content."""

    def __init__(self, content: List[Inline], replacement: List[Inline], id: Optional[str] = None, suggestion_status: Optional[SuggestionStatus] = None):
        super().__init__(id = id, suggestion_status = suggestion_status, content = content)
        self.replacement = replacement
