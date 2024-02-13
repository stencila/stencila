# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._entity import Entity


@dataclass(init=False)
class StringHint(Entity):
    """
    A hint to the structure of an `String`.
    """

    type: Literal["StringHint"] = field(default="StringHint", init=False)

    chars: int
    """The number of characters in the string."""

    def __init__(self, chars: int, id: Optional[str] = None):
        super().__init__(id = id)
        self.chars = chars
