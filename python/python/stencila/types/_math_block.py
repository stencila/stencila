# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._author import Author
from ._compilation_digest import CompilationDigest
from ._compilation_message import CompilationMessage
from ._cord import Cord
from ._math import Math


@dataclass(init=False)
class MathBlock(Math):
    """
    A block of math, e.g an equation, to be treated as block content.
    """

    type: Literal["MathBlock"] = field(default="MathBlock", init=False)

    label: Optional[str] = None
    """A short label for the math block."""

    def __init__(self, code: Cord, id: Optional[str] = None, math_language: Optional[str] = None, authors: Optional[List[Author]] = None, compilation_digest: Optional[CompilationDigest] = None, compilation_messages: Optional[List[CompilationMessage]] = None, mathml: Optional[str] = None, label: Optional[str] = None):
        super().__init__(id = id, code = code, math_language = math_language, authors = authors, compilation_digest = compilation_digest, compilation_messages = compilation_messages, mathml = mathml)
        self.label = label
