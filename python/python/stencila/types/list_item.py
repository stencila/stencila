# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
ImageObject = ForwardRef("ImageObject")
from .node import Node
from .property_value_or_str import PropertyValueOrStr
from .text import Text
from .thing import Thing


@dataclass(init=False)
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

    def __init__(self, content: List[Block], id: Optional[str] = None, alternate_names: Optional[List[str]] = None, description: Optional[Text] = None, identifiers: Optional[List[PropertyValueOrStr]] = None, images: Optional[List[ImageObject]] = None, name: Optional[str] = None, url: Optional[str] = None, item: Optional[Node] = None, is_checked: Optional[bool] = None, position: Optional[int] = None):
        super().__init__(id = id, alternate_names = alternate_names, description = description, identifiers = identifiers, images = images, name = name, url = url)
        self.content = content
        self.item = item
        self.is_checked = is_checked
        self.position = position
