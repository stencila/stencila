# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .inline import Inline


@dataclass(kw_only=True, frozen=True)
class Suggestion(Entity):
    """
    Abstract base class for nodes that indicate a suggested change to inline content.
    """

    type: Literal["Suggestion"] = field(default="Suggestion", init=False)

    content: List[Inline]
    """The content that is suggested to be inserted or deleted."""
