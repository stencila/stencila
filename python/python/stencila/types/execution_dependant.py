# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .execution_dependant_node import ExecutionDependantNode
from .execution_dependant_relation import ExecutionDependantRelation


@dataclass(kw_only=True, frozen=True)
class ExecutionDependant(Entity):
    """
    A downstream execution dependant of a node.
    """

    type: Literal["ExecutionDependant"] = field(default="ExecutionDependant", init=False)

    dependant_relation: ExecutionDependantRelation
    """The relation to the dependant"""

    dependant_node: ExecutionDependantNode
    """The node that is the dependant"""

    code_location: Optional[List[int]] = None
    """The location that the dependant is defined within code"""
