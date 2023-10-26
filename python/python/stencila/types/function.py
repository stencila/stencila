# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .parameter import Parameter
Validator = ForwardRef("Validator")


@dataclass(init=False)
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

    def __init__(self, name: str, parameters: List[Parameter], id: Optional[str] = None, returns: Optional[Validator] = None):
        super().__init__(id = id)
        self.name = name
        self.parameters = parameters
        self.returns = returns
