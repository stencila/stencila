# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .inline import Inline
from .mark import Mark


@dataclass(kw_only=True, frozen=True)
class Strikeout(Mark):
    """
    Content that is marked as struck out
    """

    type: Literal["Strikeout"] = field(default="Strikeout", init=False)

    content: List[Inline]
    """The content that is marked."""
