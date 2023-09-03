# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity


@dataclass(kw_only=True, frozen=True)
class ThematicBreak(Entity):
    """
    A thematic break, such as a scene change in a story, a transition to another topic, or a new document.
    """

    type: Literal["ThematicBreak"] = field(default="ThematicBreak", init=False)
