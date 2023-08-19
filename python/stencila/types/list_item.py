# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .blocks_or_inlines import BlocksOrInlines
from .image_object_or_str import ImageObjectOrStr
from .node import Node
from .property_value_or_str import PropertyValueOrStr


class ListItem(BaseModel):
    """
    A single item in a list.
    """

    id: Optional[str]
    """The identifier for this item"""

    alternate_names: Optional[List[str]]
    """Alternate names (aliases) for the item."""

    description: Optional[List[Block]]
    """A description of the item."""

    identifiers: Optional[List[PropertyValueOrStr]]
    """Any kind of identifier for any kind of Thing."""

    images: Optional[List[ImageObjectOrStr]]
    """Images of the item."""

    name: Optional[str]
    """The name of the item."""

    url: Optional[str]
    """The URL of the item."""

    content: Optional[BlocksOrInlines]
    """The content of the list item."""

    item: Optional[Node]
    """The item represented by this list item."""

    is_checked: Optional[bool]
    """A flag to indicate if this list item is checked."""

    position: Optional[int]
    """The position of the item in a series or sequence of items."""
