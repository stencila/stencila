# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .list_item import ListItem
from .list_order import ListOrder


class List(BaseModel):
    """
    A list of items.
    """

    id: Optional[str]
    """The identifier for this item"""

    items: List[ListItem]
    """The items in the list."""

    order: ListOrder
    """The ordering of the list."""
