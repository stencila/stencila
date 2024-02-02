# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._author import Author
from ._cord import Cord
from ._entity import Entity


@dataclass(init=False)
class CodeStatic(Entity):
    """
    Abstract base type for non-executable code nodes (e.g. `CodeBlock`).
    """

    type: Literal["CodeStatic"] = field(default="CodeStatic", init=False)

    code: Cord
    """The code."""

    programming_language: Optional[str] = None
    """The programming language of the code."""

    authors: Optional[List[Author]] = None
    """The authors of the code."""

    def __init__(self, code: Cord, id: Optional[str] = None, programming_language: Optional[str] = None, authors: Optional[List[Author]] = None):
        super().__init__(id = id)
        self.code = code
        self.programming_language = programming_language
        self.authors = authors
