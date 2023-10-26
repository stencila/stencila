# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .entity import Entity


@dataclass(init=False)
class Section(Entity):
    """
    A section of a document.
    """

    type: Literal["Section"] = field(default="Section", init=False)

    content: List[Block]
    """The content within the section"""

    def __init__(self, content: List[Block], id: Optional[str] = None):
        super().__init__(id = id)
        self.content = content
