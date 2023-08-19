# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .node import Node


class EnumValidator(BaseModel):
    """
    A schema specifying that a node must be one of several values.
    """

    id: Optional[str]
    """The identifier for this item"""

    values: List[Node]
    """A node is valid if it is equal to any of these values."""
