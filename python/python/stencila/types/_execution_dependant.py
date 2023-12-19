# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._code_location import CodeLocation
from ._entity import Entity
from ._execution_dependant_node import ExecutionDependantNode
from ._execution_dependant_relation import ExecutionDependantRelation


@dataclass(init=False)
class ExecutionDependant(Entity):
    """
    A downstream execution dependant of a node.
    """

    type: Literal["ExecutionDependant"] = field(default="ExecutionDependant", init=False)

    dependant_relation: ExecutionDependantRelation
    """The relation to the dependant."""

    dependant_node: ExecutionDependantNode
    """The node that is the dependant."""

    code_location: Optional[CodeLocation] = None
    """The location that the dependant is defined."""

    def __init__(self, dependant_relation: ExecutionDependantRelation, dependant_node: ExecutionDependantNode, id: Optional[str] = None, code_location: Optional[CodeLocation] = None):
        super().__init__(id = id)
        self.dependant_relation = dependant_relation
        self.dependant_node = dependant_node
        self.code_location = code_location
