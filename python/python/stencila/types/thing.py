# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
ImageObject = ForwardRef("ImageObject")
from .property_value_or_str import PropertyValueOrStr
from .text import Text


@dataclass(init=False)
class Thing(Entity):
    """
    The most generic type of item.
    """

    type: Literal["Thing"] = field(default="Thing", init=False)

    alternate_names: Optional[List[str]] = None
    """Alternate names (aliases) for the item."""

    description: Optional[Text] = None
    """A description of the item."""

    identifiers: Optional[List[PropertyValueOrStr]] = None
    """Any kind of identifier for any kind of Thing."""

    images: Optional[List[ImageObject]] = None
    """Images of the item."""

    name: Optional[str] = None
    """The name of the item."""

    url: Optional[str] = None
    """The URL of the item."""

    def __init__(self, id: Optional[str] = None, alternate_names: Optional[List[str]] = None, description: Optional[Text] = None, identifiers: Optional[List[PropertyValueOrStr]] = None, images: Optional[List[ImageObject]] = None, name: Optional[str] = None, url: Optional[str] = None):
        super().__init__(id = id)
        self.alternate_names = alternate_names
        self.description = description
        self.identifiers = identifiers
        self.images = images
        self.name = name
        self.url = url
