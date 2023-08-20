# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .array import Array
from .block import Block
from .code_executable import CodeExecutable


@dataclass(kw_only=True, frozen=True)
class For(CodeExecutable):
    """
    Repeat a block content for each item in an array.
    """

    type: Literal["For"] = field(default="For", init=False)

    symbol: str
    """The name to give to the variable representing each item in the iterated array"""

    content: List[Block]
    """The content to repeat for each item"""

    otherwise: Optional[List[Block]] = None
    """The content to render if there are no items"""

    iterations: Optional[List[Array]] = None
    """The content repeated for each iteration"""
