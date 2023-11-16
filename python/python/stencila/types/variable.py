# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .node import Node


@dataclass(init=False)
class Variable(Entity):
    """
    A variable representing a name / value pair.
    """

    type: Literal["Variable"] = field(default="Variable", init=False)

    name: str
    """The name of the variable."""

    kind: Optional[str] = None
    """The expected type of variable e.g. `Number`, `Timestamp`, `Datatable`"""

    value: Optional[Node] = None
    """The value of the variable."""

    def __init__(self, name: str, id: Optional[str] = None, kind: Optional[str] = None, value: Optional[Node] = None):
        super().__init__(id = id)
        self.name = name
        self.kind = kind
        self.value = value
