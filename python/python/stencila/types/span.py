# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .inline import Inline
from .styled import Styled


@dataclass(kw_only=True, frozen=True)
class Span(Styled):
    """
    Styled inline content.
    """

    type: Literal["Span"] = field(default="Span", init=False)

    content: List[Inline]
    """The content within the span."""
