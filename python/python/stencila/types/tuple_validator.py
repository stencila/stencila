# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
Validator = ForwardRef("Validator")


@dataclass(init=False)
class TupleValidator(Entity):
    """
    A validator specifying constraints on an array of heterogeneous items.
    """

    type: Literal["TupleValidator"] = field(default="TupleValidator", init=False)

    items: Optional[List[Validator]] = None
    """An array of validators specifying the constraints on each successive item in the array."""

    def __init__(self, id: Optional[str] = None, items: Optional[List[Validator]] = None):
        super().__init__(id = id)
        self.items = items
