# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .executable import Executable
from .node import Node
Validator = ForwardRef("Validator")


@dataclass(kw_only=True, frozen=True)
class Parameter(Executable):
    """
    A parameter of a document.
    """

    type: Literal["Parameter"] = field(default="Parameter", init=False)

    name: str
    """The name of the parameter."""

    label: Optional[str] = None
    """A short label for the parameter."""

    value: Optional[Node] = None
    """The current value of the parameter."""

    default: Optional[Node] = None
    """The default value of the parameter."""

    validator: Optional[Validator] = None
    """The validator that the value is validated against."""

    hidden: Optional[bool] = None
    """Whether the parameter should be hidden."""

    derived_from: Optional[str] = None
    """The dotted path to the object (e.g. a database table column) that the parameter should be derived from"""
