# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .node import Node


class ConstantValidator(BaseModel):
    """
    A validator specifying a constant value that a node must have.
    """

    id: Optional[str]
    """The identifier for this item"""

    value: Node
    """The value that the node must have."""
