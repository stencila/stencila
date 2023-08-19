# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .node import Node


class Variable(BaseModel):
    """
    A variable representing a name / value pair.
    """

    id: Optional[str]
    """The identifier for this item"""

    namespace: str
    """The namespace, usually a document path, within which the variable resides"""

    name: str
    """The name of the variable."""

    kind: Optional[str]
    """The expected type of variable e.g. `Number`, `Timestamp`, `Datatable`"""

    value: Optional[Node]
    """The value of the variable."""
