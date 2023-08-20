# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .inline import Inline
from .mark import Mark


@dataclass(kw_only=True, frozen=True)
class Subscript(Mark):
    """
    Subscripted content.
    """

    type: Literal["Subscript"] = field(default="Subscript", init=False)

    content: List[Inline]
    """The content that is marked."""
