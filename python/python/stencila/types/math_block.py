# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .cord import Cord
from .execution_digest import ExecutionDigest
from .math import Math


@dataclass(init=False)
class MathBlock(Math):
    """
    A block of math, e.g an equation, to be treated as block content.
    """

    type: Literal["MathBlock"] = field(default="MathBlock", init=False)

    label: Optional[str] = None
    """A short label for the math block."""

    def __init__(self, math_language: str, code: Cord, id: Optional[str] = None, compile_digest: Optional[ExecutionDigest] = None, errors: Optional[List[str]] = None, mathml: Optional[str] = None, label: Optional[str] = None):
        super().__init__(id = id, math_language = math_language, code = code, compile_digest = compile_digest, errors = errors, mathml = mathml)
        self.label = label
