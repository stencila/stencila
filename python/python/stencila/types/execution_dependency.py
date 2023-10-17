# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .execution_dependency_node import ExecutionDependencyNode
from .execution_dependency_relation import ExecutionDependencyRelation


@dataclass(kw_only=True, frozen=True)
class ExecutionDependency(Entity):
    """
    An upstream execution dependency of a node.
    """

    type: Literal["ExecutionDependency"] = field(default="ExecutionDependency", init=False)

    dependency_relation: ExecutionDependencyRelation
    """The relation to the dependency"""

    dependency_node: ExecutionDependencyNode
    """The node that is the dependency"""

    code_location: Optional[List[int]] = None
    """The location that the dependency is defined within code"""
