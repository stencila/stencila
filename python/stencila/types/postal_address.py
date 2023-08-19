# Generated file; do not edit. See the Rust `schema-gen` crate.

from .prelude import *

from .block import Block
from .image_object_or_str import ImageObjectOrStr
from .property_value_or_str import PropertyValueOrStr


class PostalAddress(BaseModel):
    """
    A physical mailing address.
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

    emails: Optional[List[str]]
    """Email address for correspondence."""

    telephone_numbers: Optional[List[str]]
    """Telephone numbers for the contact point."""

    available_languages: Optional[List[str]]
    """Languages (human not programming) in which it is possible to communicate with the organization/department etc."""

    street_address: Optional[str]
    """The street address."""

    post_office_box_number: Optional[str]
    """The post office box number."""

    address_locality: Optional[str]
    """The locality in which the street address is, and which is in the region."""

    address_region: Optional[str]
    """The region in which the locality is, and which is in the country."""

    postal_code: Optional[str]
    """The postal code."""

    address_country: Optional[str]
    """The country."""
