# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .compilation_digest import CompilationDigest
from .compilation_error import CompilationError
from .cord import Cord
from .styled import Styled


@dataclass(init=False)
class StyledBlock(Styled):
    """
    Styled block content.
    """

    type: Literal["StyledBlock"] = field(default="StyledBlock", init=False)

    content: List[Block]
    """The content within the styled block"""

    def __init__(self, code: Cord, content: List[Block], id: Optional[str] = None, style_language: Optional[str] = None, compilation_digest: Optional[CompilationDigest] = None, compilation_errors: Optional[List[CompilationError]] = None, css: Optional[str] = None, classes: Optional[List[str]] = None):
        super().__init__(id = id, code = code, style_language = style_language, compilation_digest = compilation_digest, compilation_errors = compilation_errors, css = css, classes = classes)
        self.content = content
