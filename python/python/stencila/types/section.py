# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .entity import Entity


@dataclass(kw_only=True, frozen=True)
class Section(Entity):
    """
    A section of a document.
    """

    type: Literal["Section"] = field(default="Section", init=False)

    content: List[Block]
    """The content within the section"""
