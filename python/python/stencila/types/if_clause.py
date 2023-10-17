# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .code_executable import CodeExecutable


@dataclass(kw_only=True, frozen=True)
class IfClause(CodeExecutable):
    """
    A clause within a `If` node.
    """

    type: Literal["IfClause"] = field(default="IfClause", init=False)

    is_active: Optional[bool] = None
    """Whether this clause is the active clause in the parent `If` node"""

    content: List[Block]
    """The content to render if the result is truthy"""
