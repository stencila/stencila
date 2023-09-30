# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
Validator = ForwardRef("Validator")


@dataclass(kw_only=True, frozen=True)
class ArrayValidator(Entity):
    """
    A validator specifying constraints on an array node.
    """

    type: Literal["ArrayValidator"] = field(default="ArrayValidator", init=False)

    items_nullable: Optional[bool] = None
    """Whether items can have the value `Node::Null`"""

    items_validator: Optional[Validator] = None
    """Another validator node specifying the constraints on all items in the array."""

    contains: Optional[Validator] = None
    """An array node is valid if at least one of its items is valid against the `contains` schema."""

    min_items: Optional[int] = None
    """An array node is valid if its size is greater than, or equal to, this value."""

    max_items: Optional[int] = None
    """An array node is valid if its size is less than, or equal to, this value."""

    unique_items: Optional[bool] = None
    """A flag to indicate that each value in the array should be unique."""
