# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._author import Author
from ._entity import Entity
from ._inline import Inline


@dataclass(init=False)
class Heading(Entity):
    """
    A heading.
    """

    type: Literal["Heading"] = field(default="Heading", init=False)

    level: int = 0
    """The level of the heading."""

    content: List[Inline]
    """Content of the heading."""

    authors: Optional[List[Author]] = None
    """The authors of the heading."""

    def __init__(self, content: List[Inline], id: Optional[str] = None, level: int = 0, authors: Optional[List[Author]] = None):
        super().__init__(id = id)
        self.level = level
        self.content = content
        self.authors = authors