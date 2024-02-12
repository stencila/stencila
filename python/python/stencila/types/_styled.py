# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._author import Author
from ._compilation_digest import CompilationDigest
from ._compilation_message import CompilationMessage
from ._cord import Cord
from ._entity import Entity


@dataclass(init=False)
class Styled(Entity):
    """
    An abstract base class for a document node that has styling applied to it and/or its content.
    """

    type: Literal["Styled"] = field(default="Styled", init=False)

    code: Cord
    """The code of the equation in the `styleLanguage`."""

    style_language: Optional[str] = None
    """The language used for the style specification e.g. css, tw"""

    authors: Optional[List[Author]] = None
    """The authors of the styling code."""

    compilation_digest: Optional[CompilationDigest] = None
    """A digest of the `code` and `styleLanguage`."""

    compilation_messages: Optional[List[CompilationMessage]] = None
    """Messages generated while parsing and transpiling the style."""

    css: Optional[str] = None
    """A Cascading Style Sheet (CSS) transpiled from the `code` property."""

    classes: Optional[List[str]] = None
    """A list of class names associated with the node."""

    def __init__(self, code: Cord, id: Optional[str] = None, style_language: Optional[str] = None, authors: Optional[List[Author]] = None, compilation_digest: Optional[CompilationDigest] = None, compilation_messages: Optional[List[CompilationMessage]] = None, css: Optional[str] = None, classes: Optional[List[str]] = None):
        super().__init__(id = id)
        self.code = code
        self.style_language = style_language
        self.authors = authors
        self.compilation_digest = compilation_digest
        self.compilation_messages = compilation_messages
        self.css = css
        self.classes = classes
