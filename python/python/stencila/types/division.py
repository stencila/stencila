# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .styled import Styled


@dataclass(kw_only=True, frozen=True)
class Division(Styled):
    """
    Styled block content.
    """

    type: Literal["Division"] = field(default="Division", init=False)

    content: List[Block]
    """The content within the division"""
