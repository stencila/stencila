# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._entity import Entity


@dataclass(init=False)
class Unknown(Entity):
    """
    A type to indicate a value or or other type in unknown.
    """

    type: Literal["Unknown"] = field(default="Unknown", init=False)

    def __init__(self, id: Optional[str] = None):
        super().__init__(id = id)
        
