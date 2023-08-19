# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .image_object_or_str import ImageObjectOrStr
from .person_or_organization import PersonOrOrganization
from .property_value_or_str import PropertyValueOrStr
from .thing import Thing


class MonetaryGrant(BaseModel):
    """
    A monetary grant.
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

    funded_items: Optional[List[Thing]]
    """Indicates an item funded or sponsored through a Grant."""

    sponsors: Optional[List[PersonOrOrganization]]
    """A person or organization that supports a thing through a pledge, promise, or financial contribution."""

    amounts: Optional[float]
    """The amount of money."""

    funders: Optional[List[PersonOrOrganization]]
    """A person or organization that supports (sponsors) something through some kind of financial contribution."""
