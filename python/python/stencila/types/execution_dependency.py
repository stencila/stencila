# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .code_location import CodeLocation
from .entity import Entity
from .execution_dependency_node import ExecutionDependencyNode
from .execution_dependency_relation import ExecutionDependencyRelation


@dataclass(init=False)
class ExecutionDependency(Entity):
    """
    An upstream execution dependency of a node.
    """

    type: Literal["ExecutionDependency"] = field(default="ExecutionDependency", init=False)

    dependency_relation: ExecutionDependencyRelation
    """The relation to the dependency."""

    dependency_node: ExecutionDependencyNode
    """The node that is the dependency."""

    code_location: Optional[CodeLocation] = None
    """The location that the dependency is defined."""

    def __init__(self, dependency_relation: ExecutionDependencyRelation, dependency_node: ExecutionDependencyNode, id: Optional[str] = None, code_location: Optional[CodeLocation] = None):
        super().__init__(id = id)
        self.dependency_relation = dependency_relation
        self.dependency_node = dependency_node
        self.code_location = code_location
