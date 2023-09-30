# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .math import Math


@dataclass(kw_only=True, frozen=True)
class MathBlock(Math):
    """
    A block of math, e.g an equation, to be treated as block content.
    """

    type: Literal["MathBlock"] = field(default="MathBlock", init=False)

    label: Optional[str] = None
    """A short label for the math block."""
