# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .mark import Mark


@dataclass(kw_only=True, frozen=True)
class Strong(Mark):
    """
    Strongly emphasized content.
    """

    type: Literal["Strong"] = field(default="Strong", init=False)
