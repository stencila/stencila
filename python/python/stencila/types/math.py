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

    math_language: str
    """The language used for the equation e.g tex, mathml, asciimath."""

    code: Cord
    """The code of the equation in the `mathLanguage`."""

    compile_digest: Optional[ExecutionDigest] = None
    """A digest of the `code` and `mathLanguage`."""

    errors: Optional[List[str]] = None
    """Errors that occurred when parsing the math equation."""

    mathml: Optional[str] = None
    """The MathML transpiled from the `code`."""

    def __init__(self, math_language: str, code: Cord, id: Optional[str] = None, compile_digest: Optional[ExecutionDigest] = None, errors: Optional[List[str]] = None, mathml: Optional[str] = None):
        super().__init__(id = id)
        self.math_language = math_language
        self.code = code
        self.compile_digest = compile_digest
        self.errors = errors
        self.mathml = mathml
