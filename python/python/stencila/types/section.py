# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .entity import Entity
from .section_type import SectionType


@dataclass(init=False)
class Section(Entity):
    """
    A section of a document.
    """

    type: Literal["Section"] = field(default="Section", init=False)

    content: List[Block]
    """The content within the section."""

    section_type: Optional[SectionType] = None
    """The type of section."""

    def __init__(self, content: List[Block], id: Optional[str] = None, section_type: Optional[SectionType] = None):
        super().__init__(id = id)
        self.content = content
        self.section_type = section_type
