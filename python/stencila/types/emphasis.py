# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .mark import Mark


@dataclass(kw_only=True, frozen=True)
class Emphasis(Mark):
    """
    Emphasized content.
    """

    type: Literal["Emphasis"] = field(default="Emphasis", init=False)
