# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity


@dataclass(init=False)
class Suggestion(Entity):
    """
    Abstract base type for nodes that indicate a suggested change to content.
    """

    type: Literal["Suggestion"] = field(default="Suggestion", init=False)

    def __init__(self, id: Optional[str] = None):
        super().__init__(id = id)
        
