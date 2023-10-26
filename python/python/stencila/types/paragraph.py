# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .inline import Inline


@dataclass(init=False)
class Paragraph(Entity):
    """
    A paragraph.
    """

    type: Literal["Paragraph"] = field(default="Paragraph", init=False)

    content: List[Inline]
    """The contents of the paragraph."""

    def __init__(self, content: List[Inline], id: Optional[str] = None):
        super().__init__(id = id)
        self.content = content
