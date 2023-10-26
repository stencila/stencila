# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
from .list_item import ListItem
from .list_order import ListOrder


@dataclass(init=False)
class List(Entity):
    """
    A list of items.
    """

    type: Literal["List"] = field(default="List", init=False)

    items: List[ListItem]
    """The items in the list."""

    order: ListOrder
    """The ordering of the list."""

    def __init__(self, items: List[ListItem], order: ListOrder, id: Optional[str] = None):
        super().__init__(id = id)
        self.items = items
        self.order = order
