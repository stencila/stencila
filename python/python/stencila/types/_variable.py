# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from ._entity import Entity
from ._hint import Hint
from ._node import Node


@dataclass(init=False)
class Variable(Entity):
    """
    A variable representing a name / value pair.
    """

    type: Literal["Variable"] = field(default="Variable", init=False)

    name: str
    """The name of the variable."""

    programming_language: Optional[str] = None
    """The programming language that the variable is defined in e.g. Python, JSON."""

    native_type: Optional[str] = None
    """The native type of the variable e.g. `float`, `datetime.datetime`, `pandas.DataFrame`"""

    node_type: Optional[str] = None
    """The Stencila node type of the variable e.g. `Number`, `DateTime`, `Datatable`."""

    value: Optional[Node] = None
    """The value of the variable."""

    hint: Optional[Hint] = None
    """A hint to the value and/or structure of the variable."""

    native_hint: Optional[str] = None
    """A textual hint to the value and/or structure of the variable."""

    def __init__(self, name: str, id: Optional[str] = None, programming_language: Optional[str] = None, native_type: Optional[str] = None, node_type: Optional[str] = None, value: Optional[Node] = None, hint: Optional[Hint] = None, native_hint: Optional[str] = None):
        super().__init__(id = id)
        self.name = name
        self.programming_language = programming_language
        self.native_type = native_type
        self.node_type = node_type
        self.value = value
        self.hint = hint
        self.native_hint = native_hint
