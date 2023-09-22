# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .suggestion import Suggestion


@dataclass(kw_only=True, frozen=True)
class Insert(Suggestion):
    """
    A suggestion to insert some inline content.
    """

    type: Literal["Insert"] = field(default="Insert", init=False)
