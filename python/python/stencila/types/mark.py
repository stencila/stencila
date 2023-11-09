# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .inline import Inline


@dataclass(init=False)
class Mark(Entity):
    """
    Abstract base class for nodes that mark some other inline content in some way (e.g. as being emphasised, or quoted).
    """

    type: Literal["Mark"] = field(default="Mark", init=False)

    content: List[Inline]
    """The content that is marked."""

    def __init__(self, content: List[Inline], id: Optional[str] = None):
        super().__init__(id = id)
        self.content = content
