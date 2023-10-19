# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .entity import Entity
ImageObject = ForwardRef("ImageObject")
from .property_value_or_str import PropertyValueOrStr
from .text import Text


@dataclass(kw_only=True, frozen=True)
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
