# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .compilation_digest import CompilationDigest
from .compilation_error import CompilationError
from .cord import Cord
from .entity import Entity
from .person_or_organization_or_software_application import PersonOrOrganizationOrSoftwareApplication


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

    authors: Optional[List[PersonOrOrganizationOrSoftwareApplication]] = None
    """The authors of the math."""

    compilation_digest: Optional[CompilationDigest] = None
    """A digest of the `code` and `mathLanguage`."""

    compilation_errors: Optional[List[CompilationError]] = None
    """Errors generated when parsing and compiling the math expression."""

    mathml: Optional[str] = None
    """The MathML transpiled from the `code`."""

    def __init__(self, code: Cord, id: Optional[str] = None, math_language: Optional[str] = None, authors: Optional[List[PersonOrOrganizationOrSoftwareApplication]] = None, compilation_digest: Optional[CompilationDigest] = None, compilation_errors: Optional[List[CompilationError]] = None, mathml: Optional[str] = None):
        super().__init__(id = id)
        self.code = code
        self.math_language = math_language
        self.authors = authors
        self.compilation_digest = compilation_digest
        self.compilation_errors = compilation_errors
        self.mathml = mathml
