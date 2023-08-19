# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .execution_dependency_node import ExecutionDependencyNode
from .execution_dependency_relation import ExecutionDependencyRelation


class ExecutionDependency(BaseModel):
    """
    An upstream execution dependency of a node
    """

    dependency_relation: ExecutionDependencyRelation
    """The relation to the dependency"""

    dependency_node: ExecutionDependencyNode
    """The node that is the dependency"""

    code_location: Optional[List[int]]
    """The location that the dependency is defined within code"""
