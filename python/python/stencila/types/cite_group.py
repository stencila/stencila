# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .cite import Cite
from .entity import Entity


@dataclass(init=False)
class CiteGroup(Entity):
    """
    A group of `Cite` nodes.
    """

    type: Literal["CiteGroup"] = field(default="CiteGroup", init=False)

    items: List[Cite]
    """One or more `Cite`s to be referenced in the same surrounding text."""

    def __init__(self, items: List[Cite], id: Optional[str] = None):
        super().__init__(id = id)
        self.items = items
