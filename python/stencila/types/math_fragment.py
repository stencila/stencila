# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .math import Math


@dataclass(kw_only=True, frozen=True)
class MathFragment(Math):
    """
    A fragment of math, e.g a variable name, to be treated as inline content.
    """

    type: Literal["MathFragment"] = field(default="MathFragment", init=False)
