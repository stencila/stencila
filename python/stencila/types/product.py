# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .brand import Brand
from .image_object_or_str import ImageObjectOrStr
from .property_value_or_str import PropertyValueOrStr


class Product(BaseModel):
    """
    Any offered product or service. For example, a pair of shoes;    a haircut; or an episode of a TV show streamed online.
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

    brands: Optional[List[Brand]]
    """Brands that the product is labelled with."""

    logo: Optional[ImageObjectOrStr]
    """The logo of the product."""

    product_id: Optional[str]
    """Product identification code."""
