# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
Validator = ForwardRef("Validator")


@dataclass(kw_only=True, frozen=True)
class TupleValidator(Entity):
    """
    A validator specifying constraints on an array of heterogeneous items.
    """

    type: Literal["TupleValidator"] = field(default="TupleValidator", init=False)

    items: Optional[List[Validator]] = None
    """An array of validators specifying the constraints on each successive item in the array."""
