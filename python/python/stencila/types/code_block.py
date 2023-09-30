# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .code_static import CodeStatic


@dataclass(kw_only=True, frozen=True)
class CodeBlock(CodeStatic):
    """
    A code block.
    """

    type: Literal["CodeBlock"] = field(default="CodeBlock", init=False)
