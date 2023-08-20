# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .code_executable import CodeExecutable
from .node import Node


@dataclass(kw_only=True, frozen=True)
class CodeExpression(CodeExecutable):
    """
    An executable programming code expression.
    """

    type: Literal["CodeExpression"] = field(default="CodeExpression", init=False)

    output: Optional[Node] = None
    """The value of the expression when it was last evaluated."""
