# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity


@dataclass(init=False)
class Role(Entity):
    """
    Represents additional information about a relationship or property.
    """

    type: Literal["Role"] = field(default="Role", init=False)

    def __init__(self, id: Optional[str] = None):
        super().__init__(id = id)
        
