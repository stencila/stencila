# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .inline import Inline


@dataclass(init=False)
class Suggestion(Entity):
    """
    Abstract base class for nodes that indicate a suggested change to inline content.
    """

    type: Literal["Suggestion"] = field(default="Suggestion", init=False)

    content: List[Inline]
    """The content that is suggested to be inserted or deleted."""

    def __init__(self, content: List[Inline], id: Optional[str] = None):
        super().__init__(id = id)
        self.content = content
