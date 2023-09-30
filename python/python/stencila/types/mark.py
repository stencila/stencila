# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .inline import Inline


@dataclass(kw_only=True, frozen=True)
class Mark(Entity):
    """
    Abstract base class for nodes that mark some other inline content    in some way (e.g. as being emphasised, or quoted).
    """

    type: Literal["Mark"] = field(default="Mark", init=False)

    content: List[Inline]
    """The content that is marked."""
