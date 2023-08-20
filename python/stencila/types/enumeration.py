# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .image_object_or_str import ImageObjectOrStr
from .property_value_or_str import PropertyValueOrStr
from .thing import Thing


@dataclass(kw_only=True, frozen=True)
class Enumeration(Thing):
    """
    Lists or enumerations, for example, a list of cuisines or music genres, etc.
    """

    type: Literal["Enumeration"] = field(default="Enumeration", init=False)

    alternate_names: Optional[List[str]] = None
    """Alternate names (aliases) for the item."""

    description: Optional[List[Block]] = None
    """A description of the item."""

    identifiers: Optional[List[PropertyValueOrStr]] = None
    """Any kind of identifier for any kind of Thing."""

    images: Optional[List[ImageObjectOrStr]] = None
    """Images of the item."""

    name: Optional[str] = None
    """The name of the item."""

    url: Optional[str] = None
    """The URL of the item."""
