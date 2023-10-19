# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .node import Node
from .thing import Thing


@dataclass(kw_only=True, frozen=True)
class ListItem(Thing):
    """
    A single item in a list.
    """

    type: Literal["ListItem"] = field(default="ListItem", init=False)

    content: List[Block]
    """The content of the list item."""

    item: Optional[Node] = None
    """The item represented by this list item."""

    is_checked: Optional[bool] = None
    """A flag to indicate if this list item is checked."""

    position: Optional[int] = None
    """The position of the item in a series or sequence of items."""
