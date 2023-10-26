# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity


@dataclass(init=False)
class BooleanValidator(Entity):
    """
    A schema specifying that a node must be a boolean value.
    """

    type: Literal["BooleanValidator"] = field(default="BooleanValidator", init=False)

    def __init__(self, id: Optional[str] = None):
        super().__init__(id = id)
        
