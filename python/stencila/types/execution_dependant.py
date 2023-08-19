# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .execution_dependant_node import ExecutionDependantNode
from .execution_dependant_relation import ExecutionDependantRelation


class ExecutionDependant(BaseModel):
    """
    A downstream execution dependant of a node
    """

    dependant_relation: ExecutionDependantRelation
    """The relation to the dependant"""

    dependant_node: ExecutionDependantNode
    """The node that is the dependant"""

    code_location: Optional[List[int]]
    """The location that the dependant is defined within code"""
