# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .inline import Inline
from .mark import Mark


@dataclass(kw_only=True, frozen=True)
class Strong(Mark):
    """
    Strongly emphasised content.
    """

    type: Literal["Strong"] = field(default="Strong", init=False)

    content: List[Inline]
    """The content that is marked."""
