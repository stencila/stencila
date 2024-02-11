# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._entity import Entity
from ._hint import Hint


@dataclass(init=False)
class ObjectHint(Entity):
    """
    A hint to the structure of an `Object`.
    """

    type: Literal["ObjectHint"] = field(default="ObjectHint", init=False)

    length: int
    """The length (number of entires) of the object."""

    keys: List[str]
    """The keys of the object entries."""

    values: List[Hint]
    """The types of the object entries."""

    def __init__(self, length: int, keys: List[str], values: List[Hint], id: Optional[str] = None):
        super().__init__(id = id)
        self.length = length
        self.keys = keys
        self.values = values
