# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .cord import Cord
from .entity import Entity
from .execution_digest import ExecutionDigest


@dataclass(init=False)
class Math(Entity):
    """
    Abstract base type for a mathematical variable or equation.
    """

    type: Literal["Math"] = field(default="Math", init=False)

    code: Cord
    """The code of the equation in the `mathLanguage`."""

    math_language: Optional[str] = None
    """The language used for the equation e.g tex, mathml, asciimath."""

    compilation_digest: Optional[ExecutionDigest] = None
    """A digest of the `code` and `mathLanguage`."""

    compilation_errors: Optional[List[str]] = None
    """Errors that occurred when parsing and compiling the math equation."""

    mathml: Optional[str] = None
    """The MathML transpiled from the `code`."""

    def __init__(self, code: Cord, id: Optional[str] = None, math_language: Optional[str] = None, compilation_digest: Optional[ExecutionDigest] = None, compilation_errors: Optional[List[str]] = None, mathml: Optional[str] = None):
        super().__init__(id = id)
        self.code = code
        self.math_language = math_language
        self.compilation_digest = compilation_digest
        self.compilation_errors = compilation_errors
        self.mathml = mathml
