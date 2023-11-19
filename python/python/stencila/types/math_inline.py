# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .compilation_digest import CompilationDigest
from .compilation_error import CompilationError
from .cord import Cord
from .math import Math


@dataclass(init=False)
class MathInline(Math):
    """
    A fragment of math, e.g a variable name, to be treated as inline content.
    """

    type: Literal["MathInline"] = field(default="MathInline", init=False)

    def __init__(self, code: Cord, id: Optional[str] = None, math_language: Optional[str] = None, compilation_digest: Optional[CompilationDigest] = None, compilation_errors: Optional[List[CompilationError]] = None, mathml: Optional[str] = None):
        super().__init__(id = id, code = code, math_language = math_language, compilation_digest = compilation_digest, compilation_errors = compilation_errors, mathml = mathml)
        
