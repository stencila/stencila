# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .primitive import Primitive
from .thing import Thing


@dataclass(kw_only=True, frozen=True)
class PropertyValue(Thing):
    """
    A property-value pair.
    """

    type: Literal["PropertyValue"] = field(default="PropertyValue", init=False)

    property_id: Optional[str] = None
    """A commonly used identifier for the characteristic represented by the property."""

    value: Primitive
    """The value of the property."""
