# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .cord import Cord
from .execution_digest import ExecutionDigest
from .math import Math


@dataclass(init=False)
class MathFragment(Math):
    """
    A fragment of math, e.g a variable name, to be treated as inline content.
    """

    type: Literal["MathFragment"] = field(default="MathFragment", init=False)

    def __init__(self, math_language: str, code: Cord, id: Optional[str] = None, compile_digest: Optional[ExecutionDigest] = None, errors: Optional[List[str]] = None, mathml: Optional[str] = None):
        super().__init__(id = id, math_language = math_language, code = code, compile_digest = compile_digest, errors = errors, mathml = mathml)
        
