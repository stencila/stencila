# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .node import Node


@dataclass(kw_only=True, frozen=True)
class Variable(Entity):
    """
    A variable representing a name / value pair.
    """

    type: Literal["Variable"] = field(default="Variable", init=False)

    namespace: str
    """The namespace, usually a document path, within which the variable resides"""

    name: str
    """The name of the variable."""

    kind: Optional[str] = None
    """The expected type of variable e.g. `Number`, `Timestamp`, `Datatable`"""

    value: Optional[Node] = None
    """The value of the variable."""
