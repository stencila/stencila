# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .cord import Cord
from .entity import Entity
from .execution_digest import ExecutionDigest


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

    compilation_digest: Optional[ExecutionDigest] = None
    """A digest of the `code` and `styleLanguage`."""

    compilation_errors: Optional[List[str]] = None
    """Errors that occurred when transpiling the `code`."""

    css: Optional[str] = None
    """A Cascading Style Sheet (CSS) transpiled from the `code` property."""

    classes: Optional[List[str]] = None
    """A list of class names associated with the node."""

    def __init__(self, code: Cord, id: Optional[str] = None, style_language: Optional[str] = None, compilation_digest: Optional[ExecutionDigest] = None, compilation_errors: Optional[List[str]] = None, css: Optional[str] = None, classes: Optional[List[str]] = None):
        super().__init__(id = id)
        self.code = code
        self.style_language = style_language
        self.compilation_digest = compilation_digest
        self.compilation_errors = compilation_errors
        self.css = css
        self.classes = classes
