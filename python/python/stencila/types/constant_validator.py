# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .node import Node


@dataclass(init=False)
class ConstantValidator(Entity):
    """
    A validator specifying a constant value that a node must have.
    """

    type: Literal["ConstantValidator"] = field(default="ConstantValidator", init=False)

    value: Node
    """The value that the node must have."""

    def __init__(self, value: Node, id: Optional[str] = None):
        super().__init__(id = id)
        self.value = value
