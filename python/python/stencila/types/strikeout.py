# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .mark import Mark


@dataclass(kw_only=True, frozen=True)
class Strikeout(Mark):
    """
    Content that is marked as struck out.
    """

    type: Literal["Strikeout"] = field(default="Strikeout", init=False)
