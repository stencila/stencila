# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .cord import Cord
from .execution_digest import ExecutionDigest
from .styled import Styled


@dataclass(init=False)
class Division(Styled):
    """
    Styled block content.
    """

    type: Literal["Division"] = field(default="Division", init=False)

    content: List[Block]
    """The content within the division"""

    def __init__(self, code: Cord, content: List[Block], id: Optional[str] = None, style_language: Optional[str] = None, compilation_digest: Optional[ExecutionDigest] = None, compilation_errors: Optional[List[str]] = None, css: Optional[str] = None, classes: Optional[List[str]] = None):
        super().__init__(id = id, code = code, style_language = style_language, compilation_digest = compilation_digest, compilation_errors = compilation_errors, css = css, classes = classes)
        self.content = content
