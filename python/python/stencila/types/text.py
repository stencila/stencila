# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .cord import Cord
from .entity import Entity


@dataclass(kw_only=True, frozen=True)
class Text(Entity):
    """
    Textual content.
    """

    type: Literal["Text"] = field(default="Text", init=False)

    value: Cord
    """The value of the text content"""
