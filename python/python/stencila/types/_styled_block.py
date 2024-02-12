# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._author import Author
from ._block import Block
from ._compilation_digest import CompilationDigest
from ._compilation_message import CompilationMessage
from ._cord import Cord
from ._styled import Styled


@dataclass(init=False)
class StyledBlock(Styled):
    """
    Styled block content.
    """

    type: Literal["StyledBlock"] = field(default="StyledBlock", init=False)

    content: List[Block]
    """The content within the styled block"""

    def __init__(self, code: Cord, content: List[Block], id: Optional[str] = None, style_language: Optional[str] = None, authors: Optional[List[Author]] = None, compilation_digest: Optional[CompilationDigest] = None, compilation_messages: Optional[List[CompilationMessage]] = None, css: Optional[str] = None, classes: Optional[List[str]] = None):
        super().__init__(id = id, code = code, style_language = style_language, authors = authors, compilation_digest = compilation_digest, compilation_messages = compilation_messages, css = css, classes = classes)
        self.content = content
