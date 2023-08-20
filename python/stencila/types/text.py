# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .text_value import TextValue


@dataclass(kw_only=True, frozen=True)
class Text(Entity):
    """
    Textual content
    """

    type: Literal["Text"] = field(default="Text", init=False)

    value: TextValue
    """The value of the text content"""
