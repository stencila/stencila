# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._author import Author
from ._compilation_digest import CompilationDigest
from ._compilation_message import CompilationMessage
from ._cord import Cord
from ._math import Math


@dataclass(init=False)
class MathInline(Math):
    """
    A fragment of math, e.g a variable name, to be treated as inline content.
    """

    type: Literal["MathInline"] = field(default="MathInline", init=False)

    def __init__(self, code: Cord, id: Optional[str] = None, math_language: Optional[str] = None, authors: Optional[List[Author]] = None, compilation_digest: Optional[CompilationDigest] = None, compilation_messages: Optional[List[CompilationMessage]] = None, mathml: Optional[str] = None):
        super().__init__(id = id, code = code, math_language = math_language, authors = authors, compilation_digest = compilation_digest, compilation_messages = compilation_messages, mathml = mathml)
        
