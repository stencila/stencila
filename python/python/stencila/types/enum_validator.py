# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .node import Node


@dataclass(init=False)
class EnumValidator(Entity):
    """
    A schema specifying that a node must be one of several values.
    """

    type: Literal["EnumValidator"] = field(default="EnumValidator", init=False)

    values: List[Node]
    """A node is valid if it is equal to any of these values."""

    def __init__(self, values: List[Node], id: Optional[str] = None):
        super().__init__(id = id)
        self.values = values
