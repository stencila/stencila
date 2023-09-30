# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .mark import Mark


@dataclass(kw_only=True, frozen=True)
class Subscript(Mark):
    """
    Subscripted content.
    """

    type: Literal["Subscript"] = field(default="Subscript", init=False)
