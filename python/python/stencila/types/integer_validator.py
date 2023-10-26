# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .number_validator import NumberValidator


@dataclass(init=False)
class IntegerValidator(NumberValidator):
    """
    A validator specifying the constraints on an integer node.
    """

    type: Literal["IntegerValidator"] = field(default="IntegerValidator", init=False)

    def __init__(self, id: Optional[str] = None, minimum: Optional[float] = None, exclusive_minimum: Optional[float] = None, maximum: Optional[float] = None, exclusive_maximum: Optional[float] = None, multiple_of: Optional[float] = None):
        super().__init__(id = id, minimum = minimum, exclusive_minimum = exclusive_minimum, maximum = maximum, exclusive_maximum = exclusive_maximum, multiple_of = multiple_of)
        
