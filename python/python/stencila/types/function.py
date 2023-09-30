# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .parameter import Parameter
Validator = ForwardRef("Validator")


@dataclass(kw_only=True, frozen=True)
class Function(Entity):
    """
    A function with a name, which might take Parameters and return a value of a certain type.
    """

    type: Literal["Function"] = field(default="Function", init=False)

    name: str
    """The name of the function."""

    parameters: List[Parameter]
    """The parameters of the function."""

    returns: Optional[Validator] = None
    """The return type of the function."""
