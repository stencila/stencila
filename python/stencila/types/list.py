# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .list_item import ListItem
from .list_order import ListOrder


@dataclass(kw_only=True, frozen=True)
class List(Entity):
    """
    A list of items.
    """

    type: Literal["List"] = field(default="List", init=False)

    items: List[ListItem]
    """The items in the list."""

    order: ListOrder
    """The ordering of the list."""
