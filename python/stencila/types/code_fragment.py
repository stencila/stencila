# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .code_static import CodeStatic


@dataclass(kw_only=True, frozen=True)
class CodeFragment(CodeStatic):
    """
    Inline code.
    """

    type: Literal["CodeFragment"] = field(default="CodeFragment", init=False)
