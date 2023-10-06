# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .inline import Inline


@dataclass(kw_only=True, frozen=True)
class Heading(Entity):
    """
    A heading.
    """

    type: Literal["Heading"] = field(default="Heading", init=False)

    level: int = 0
    """The level of the heading."""

    content: List[Inline]
    """Content of the heading."""
