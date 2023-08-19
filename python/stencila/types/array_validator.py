# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .validator import Validator


class ArrayValidator(BaseModel):
    """
    A validator specifying constraints on an array node.
    """

    id: Optional[str]
    """The identifier for this item"""

    items_nullable: Optional[bool]
    """Whether items can have the value `Node::Null`"""

    items_validator: Optional[Validator]
    """Another validator node specifying the constraints on all items in the array."""

    contains: Optional[Validator]
    """An array node is valid if at least one of its items is valid against the `contains` schema."""

    min_items: Optional[int]
    """An array node is valid if its size is greater than, or equal to, this value."""

    max_items: Optional[int]
    """An array node is valid if its size is less than, or equal to, this value."""

    unique_items: Optional[bool]
    """A flag to indicate that each value in the array should be unique."""
