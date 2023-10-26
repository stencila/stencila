# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity


@dataclass(init=False)
class ThematicBreak(Entity):
    """
    A thematic break, such as a scene change in a story, a transition to another topic, or a new document.
    """

    type: Literal["ThematicBreak"] = field(default="ThematicBreak", init=False)

    def __init__(self, id: Optional[str] = None):
        super().__init__(id = id)
        
