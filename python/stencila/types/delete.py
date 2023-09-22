# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .suggestion import Suggestion


@dataclass(kw_only=True, frozen=True)
class Delete(Suggestion):
    """
    A suggestion to delete some inline content.
    """

    type: Literal["Delete"] = field(default="Delete", init=False)
