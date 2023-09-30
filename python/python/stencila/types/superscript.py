# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .mark import Mark


@dataclass(kw_only=True, frozen=True)
class Superscript(Mark):
    """
    Superscripted content.
    """

    type: Literal["Superscript"] = field(default="Superscript", init=False)
