# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._entity import Entity
from ._suggestion_status import SuggestionStatus


@dataclass(init=False)
class Suggestion(Entity):
    """
    Abstract base type for nodes that indicate a suggested change to content.
    """

    type: Literal["Suggestion"] = field(default="Suggestion", init=False)

    suggestion_status: Optional[SuggestionStatus] = None
    """The status of the suggestion including whether it is proposed, accepted, or rejected."""

    def __init__(self, id: Optional[str] = None, suggestion_status: Optional[SuggestionStatus] = None):
        super().__init__(id = id)
        self.suggestion_status = suggestion_status
