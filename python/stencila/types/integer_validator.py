# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .number_validator import NumberValidator


@dataclass(kw_only=True, frozen=True)
class IntegerValidator(NumberValidator):
    """
    A validator specifying the constraints on an integer node.
    """

    type: Literal["IntegerValidator"] = field(default="IntegerValidator", init=False)
