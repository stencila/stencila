# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .number_validator import NumberValidator


@dataclass(kw_only=True, frozen=True)
class IntegerValidator(NumberValidator):
    """
    A validator specifying the constraints on an integer node.
    """

    type: Literal["IntegerValidator"] = field(default="IntegerValidator", init=False)

    minimum: Optional[float] = None
    """The inclusive lower limit for a numeric node."""

    exclusive_minimum: Optional[float] = None
    """The exclusive lower limit for a numeric node."""

    maximum: Optional[float] = None
    """The inclusive upper limit for a numeric node."""

    exclusive_maximum: Optional[float] = None
    """The exclusive upper limit for a numeric node."""

    multiple_of: Optional[float] = None
    """A number that a numeric node must be a multiple of."""
