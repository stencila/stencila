# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .cord import Cord
from .entity import Entity


@dataclass(init=False)
class Text(Entity):
    """
    Textual content.
    """

    type: Literal["Text"] = field(default="Text", init=False)

    value: Cord
    """The value of the text content"""

    def __init__(self, value: Cord, id: Optional[str] = None):
        super().__init__(id = id)
        self.value = value
